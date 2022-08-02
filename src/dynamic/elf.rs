//! ELF (aarch64) support and utils

use crate::result::*;
use core::ptr;

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
    RelaCount = 0x6FFFFFF9
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
    pub val_ptr: u64,
}

impl Dyn {
    /// Finds a value corresponding to a certain [`Tag`]
    /// 
    /// This takes this [`Dyn`] as the start of contiguous [`Dyn`]s and iterates over them until the final one is reached
    /// 
    /// # Arguments
    /// 
    /// * `tag`: The [`Tag`] to use
    pub fn find_value(&self, tag: Tag) -> Result<u64> {
        unsafe {
            let mut found: *const u64 = ptr::null();
            let mut self_ptr = self as *const Self;
        
            while (*self_ptr).tag != Tag::Invalid {
                if (*self_ptr).tag == tag {
                    result_return_unless!(found.is_null(), rc::ResultDuplicatedDtEntry);
                    found = &(*self_ptr).val_ptr;
                }
                self_ptr = self_ptr.offset(1);
            }
            result_return_if!(found.is_null(), rc::ResultMissingDtEntry);

            Ok(*found)
        }
    }
}

/// Represents an ELF info symbol
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InfoSymbol {
    /// The relocation type
    pub relocation_type: RelocationType,
    /// The symbol value
    pub symbol: u32,
}

/// Represents an info value
#[derive(Copy, Clone)]
#[repr(C)]
pub union Info {
    /// The value
    pub value: u64,
    /// The symbol
    pub symbol: InfoSymbol,
}

/// Represents a rela type
#[derive(Copy, Clone)]
#[repr(C)]
pub struct Rela {
    /// The offset
    pub offset: u64,
    /// The info
    pub info: Info,
    /// The addend value
    pub addend: i64,
}