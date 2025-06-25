#[cfg(feature = "vty")]
pub mod vty {

    use alloc::boxed::Box;
    use alloc::vec::Vec;
    use embedded_graphics_core::prelude::OriginDimensions;

    use crate::gpu::canvas::{AlphaBlend, CanvasManager, RGBA8, sealed::CanvasColorFormat};
    use crate::result::Result;

    use embedded_graphics_core::Pixel;
    pub use embedded_graphics_core::draw_target::DrawTarget;
    pub use embedded_graphics_core::geometry::{Dimensions, Point, Size};
    pub use embedded_graphics_core::pixelcolor::{Rgb888, RgbColor};
    pub use embedded_graphics_core::primitives::rectangle::Rectangle;

    pub struct PersistantBufferedCanvas {
        buffer: Box<[Rgb888]>,
        canvas: CanvasManager<Rgb888>,
    }

    impl PersistantBufferedCanvas {
        pub fn new(canvas: CanvasManager<Rgb888>) -> Self {
            // new buffer with reserved size
            let mut buffer =
                Vec::with_capacity((canvas.surface.width() * canvas.surface.height()) as usize);
            // fill the buffer with black pixels to start
            buffer.resize(
                (canvas.surface.width() * canvas.surface.height()) as usize,
                Rgb888::new(0, 0, 0),
            );

            Self {
                buffer: buffer.into_boxed_slice(),
                canvas,
            }
        }
    }

    impl CanvasColorFormat for Rgb888 {
        type RawType = u32;

        const COLOR_FORMAT: crate::gpu::ColorFormat = <RGBA8 as CanvasColorFormat>::COLOR_FORMAT;
        const PIXEL_FORMAT: crate::gpu::PixelFormat = <RGBA8 as CanvasColorFormat>::PIXEL_FORMAT;

        fn blend_with(self, other: Self, blend: AlphaBlend) -> Self {
            if matches!(blend, AlphaBlend::Destination) {
                other
            } else {
                self
            }
        }

        fn from_raw(raw: Self::RawType) -> Self {
            let intermediate = RGBA8::from_bits(raw);
            Rgb888::new(intermediate.r(), intermediate.g(), intermediate.b())
        }

        fn new() -> Self {
            Rgb888::new(0, 0, 0)
        }

        fn new_scaled(r: u8, g: u8, b: u8, _a: u8) -> Self {
            Rgb888::new(r, g, b)
        }

        fn scale_alpha(self, _alpha: f32) -> Self {
            self
        }

        fn to_raw(self) -> Self::RawType {
            RGBA8::new_scaled(self.r(), self.g(), self.b(), 255).to_raw()
        }
    }

    impl OriginDimensions for PersistantBufferedCanvas {
        fn size(&self) -> Size {
            Size {
                width: self.canvas.surface.width(),
                height: self.canvas.surface.height(),
            }
        }
    }

    impl DrawTarget for PersistantBufferedCanvas {
        type Color = Rgb888;
        type Error = crate::result::ResultCode;
        fn draw_iter<I>(&mut self, pixels: I) -> Result<()>
        where
            I: IntoIterator<Item = Pixel<Self::Color>>,
        {
            for Pixel(Point { x, y }, color) in pixels.into_iter() {
                self.buffer[(x + y * self.canvas.surface.width() as i32) as usize] = color;
            }

            self.canvas.render_prepared_buffer(self.buffer.as_ref())?;

            self.canvas.wait_vsync_event(None)
        }

        fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<()>
        where
            I: IntoIterator<Item = Self::Color>,
        {
            let Rectangle {
                top_left: Point { x, y },
                size: Size { width, height },
            } = *area;

            let mut color_iter = colors.into_iter().peekable();

            if color_iter.peek().is_none() {
                // no point iterating and rendering
                return Ok(());
            }

            for y in y..(y + height as i32) {
                for x in x..(x + width as i32) {
                    if let Some(color) = color_iter.next() {
                        self.buffer[(x + y * self.canvas.surface.width() as i32) as usize] = color;
                    }
                }
            }
            self.canvas.render_prepared_buffer(self.buffer.as_ref())?;

            self.canvas.wait_vsync_event(None)
        }

        fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<()> {
            let Rectangle {
                top_left: Point { x, y },
                size: Size { width, height },
            } = *area;
            for y in y..(y + height as i32) {
                for x in x..(x + width as i32) {
                    self.buffer[(x + y * self.canvas.surface.width() as i32) as usize] = color;
                }
            }
            self.canvas.render_prepared_buffer(self.buffer.as_ref())?;

            self.canvas.wait_vsync_event(None)
        }
    }
}
pub mod scrollback {
    use core::num::NonZeroU16;

    use crate::{
        gpu::{
            self,
            canvas::{Canvas, CanvasManager, RGBA4},
        },
        result::Result,
        sync::RwLock,
    };

    use crate::sync::Mutex;
    use crate::thread::{Builder, JoinHandle};
    use alloc::{
        collections::vec_deque::VecDeque,
        string::{String, ToString},
        sync::Arc,
    };

    #[derive(Clone)]
    pub struct BackgroundWriter {
        inner: Arc<Mutex<VecDeque<String>>>,
    }

    impl BackgroundWriter {
        pub fn new(
            gpu_ctx: Arc<RwLock<gpu::Context>>,
            history_limit: u16,
            line_max_chars: NonZeroU16,
            line_wrap: bool,
            text_color: Option<RGBA4>,
            scale: u8,
        ) -> Result<Self> {
            let mut console = ScrollbackConsole::new(
                gpu_ctx,
                history_limit,
                line_max_chars,
                line_wrap,
                text_color,
                scale,
            )?;

            let fake_channel = Arc::new(Mutex::new(VecDeque::new()));
            let _background_thread: JoinHandle<Result<()>> = {
                let receiver = Arc::downgrade(&fake_channel);
                Builder::new()
                    .name("console")
                    .stack_size(0x1000)
                    .spawn(move || {
                        while let Some(reader) = receiver.upgrade() {
                            {
                                let mut reader = reader.lock();
                                while let Some(message) = reader.pop_front() {
                                    console.write(message);
                                }
                            }
                            console.draw()?;

                            console.wait_vsync_event(None)?;
                        }
                        Ok(())
                    })?
            };
            Ok(Self {
                inner: fake_channel,
            })
        }

        #[inline(always)]
        pub fn write(&self, message: impl ToString) {
            self.inner.lock().push_back(message.to_string());
        }
    }

    /// This console creates a full-screen layer that will just scroll through provided strings
    pub struct ScrollbackConsole {
        canvas: CanvasManager<RGBA4>,
        pub text_color: RGBA4,
        pub history_limit: u16,
        pub line_max_chars: u16,
        pub line_wrap: bool,
        scrollback_history: alloc::collections::VecDeque<String>,
        pub scrollback_history_offset: u16,
        current_line: String,
        pub scale: u8,
    }

    unsafe impl Send for ScrollbackConsole {}
    unsafe impl Sync for ScrollbackConsole {}

    impl ScrollbackConsole {
        #[inline(always)]
        pub fn new(
            gpu_ctx: Arc<RwLock<gpu::Context>>,
            history_limit: u16,
            line_max_chars: NonZeroU16,
            line_wrap: bool,
            text_color: Option<RGBA4>,
            scale: u8,
        ) -> Result<Self> {
            let canvas = nx::gpu::canvas::CanvasManager::new_stray(
                gpu_ctx,
                Default::default(),
                3,
                gpu::BlockLinearHeights::OneGob,
            )?;
            Ok(Self {
                history_limit,
                text_color: text_color.unwrap_or(RGBA4::from_bits(u16::MAX)),
                line_wrap,
                line_max_chars: line_max_chars.get().min(canvas.surface.width() as u16 / 8),
                scrollback_history: VecDeque::with_capacity(history_limit as _),
                current_line: String::new(),
                canvas,
                scrollback_history_offset: 0,
                scale,
            })
        }

