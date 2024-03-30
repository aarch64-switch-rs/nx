//! Aborting implementation

use crate::result::*;
use crate::svc;
use crate::mem::alloc;
use crate::rrt0;
use core::mem;

#[cfg(feature = "services")]
use crate::ipc::sf;

#[cfg(feature = "services")]
use crate::service;

#[cfg(feature = "services")]
use crate::service::fatal;

#[cfg(feature = "services")]
use crate::service::fatal::IService;

define_bit_enum! {
    /// Represents a system to abort, plus optional flags they have
    AbortLevel (u32) {
        NeedsHeapAllocation = bit!(31),

        FatalThrow = bit!(0) | Self::NeedsHeapAllocation().get(),
        Panic = bit!(2) | Self::NeedsHeapAllocation().get(),
        ProcessExit = bit!(3),
        SvcBreak = bit!(4)
    }
}

impl AbortLevel {
    // When the desired level can't be processed (for instance, a panic due to errors allocating memory since it cannot allocate anymore) the next one is attempted, and so on
    // The last level, breaking via SVC, is guaranteed to work properly
    const LEVEL_ORDER: &'static [AbortLevel] = &[AbortLevel::FatalThrow(), AbortLevel::Panic(), AbortLevel::ProcessExit(), AbortLevel::SvcBreak()];

    /// Gets the next [`AbortLevel`]
    /// 
    /// The abort level order is the following: `FatalThrow`, `Panic`, `ProcessExit`, `SvcBreak`
    #[inline]
    pub fn get_next_level(self) -> Option<Self> {
        for i in 0..Self::LEVEL_ORDER.len() {
            if Self::LEVEL_ORDER[i] == self {
                let next_i = i + 1;
                if next_i < Self::LEVEL_ORDER.len() {
                    return Some(Self::LEVEL_ORDER[next_i]);
                }
            }
        }

        None
    }
}

fn do_abort(level: AbortLevel, rc: ResultCode) {
    if level.contains(AbortLevel::NeedsHeapAllocation()) && !alloc::is_enabled() {
        // Prevent abort methods which will allocate from running if we cannot allocate, to avoid infinite alloc-error recursions
        return;
    }

    if level == AbortLevel::FatalThrow() {
        #[cfg(feature = "services")]
        {
            match service::new_service_object::<fatal::Service>() {
                Ok(fatal) => {
                    let _ = fatal.get().throw_fatal_with_policy(rc, fatal::FatalPolicy::ErrorScreen, sf::ProcessId::new());
                },
                _ => {}
            };
        }
    }
    else if level == AbortLevel::Panic() {
        let res: Result<()> = Err(rc);
        res.unwrap();
    }
    else if level == AbortLevel::ProcessExit() {
        rrt0::exit(rc);
    }
    else if level == AbortLevel::SvcBreak() {
        svc::break_(svc::BreakReason::Panic, &rc as *const _ as *const u8, mem::size_of_val(&rc));
    }
    
    // Note: this won't be reached if the abort succeeds
}

/// Attempts to abort at the specified [`AbortLevel`]
/// 
/// Note that a certain [`AbortLevel`] may not work/be available (heap allocation is not available and that level requires allocations, etc.)
/// 
/// Therefore, this function will try with the next levels in order if the desired one fails (see [`get_next_level`][`AbortLevel::get_next_level`])
/// 
/// Also note that a success [`ResultCode`] may result in UB for certain [`AbortLevel`]s
/// 
/// This function never returns since the last possible [`AbortLevel`] is guaranteed to succeed
/// 
/// # Arguments
/// 
/// * `desired_level`: Desired [`AbortLevel`]
/// * `rc`: [`ResultCode`] to abort with
pub fn abort(desired_level: AbortLevel, rc: ResultCode) -> ! {
    let mut current_level = desired_level;

    loop {
        do_abort(current_level, rc);

        if let Some(next_level) = current_level.get_next_level() {
            current_level = next_level;
        }
        else {
            // This should never happen, since the last level is guaranteed to work
            unreachable!();
        }
    }
}