pub const RESULT_SUBMODULE: u32 = 900;

result_define_subgroup!(super::RESULT_MODULE, RESULT_SUBMODULE => {
    InvalidStack: 1,
    InvalidState: 2
});