pub const RESULT_SUBMODULE: u32 = 200;

result_define_subgroup!(super::RESULT_MODULE, RESULT_SUBMODULE => {
    DuplicatedDtEntry: 1,
    MissingDtEntry: 2
});