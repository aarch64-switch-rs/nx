use core::{num::NonZeroU16, u16};

use crate::{
    gpu::canvas::{Canvas, CanvasManager, RGBA4},
    result::Result,
};

use alloc::{collections::vec_deque::VecDeque, string::String};

/*pub struct RandomAccessConsole {
    //TODO: Make a full featured console that can have a cursor, colours, styles, etc
}*/

/// This console creates a full-screen layer that will just scroll through provided strings
pub struct ScrollbackConsole {
    pub canvas: CanvasManager<RGBA4>,
    pub text_color: RGBA4,
    pub history_limit: u16,
    pub line_max_chars: u16,
    pub line_wrap: bool,
    pub scrollback_history: alloc::collections::VecDeque<String>,
    pub scrollback_history_offset: u16,
    pub current_line: String,
}

impl ScrollbackConsole {
    #[inline]
    pub fn new(
        canvas: CanvasManager<RGBA4>,
        history_limit: u16,
        line_max_chars: NonZeroU16,
        line_wrap: bool,
        text_color: Option<RGBA4>,
    ) -> Result<Self> {
        Ok(Self {
            history_limit,
            text_color: text_color.unwrap_or(RGBA4::from_bits(u16::MAX)),
            line_wrap,
            line_max_chars: line_max_chars
                .get()
                .min(canvas.surface.width() as u16 / 8),
            scrollback_history: VecDeque::with_capacity(history_limit as _),
            current_line: String::new(),
            canvas,
            scrollback_history_offset: 0,
        })
    }

    #[inline(always)]
    pub fn scroll_up(&mut self) {
        self.scrollback_history_offset = self.scrollback_history_offset.saturating_add(1);
    }

    #[inline(always)]
    pub fn scroll_down(&mut self) {
        self.scrollback_history_offset = self.scrollback_history_offset.saturating_sub(1);
    }

    fn push_line(&mut self, text: &str, commit: bool) {
        self.current_line.push_str(text);

        let real_max_len = self
            .line_max_chars
            .min(self.canvas.surface.width() as u16 / 8) as usize;

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
        let real_max_len = self
            .line_max_chars
            .min(self.canvas.surface.width() as u16 / 8) as usize;
        debug_assert!(
            line.find('\n').is_none(),
            "History lines MUST NOT contain a newline character"
        );
        debug_assert!(
            line.len()
                <= real_max_len,
            "History lines not be longer that the max char count"
        );
        if self.scrollback_history.len() == self.history_limit as _ {
            self.scrollback_history.pop_front();
        }

        self.scrollback_history.push_back(line);

        if self.scrollback_history_offset != 0 {
            self.scrollback_history_offset += 1;
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

    pub fn draw(&mut self) -> Result<()> {
        self.canvas.render(Some(RGBA4::new()), |canvas| {
            let mut line_y = 10;
            let max_line_count = canvas.height() / 10;

            let history_print_offset = self
                .scrollback_history
                .len()
                .saturating_sub(max_line_count as usize - 1)
                .saturating_sub(self.scrollback_history_offset as usize);

            let history_lines_printed = self
                .scrollback_history
                .iter()
                .skip(history_print_offset)
                .take(max_line_count as _)
                .map(|s| {
                    canvas.draw_ascii_bitmap_text(
                        s,
                        self.text_color,
                        1,
                        2,
                        line_y,
                        crate::gpu::canvas::AlphaBlend::None,
                    );
                    line_y += 10;
                })
                .count();

            if history_lines_printed < max_line_count as usize {
                canvas.draw_ascii_bitmap_text(
                    &self.current_line,
                    self.text_color,
                    1,
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
