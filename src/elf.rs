//! ELF (aarch64) support and utils

use core::{ptr::null_mut, sync::atomic::{AtomicPtr, AtomicUsize}};

use unwinding::custom_eh_frame_finder::{FrameInfo, FrameInfoKind};

use crate::result::*;

pub mod rc;

/// Represents ELF tags
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

/// Represents ELF relocation types
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum RelocationType {
    AArch64Abs64 = 257,
    AArch64GlobDat = 1025,
    AArch64JumpSlot = 1026,
    AArch64Relative = 1027
}

/// Represents an ELF dynamic entry
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Dyn {
    /// The entry tag
    pub tag: Tag,
    /// The entry value
    pub val_ptr: usize
}

/// Represents an ELF info symbol
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InfoSymbol {
    /// The relocation type
    pub relocation_type: RelocationType,
    /// The symbol value
    pub symbol: u32
}

/// Represents an info value
#[derive(Copy, Clone)]
#[repr(C)]
pub union Info {
    /// The value
    pub value: u64,
    /// The symbol
    pub symbol: InfoSymbol
}

/// Represents a rel type
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rel {
    /// The offset
    pub offset: usize,
    /// The info
    pub info: Info
}

/// Represents a rela type
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rela {
    /// The offset
    pub offset: usize,
    /// The info
    pub info: Info,
    /// The addend value
    pub addend: i64
}

/// Relocates a base address with its corresponding [`Dyn`] reference
/// 
/// # Arguments
/// 
/// * `base_address`: The base address to relocate
/// * `start_dyn`: The [`Dyn`] reference
pub unsafe fn relocate_with_dyn(base_address: *mut u8, start_dyn: *const Dyn) -> Result<()> {
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
            let rel_base = base_address.add(rel_offset);

            for i in 0..rel_count {
                let rel = rel_base.add(i * rel_entry_size) as *const Rel;
                if (*rel).info.symbol.relocation_type  == RelocationType::AArch64Relative {
                    let relocation_offset = base_address.add((*rel).offset) as *mut *const u8;
                    *relocation_offset = base_address;
                }
            }
        }
        
        if let (Some(rela_offset), Some(rela_count)) = (rela_offset_v, rela_count_v) {
            let rela_entry_size = rela_entry_size_v.unwrap_or(core::mem::size_of::<Rela>());
            let rela_base = base_address.add(rela_offset);

            for i in 0..rela_count {
                let rela = rela_base.add(i * rela_entry_size) as *const Rela;
                if (*rela).info.symbol.relocation_type == RelocationType::AArch64Relative {
                        let relocation_offset = base_address.add((*rela).offset) as *mut *const u8;
                        *relocation_offset = base_address.offset((*rela).addend as isize);
                    }
            }
        }
    }
    Ok(())
}

/// A stuct containing a pointer sized int, representing a pointer to the start of the eh_frame_hdr elf section.
/// This is obviously not a great option to use with Rust's upcoming strict/exposed providence APIs, but works fine here as
/// the Switch has a single address space and the memory will have a static lifetime that is longer than the currently running code.
#[derive(Debug)]
pub struct EhFrameHdrPtr(AtomicPtr<u8>);

impl EhFrameHdrPtr {
    pub const fn new() -> Self {
        Self(AtomicPtr::new(null_mut()))
    }
    
    pub fn set(&self, val: *mut u8) {
        self.0.store(val, core::sync::atomic::Ordering::SeqCst);
    } 
}

unsafe impl Sync for EhFrameHdrPtr {}

unsafe impl unwinding::custom_eh_frame_finder::EhFrameFinder for EhFrameHdrPtr{
    fn find(&self, _pc: usize) -> Option<unwinding::custom_eh_frame_finder::FrameInfo> {
        match self.0.load(core::sync::atomic::Ordering::SeqCst) {
            ptr if !ptr.is_null() => {
                Some(
                    FrameInfo {
                        text_base: None,
                        kind: FrameInfoKind::EhFrameHdr(ptr as usize)
                    }
                )
            },
            _ => None
        }
    }
}

pub mod mod0;