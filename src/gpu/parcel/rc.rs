use crate::rc;

pub const RESULT_SUBMODULE: u32 = 1200;

result_define_subgroup!(rc::RESULT_MODULE, RESULT_SUBMODULE => {
    NotEnoughReadSpace: 1,
    NotEnoughWriteSpace: 2,
    FdsNotSupported: 3,
    ReadSizeMismatch: 4
});