pub const RESULT_MODULE: u32 = 430;

pub const RESULT_SUBMODULE: u32 = 0;

result_define_subgroup!(RESULT_MODULE, RESULT_SUBMODULE => {
    NotImplemented: 1,
    NotSupported: 2,
    NotInitialized: 3,
    Panicked: 4
});

/*

- Submodule list for our own results:

0: library (misc)
100: dynamic
200: dynamic/elf
300: util
400: diag
500: gpu
600: ipc
700: fs
800: input
900: thread
1000: mem
1100: gpu/binder
1200: gpu/parcel
1300: ipc/server
1400: crypto

*/