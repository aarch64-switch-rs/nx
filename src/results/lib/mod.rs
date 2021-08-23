pub const RESULT_MODULE: u32 = 430;

pub const RESULT_SUBMODULE: u32 = 0;

result_define_subgroup!(RESULT_MODULE, RESULT_SUBMODULE => {
    NotImplemented: 1,
    NotSupported: 2,
    NotInitialized: 3
});

// Note: result submodules below are ordered by their submodule values

pub mod dynamic;

pub mod elf;

pub mod util;

pub mod assert;

pub mod gpu;

pub mod ipc;

pub mod fs;

pub mod input;

pub mod thread;