use crate::rc;

pub const RESULT_SUBMODULE: u32 = 200;

result_define_subgroup!(rc::RESULT_MODULE, RESULT_SUBMODULE => {
    RelaSizeMismatch: 1,
    InvalidModuleMagic: 2,
    DuplicatedDtEntry: 3,
    MissingDtEntry: 4
});