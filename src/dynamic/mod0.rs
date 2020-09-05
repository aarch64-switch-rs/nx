#[derive(Copy, Clone)]
#[repr(C)]
pub struct Header {
    pub magic: u32,
    pub dynamic: u32,
    pub bss_start: u32,
    pub bss_end: u32,
    pub eh_frame_hdr_start: u32,
    pub eh_frame_hdr_end: u32,
    pub module_object: u32,
}

pub const MAGIC: u32 = 0x30444F4D;