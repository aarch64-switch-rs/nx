use crate::ipc::sf::sm::rc;

pub const RESULT_SUBMODULE: u32 = 1000;

result_define_subgroup!(rc::RESULT_MODULE, RESULT_SUBMODULE => {
    ShouldForwardToSession: 0
});