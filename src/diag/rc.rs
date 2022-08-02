//! Diagnostics-related result definitions

use crate::rc;

pub const RESULT_SUBMODULE: u32 = 400;

result_define_subgroup!(rc::RESULT_MODULE, RESULT_SUBMODULE => {
    AssertionFailed: 1
});