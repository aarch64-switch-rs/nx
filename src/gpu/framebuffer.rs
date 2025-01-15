use super::MultiFence;


/// We are defining this here as we are initially just supporting R4G4B4A4 and R8G8B8A8 colour/pixel formats
pub enum ChannelBits{
    Four = 4,
    Eight = 8
}

/// RGBA (4444) Pixel/Color Representation
#[bitfield(u16)]
pub struct RGBA4 {  
    /// Red channel
    #[bits(4)]
    r: u8,
    /// Green channel
    #[bits(4)]
    g: u8,
    /// Blue channel
    #[bits(4)]
    b: u8,
    /// Alpha channel
    #[bits(4)]
    a: u8,
}

/// RGBA (8888) Pixel/Color Representation
/// 
/// We are using the bitfields crate here just for consistency with RGBA4444.
/// The bit range checks should be optimised out in release mode.
#[bitfield(u16)]
pub struct RGBA8 {  
    /// Red channel
    #[bits(8)]
    r: u8,
    /// Green channel
    #[bits(8)]
    g: u8,
    /// Blue channel
    #[bits(8)]
    b: u8,
    /// Alpha channel
    #[bits(8)]
    a: u8,
}

struct FrameBuffer<const CHANNEL_BITS: ChannelBits> {
    slot: i32,
    base_pointer: usize,
    buffer_size: usize,
    fences: MultiFence,
    surface_ref: & super::surface::Surface
}

impl FrameBuffer<ChannelBits::Eight> {
    pub fn clear(&mut self, color: RGBA8, blend: bool) {
        for y in 0..self.surface_ref.get_height() {
            for x in  0..self.surface_ref.get_width() {
                // Need to replace this with a version that doesn't write line by line, but GOB by GOB
                self.draw_single(x, y, color, blend);
            }
        }
    }

    pub fn draw_single(&mut self, x: u32, y: u32, color: RGBA8, blend: bool) {
        unsafe {
            let offset = self.surface_ref.stride() * y + x;
            let cur = (self.base_pointer as *mut RGBA8).add(offset as usize); // this is fine since and item in the linear_buf is pixel sized.
            let new_color = match blend {
                true => color.blend_with(core::ptr::read(cur)),
                false => color,
            };
            *cur = new_color;
        }
    }

    pub fn draw_rect(&mut self, x: u32, y: u32, width: u32, height: u32, color: RGBA8, blend: bool) {
        let s_width = self.surface_ref.get_width();
        let s_height = self.surface_ref.get_height();
        let x0 = x.clamp(0, s_width);
        let x1 = (x+width).clamp(0, s_width);
        let y0 = y.clamp(0, s_height);
        let y1 = (y+height).clamp(0, s_height);
        for y in y0..y1 {
            for x in x0..x1 {
                self.draw_single(x, y, color, blend);
            }
        }
    }
}