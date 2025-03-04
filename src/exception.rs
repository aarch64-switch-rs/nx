use crate::result::ResultBase;
use crate::svc;
/*
/// Armv8 CPU register.
union CpuRegister {
    /// 64-bit AArch64 register view.
    x: u64,
    /// 32-bit AArch64 register view.
    w: u32,
    /// AArch32 register view.
    r: u32
}

/// Armv8 NEON register.
union FpuRegister {
    /// 128-bit vector view.
    v: u128,
    /// 64-bit double-precision view.
    d: f64,
    /// 32-bit single-precision view.
    s: f32
}

struct ExceptionContext {
    error_desc: svc::ExceptionType,
    _pad: [u32;3],

    /// General Purpose Registers 0..28
    /// Note: also contains AArch32 registers.
    cpu_gprs: [CpuRegister; 29],
    frame_pointer: CpuRegister,
    link_regiser: CpuRegister,
    stack_pointer: CpuRegister,
    program_counter: CpuRegister,

    _padding: u64,

    fpu_gprs: [FpuRegister; 32],

    /// pstate & 0xFF0FFE20
    pstate: u32,
    afsr0: u32,
    afsr1: u32,
    esr: u32,

    ///< Fault Address Register.
    far: CpuRegister
}

#[no_mangle]
#[linkage = "weak"]
unsafe extern "C" fn __nx_exception_handler(_ctx: *mut ExceptionContext) -> ! {
    // TODO: user exception handler?
    svc::return_from_exception(svc::rc::ResultNotHandled::make());
}
*/

#[no_mangle]
pub(crate) unsafe extern "C" fn __nx_exception_dispatch(
    _reason: svc::ExceptionType,
    _stack_top: *mut u8,
) -> ! {
    svc::return_from_exception(svc::rc::ResultNotHandled::make());
}