        #[inline(always)]
        pub fn scroll_up(&mut self) {
            let max_line_count = self.max_line_count();

            let history_len = self.scrollback_history.len();
            if history_len > max_line_count as usize - 1 {
                self.scrollback_history_offset = self
                    .scrollback_history_offset
                    .saturating_add(1)
                    .min(history_len as _);
            }
        }

        #[inline(always)]
        pub fn scroll_down(&mut self) {
            self.scrollback_history_offset = self.scrollback_history_offset.saturating_sub(1);
        }

        fn push_line(&mut self, text: &str, commit: bool) {
            self.current_line.push_str(text);

            let real_max_len = (self.line_max_chars as u32)
                .min((self.canvas.surface.width() - 4) / (8 * self.scale as u32))
                as usize;

            if !self.line_wrap && self.current_line.len() > real_max_len {
                self.current_line.truncate(real_max_len - 1);
                self.current_line.push('>');
            } else {
                while self.current_line.len() > real_max_len {
                    let mut temp = core::mem::take(&mut self.current_line);
                    let new_line = temp.split_off(real_max_len);
                    self.push_history_line(temp);
                    self.current_line = new_line;
                }
            }

            if commit {
                let commit_str = core::mem::take(&mut self.current_line);
                self.push_history_line(commit_str);
            }
        }

        #[inline(always)]
        pub fn push_history_line(&mut self, line: String) {
            let real_max_len =
                self.line_max_chars
                    .min(self.canvas.surface.width() as u16 / 8) as usize;
            debug_assert!(
                line.find('\n').is_none(),
                "History lines MUST NOT contain a newline character"
            );
            debug_assert!(
                line.len() <= real_max_len,
                "History lines not be longer that the max char count"
            );
            if self.scrollback_history.len() == self.history_limit as _ {
                self.scrollback_history.pop_front();
            }

            self.scrollback_history.push_back(line);

            if self.scrollback_history_offset != 0 {
                let history_len = self.scrollback_history.len();
                self.scrollback_history_offset = self
                    .scrollback_history_offset
                    .saturating_add(1)
                    .min(history_len as _);
            }
        }

        pub fn write(&mut self, text: impl AsRef<str>) {
            let mut text = text.as_ref();

            while let Some(position) = text.find('\n') {
                self.push_line(&text[..position], true);
                text = &text[position + 1..];
            }
            self.push_line(text, false);
        }

        fn max_line_count(&self) -> u32 {
            (self.canvas.surface.height() - 4) / (10 * self.scale as u32)
        }

        pub fn draw(&mut self) -> Result<()> {
            let max_line_count = self.max_line_count();
            self.canvas.render(Some(RGBA4::new()), |canvas| {
                let mut line_y = 2 + 8 * self.scale as i32; // leave a bit of a gap

                let max_history_lines = if self.scrollback_history_offset == 0 {
                    max_line_count - 1
                } else {
                    max_line_count
                };

                let history_print_offset = self
                    .scrollback_history
                    .len()
                    .saturating_sub(max_history_lines as usize)
                    .saturating_sub(self.scrollback_history_offset as usize);

                let history_lines_printed = self
                    .scrollback_history
                    .iter()
                    .skip(history_print_offset)
                    .take(max_history_lines as _)
                    .map(|s| {
                        canvas.draw_ascii_bitmap_text(
                            s,
                            self.text_color,
                            self.scale as u32,
                            2,
                            line_y,
                            crate::gpu::canvas::AlphaBlend::None,
                        );
                        line_y += 10 * self.scale as i32;
                    })
                    .count();

                if history_lines_printed < max_line_count as usize {
                    canvas.draw_ascii_bitmap_text(
                        &self.current_line,
                        self.text_color,
                        self.scale as u32,
                        2,
                        line_y,
                        crate::gpu::canvas::AlphaBlend::None,
                    );
                }

                Ok(())
            })
        }

        #[inline(always)]
        pub fn wait_vsync_event(&self, timeout: Option<i64>) -> Result<()> {
            self.canvas.wait_vsync_event(timeout)
        }
    }
}
