use core::arch::asm;

use crate::thread;

pub mod mutex;
pub mod rwlock;

#[inline(always)]
fn get_current_thread_handle() -> u32 {
    unsafe {(*thread::get_thread_local_region()).nx_thread_vars.handle}
}

#[inline(always)]
unsafe fn load_exclusive(ptr: *const u32) -> u32 {
    let value: u32;
    unsafe {
        asm!(
            "ldaxr {0:w}, [{1:x}]",
            out(reg) value,
            in(reg) ptr
        );
    }
    value
}

#[inline(always)]
unsafe fn store_exclusive(ptr: *const u32, value: u32) -> bool {
    let res: i32;
    unsafe {
        asm!(
            "stlxr {0:w}, {1:w}, [{2:x}]",
            out(reg) res,
            in(reg) value,
            in(reg) ptr
        );
    }
    res == 0 // zero on success, else 1. See https://developer.arm.com/documentation/ddi0596/2020-12/Base-Instructions/STLXR--Store-Release-Exclusive-Register-
}

#[inline(always)]
fn clear_exclusive() {
    unsafe {
        asm!("clrex");
    }
}