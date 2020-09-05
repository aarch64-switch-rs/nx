pub fn cache_flush(address: *mut u8, size: usize) {
    extern "C" {
        fn __nx_arm_cache_flush(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_arm_cache_flush(address, size);
    }
}

pub fn get_system_tick() -> u64 {
    unsafe {
        let tick: u64;
        llvm_asm!("mrs x0, cntpct_el0" : "={x0}"(tick) ::: "volatile");

        tick
    }
}

pub fn get_system_tick_frequency() -> u64 {
    unsafe {
        let tick_freq: u64;
        llvm_asm!("mrs x0, cntfrq_el0" : "={x0}"(tick_freq) ::: "volatile");

        tick_freq
    }
}

pub const fn ticks_to_nanoseconds(ticks: u64) -> u64 {
    (ticks * 625) / 12
}

pub const fn nanoseconds_to_ticks(nanoseconds: u64) -> u64 {
    (nanoseconds * 12) / 625
}