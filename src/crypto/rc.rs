use crate::rc;

pub const RESULT_SUBMODULE: u32 = 1400;

result_define_subgroup!(rc::RESULT_MODULE, RESULT_SUBMODULE => {
    InvalidSize: 1
});