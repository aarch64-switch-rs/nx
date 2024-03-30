//! MOD0 format utils

use crate::result::*;

/// Represents the `MOD0` start layout
/// 
/// These are the contents prececing the actual header
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ModuleStart {
    /// Reserved
    pub reserved: u32,
    /// The magic offset
    pub magic_offset: u32,
}

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
}

/// Finds the [`Dyn`][`super::Dyn`] reference from a base code address
/// 
/// # Arguments
/// 
/// * `base_address`: The base address
#[inline]
pub fn find_start_dyn_address(base_address: *const u8) -> Result<*const u8> {
    unsafe {
        let module_start = base_address as *const ModuleStart;
        let mod_offset = (*module_start).magic_offset as isize;
        let module = base_address.offset(mod_offset) as *const Header;
        result_return_unless!((*module).magic == Header::MAGIC, super::rc::ResultInvalidModuleMagic);

        let dyn_offset = mod_offset + (*module).dynamic as isize;
        let start_dyn = base_address.offset(dyn_offset);
        Ok(start_dyn)
    }
}