//! ARM support and utils

use core::arch::asm;

/// Represents a CPU register value (`W`, `X` or `R` value depending on the context/arch).
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct CpuRegister {
    /// The register value.
    pub reg: u64,
}

impl CpuRegister {
    /// Gets the [`CpuRegister`] as an `X` value.
    #[inline]
    pub const fn get_x(&self) -> u64 {
        self.reg
    }

    /// Sets the [`CpuRegister`] from an `X` value.
    ///
    /// # Arguments:
    ///
    /// * `x`: The value to set.
    #[inline]
    pub fn set_x(&mut self, x: u64) {
        self.reg = x;
    }

    /// Gets the [`CpuRegister`] as an `W` value.
    #[inline]
    pub const fn get_w(&self) -> u32 {
        self.reg as u32
    }

    /// Sets the [`CpuRegister`] from an `W` value.
    ///
    /// # Arguments:
    ///
    /// * `w`: The value to set.
    #[inline]
    pub fn set_w(&mut self, w: u32) {
        self.reg = w as u64;
    }

    /// Gets the [`CpuRegister`] as an `R` value.
    #[inline]
    pub const fn get_r(&self) -> u32 {
        self.reg as u32
    }

    /// Sets the [`CpuRegister`] from an `R` value.
    ///
    /// # Arguments:
    ///
    /// * `r`: The value to set.
    #[inline]
    pub fn set_r(&mut self, r: u32) {
        self.reg = r as u64;
    }
}

/// Represents a FPU register value (`V`, `D` or `S` value depending on the context/arch).
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct FpuRegister {
    /// The register value.
    pub reg: u128,
}

impl FpuRegister {
    /// Gets the [`FpuRegister`] as an `V` value.
    #[inline]
    pub const fn get_v(&self) -> u128 {
        self.reg
    }

    /// Sets the [`FpuRegister`] from an `V` value.
    ///
    /// # Arguments:
    ///
    /// * `v`: The value to set.
    #[inline]
    pub fn set_v(&mut self, v: u128) {
        self.reg = v;
    }

    /// Gets the [`FpuRegister`] as an `D` value.
    #[inline]
    pub const fn get_d(&self) -> f64 {
        self.reg as f64
    }

    /// Sets the [`FpuRegister`] from an `D` value.
    ///
    /// # Arguments:
    ///
    /// * `d`: The value to set.
    #[inline]
    pub fn set_d(&mut self, d: f64) {
        self.reg = d.to_bits() as u128;
    }

    /// Gets the [`FpuRegister`] as an `S` value.
    #[inline]
    pub const fn get_s(&self) -> f32 {
        f32::from_bits(self.reg as u32)
    }

    /// Sets the [`FpuRegister`] from an `S` value.
    ///
    /// # Arguments:
    ///
    /// * `s`: The value to set.
    #[inline]
    pub fn set_s(&mut self, s: f32) {
        self.reg = s.to_bits() as u128;
    }
}

define_bit_set! {
    /// Represents flags of different register kinds/groups.
    RegisterGroup (u32) {
        CpuGprs = bit!(0),
        CpuSprs = bit!(1),
        FpuGprs = bit!(2),
        FpuSprs = bit!(3)
    }
}

/// Represents a thread context usable with [`svc`][`crate::svc`]s.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ThreadContext {
    /// The general-purpose CPU registers.
    pub gpu_gprs: [CpuRegister; 29],
    /// The FP register.
    pub fp: u64,
    /// The LR register.
    pub lr: u64,
    /// The SP register.
    pub sp: u64,
    /// The PC register.
    pub pc: CpuRegister,
    /// The PSR value.
    pub psr: u32,
    /// The general-purpose FPU registers.
    pub fpu_gprs: [FpuRegister; 32],
    /// The FPCR value.
    pub fpcr: u32,
    /// The FPSR value.
    pub fpsr: u32,
    /// The TPIDR value.
    pub tpidr: u64,
}

/// Flushes (clean + invalidate) memory cache at a certain memory location.
///
/// The start and end address are rounded to cache line boundaries read from the `CTR_EL0` register.
///
/// # Arguments:
///
/// * `address`: Memory address.
/// * `size`: Memory size.
#[inline(always)]
#[deny(clippy::not_unsafe_ptr_arg_deref)]
pub fn cache_flush(address: *mut u8, size: usize) {
    unsafe extern "C" {
        fn __nx_arm_cache_flush(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_arm_cache_flush(address, size);
    }
}

/// Gets the system tick.
#[inline(always)]
pub fn get_system_tick() -> u64 {
    let system_tick: u64;
    unsafe {
        asm!(
            "mrs {}, cntpct_el0",
            out(reg) system_tick
        );
    }
    system_tick
}

/// Gets the system tick frequency.
#[inline(always)]
pub fn get_system_tick_frequency() -> u64 {
    let system_tick_freq: u64;
    unsafe {
        asm!(
            "mrs {}, cntfrq_el0",
            out(reg) system_tick_freq
        );
    }
    system_tick_freq
}

/// Converts ticks to nanoseconds.
///
/// # Arguments:
///
/// * `ticks`: Ticks to convert.
#[inline]
pub const fn ticks_to_nanoseconds(ticks: u64) -> u64 {
    (ticks * 625) / 12
}

/// Converts nanoseconds to ticks.
///
/// # Arguments:
///
/// * `ns`: Nanoseconds to convert.
#[inline]
pub const fn nanoseconds_to_ticks(ns: u64) -> u64 {
    (ns * 12) / 625
}
