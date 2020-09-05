pub const RESULT_SUBMODULE: u32 = 300;

result_define_subgroup!(super::RESULT_MODULE, RESULT_SUBMODULE => {
    InvalidPointer: 1,
    InvalidSize: 2,
    InvalidConversion: 3
});