use core::arch::asm;
use crate::svc;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct CpuRegister {
    pub reg: u64
}

impl CpuRegister {
    pub const fn get_x(&self) -> u64 {
        self.reg
    }

    pub fn set_x(&mut self, x: u64) {
        self.reg = x;
    }

    pub const fn get_w(&self) -> u32 {
        self.reg as u32
    }

    pub fn set_w(&mut self, w: u32) {
        self.reg = w as u64;
    }

    pub const fn get_r(&self) -> u32 {
        self.reg as u32
    }

    pub fn set_r(&mut self, r: u32) {
        self.reg = r as u64;
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct FpuRegister {
    pub reg: u128
}

impl FpuRegister {
    pub const fn get_v(&self) -> u128 {
        self.reg
    }

    pub fn set_v(&mut self, v: u128) {
        self.reg = v;
    }

    pub const fn get_d(&self) -> f64 {
        self.reg as f64
    }

    pub fn set_d(&mut self, d: f64) {
        self.reg = d as u128;
    }

    pub const fn get_s(&self) -> f32 {
        self.reg as f32
    }

    pub fn set_s(&mut self, s: f32) {
        self.reg = s as u128;
    }
}

bit_enum! {
    RegisterGroup (u32) {
        CpuGprs = bit!(0),
        CpuSprs = bit!(1),
        FpuGprs = bit!(2),
        FpuSprs = bit!(3)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ThreadContext {
    pub gpu_gprs: [CpuRegister; 29],
    pub fp: u64,
    pub lr: u64,
    pub sp: u64,
    pub pc: CpuRegister,
    pub psr: u32,
    pub fpu_gprs: [FpuRegister; 32],
    pub fpcr: u32,
    pub fpsr: u32,
    pub tpidr: u64
}

#[inline(always)]
pub fn cache_flush(address: *mut u8, size: usize) {
    extern "C" {
        fn __nx_arm_cache_flush(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_arm_cache_flush(address, size);
    }
}

#[inline(always)]
pub fn get_system_tick() -> u64 {
    let system_tick: u64;

    #[cfg(target_pointer_width = "64")]
    unsafe {
        asm!(
            "mrs {}, cntpct_el0",
            out(reg) system_tick
        );
    }

    #[cfg(target_pointer_width = "32")]
    {
        system_tick = svc::get_system_tick();
    }

    system_tick
}

#[inline(always)]
#[cfg(target_pointer_width = "64")]
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

pub const fn ticks_to_nanoseconds(ticks: u64) -> u64 {
    (ticks * 625) / 12
}

pub const fn nanoseconds_to_ticks(nanoseconds: u64) -> u64 {
    (nanoseconds * 12) / 625
}