//! MOD0 format utils

/// Represents the `MOD0` header structure
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Header {
    /// The magic, whose expected value is [`MAGIC`][`Header::MAGIC`]
    pub magic: u32,
    /// The dynamic offset
    pub dynamic: u32,
    /// The BSS start offset
    pub bss_start: u32,
    /// The BSS end offset
    pub bss_end: u32,
    /// The eh_frame_hdr start offset
    pub eh_frame_hdr_start: u32,
    /// The eh_frame_hdr end offset
    pub eh_frame_hdr_end: u32,
    /// The offset to runtime-generated module object
    pub module_object: u32,
}

impl Header {
    /// The header magic value (`MOD0`)
    pub const MAGIC: u32 = u32::from_le_bytes(*b"MOD0");

    /// Gets whether the header is valid
    /// 
    /// Essentially checks that the magic has the expected [`MAGIC`][`Header::MAGIC`] value
    #[inline]
    pub const fn is_valid(&self) -> bool {
        self.magic == Self::MAGIC
    }
}