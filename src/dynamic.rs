//! Dynamic (ELF/relocation) support

use crate::result::*;

pub mod rc;

/// Represents the start layout of a base process address
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ModuleStart {
    pub reserved: u32,
    pub magic_offset: u32,
}

/// Relocates a base address with its corresponding [`Dyn`][`elf::Dyn`] reference
/// 
/// # Arguments
/// 
/// * `base_address`: Base address to relocate
/// * `dynamic`: It's [`Dyn`][`elf::Dyn`] reference
pub fn relocate_with_dyn(base_address: *const u8, dynamic: *const elf::Dyn) -> Result<()> {
    unsafe {
        let rela_offset = (*dynamic).find_value(elf::Tag::RelaOffset)?;
        let rela_size = (*dynamic).find_value(elf::Tag::RelaSize)?;
        let rela_entry_size = (*dynamic).find_value(elf::Tag::RelaEntrySize)?;
        let rela_count = (*dynamic).find_value(elf::Tag::RelaCount)?;
        result_return_unless!(rela_size == rela_entry_size * rela_count, rc::ResultRelaSizeMismatch);

        let rela_base = base_address.offset(rela_offset as isize) as *const elf::Rela;
        for i in 0..rela_count {
            let rela = rela_base.offset(i as isize);
            match (*rela).info.symbol.relocation_type {
                elf::RelocationType::AArch64Relative => {
                    if (*rela).info.symbol.symbol == 0 {
                        let relocation_offset = base_address.offset((*rela).offset as isize) as *mut *const u8;
                        *relocation_offset = base_address.offset((*rela).addend as isize);
                    }
                },
                _ => {}
            }
        }
    }
    Ok(())
}

/// Relocates a base address
/// 
/// # Arguments
/// 
/// * `base_address`: Base address to relocate
pub fn relocate(base_address: *const u8) -> Result<()> {
    unsafe {
        let module_start = base_address as *const ModuleStart;
        let mod_offset = (*module_start).magic_offset as isize;
        let module = base_address.offset(mod_offset) as *const mod0::Header;
        result_return_unless!((*module).is_valid(), rc::ResultInvalidModuleMagic);

        let dyn_offset = mod_offset + (*module).dynamic as isize;
        let dynamic = base_address.offset(dyn_offset) as *const elf::Dyn;
        relocate_with_dyn(base_address, dynamic)
    }
}

pub mod elf;

pub mod mod0;