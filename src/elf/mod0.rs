use crate::result::*;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ModuleStart {
    pub reserved: u32,
    pub magic_offset: u32,
}

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

pub const MAGIC: u32 = u32::from_le_bytes(*b"MOD0");

#[inline]
pub fn find_start_dyn_address(base_address: *const u8) -> Result<*const u8> {
    unsafe {
        let module_start = base_address as *const ModuleStart;
        let mod_offset = (*module_start).magic_offset as isize;
        let module = base_address.offset(mod_offset) as *const Header;
        result_return_unless!((*module).magic == super::mod0::MAGIC, super::rc::ResultInvalidModuleMagic);

        let dyn_offset = mod_offset + (*module).dynamic as isize;
        let start_dyn = base_address.offset(dyn_offset);
        Ok(start_dyn)
    }
}