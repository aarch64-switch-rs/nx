pub const RESULT_SUBMODULE: u32 = 100;

result_define_subgroup!(super::RESULT_MODULE, RESULT_SUBMODULE => {
    RelaSizeMismatch: 1,
    InvalidModuleMagic: 2
});