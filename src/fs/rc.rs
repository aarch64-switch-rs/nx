//! FS-related result definitions

use crate::rc;

pub const RESULT_SUBMODULE: u32 = 700;

result_define_subgroup!(rc::RESULT_MODULE, RESULT_SUBMODULE => {
    DeviceNotFound: 1,
    InvalidPath: 2,
    NotInSameFileSystem: 3
});