use crate::result::*;
use core::ptr;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(i64)]
pub enum Tag {
    #[default]
    Invalid = 0,
    Needed = 1,
    PltRelSize = 2,
    Hash = 4,
    StrTab = 5,
    SymTab = 6,
    RelaOffset = 7,
    RelaSize = 8,
    RelaEntrySize = 9,
    SymEnt = 11,
    RelOffset = 17,
    RelSize = 18,
    RelEntrySize = 19,
    PltRel = 20,
    JmpRel = 23,
    InitArray = 25,
    FiniArray = 26,
    InitArraySize = 27,
    FiniArraySize = 28,
    RelaCount = 0x6FFFFFF9,
    RelCount = 0x6FFFFFFA
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum RelocationType {
    AArch64Abs64 = 257,
    AArch64GlobDat = 1025,
    AArch64JumpSlot = 1026,
    AArch64Relative = 1027
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Dyn {
    pub tag: Tag,
    pub val_ptr: usize
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InfoSymbol {
    pub relocation_type: RelocationType,
    pub symbol: u32
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union Info {
    pub value: u64,
    pub symbol: InfoSymbol
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rel {
    pub offset: usize,
    pub info: Info
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rela {
    pub offset: usize,
    pub info: Info,
    pub addend: i64
}

pub fn relocate_with_dyn(base_address: *const u8, start_dyn: *const Dyn) -> Result<()> {
    unsafe {
        let mut rel_offset_v: Option<usize> = None;
        let mut rel_entry_size_v: Option<usize> = None;
        let mut rel_count_v: Option<usize> = None;
        let mut rela_offset_v: Option<usize> = None;
        let mut rela_entry_size_v: Option<usize> = None;
        let mut rela_count_v: Option<usize> = None;

        let mut cur_dyn = start_dyn;
        loop {
            match (*cur_dyn).tag {
                Tag::Invalid => break,
                Tag::RelOffset => rel_offset_v = Some((*cur_dyn).val_ptr),
                Tag::RelEntrySize => rel_entry_size_v = Some((*cur_dyn).val_ptr),
                Tag::RelCount => rel_count_v = Some((*cur_dyn).val_ptr),
                Tag::RelaOffset => rela_offset_v = Some((*cur_dyn).val_ptr),
                Tag::RelaEntrySize => rela_entry_size_v = Some((*cur_dyn).val_ptr),
                Tag::RelaCount => rela_count_v = Some((*cur_dyn).val_ptr),
                _ => {}
            };

            cur_dyn = cur_dyn.add(1);
        }

        if let (Some(rel_offset), Some(rel_count)) = (rel_offset_v, rel_count_v) {
            let rel_entry_size = rel_entry_size_v.unwrap_or(core::mem::size_of::<Rel>());
            let rel_base = base_address.offset(rel_offset as isize);

            for i in 0..rel_count {
                let rel = rel_base.offset((i * rel_entry_size) as isize) as *const Rel;
                match (*rel).info.symbol.relocation_type {
                    RelocationType::AArch64Relative => {
                        let relocation_offset = base_address.offset((*rel).offset as isize) as *mut *const u8;
                        *relocation_offset = base_address;
                    },
                    _ => {}
                }
            }
        }
        
        if let (Some(rela_offset), Some(rela_count)) = (rela_offset_v, rela_count_v) {
            let rela_entry_size = rela_entry_size_v.unwrap_or(core::mem::size_of::<Rela>());
            let rela_base = base_address.offset(rela_offset as isize);

            for i in 0..rela_count {
                let rela = rela_base.offset((i * rela_entry_size) as isize) as *const Rela;
                match (*rela).info.symbol.relocation_type {
                    RelocationType::AArch64Relative => {
                        let relocation_offset = base_address.offset((*rela).offset as isize) as *mut *const u8;
                        *relocation_offset = base_address.offset((*rela).addend as isize);
                    },
                    _ => {}
                }
            }
        }
    }
    Ok(())
}