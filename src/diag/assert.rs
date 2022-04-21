use crate::result::*;
use crate::svc;
use crate::mem::alloc;
use crate::rrt0;
use crate::ipc::sf;
use crate::service;
use crate::service::fatal;
use crate::service::fatal::IService;
use core::mem;

pub mod rc;

bit_enum! {
    AssertLevel(u32) {
        NeedsHeapAllocation = bit!(31),

        FatalThrow = 1 | Self::NeedsHeapAllocation().get(),
        Panic = 2 | Self::NeedsHeapAllocation().get(),
        ProcessExit = 3,
        SvcBreak = 4
    }
}

impl AssertLevel {
    // Assert types work like different levels - when the desired level can't be processed (for instance, a panic due to errors allocating memory since it cannot allocate anymore) the next one is tried, and so on
    // The last level, breaking via SVC, should always work properly
    const ASSERT_LEVEL_ORDER: [AssertLevel; 4] = [AssertLevel::FatalThrow(), AssertLevel::Panic(), AssertLevel::ProcessExit(), AssertLevel::SvcBreak()];

    #[inline]
    pub fn get_next_level(self) -> Option<Self> {
        for i in 0..Self::ASSERT_LEVEL_ORDER.len() {
            if Self::ASSERT_LEVEL_ORDER[i] == self {
                let next_i = i + 1;
                if next_i < Self::ASSERT_LEVEL_ORDER.len() {
                    return Some(Self::ASSERT_LEVEL_ORDER[next_i]);
                }
            }
        }

        None
    }
}

#[inline]
fn do_assert(level: AssertLevel, rc: ResultCode) -> bool {
    if level.contains(AssertLevel::NeedsHeapAllocation()) && !alloc::is_enabled() {
        // Prevent assertion methods which will allocate from running if we cannot allocate, to avoid infinite alloc-error recursions
        false
    }
    else {
        if level == AssertLevel::FatalThrow() {
            match service::new_service_object::<fatal::Service>() {
                Ok(fatal) => {
                    let _ = fatal.get().throw_fatal_with_policy(rc, fatal::FatalPolicy::ErrorScreen, sf::ProcessId::new());
                },
                _ => {}
            };
        }
        else if level == AssertLevel::Panic() {
            let res: Result<()> = Err(rc);
            res.unwrap();
        }
        else if level == AssertLevel::ProcessExit() {
            rrt0::exit(rc);
        }
        else if level == AssertLevel::SvcBreak() {
            svc::break_(svc::BreakReason::Panic, &rc as *const _ as *const u8, mem::size_of::<ResultCode>());
        }
        
        // Note: this shouldn't be reached if the assertions succeed ;)
        true
    }
}

#[inline]
pub fn assert(desired_level: AssertLevel, rc: ResultCode) {
    if rc.is_failure() {
        let mut current_level = desired_level;

        while !do_assert(current_level, rc) {
            // Assert method failed, try with the next level
            if let Some(next_level) = current_level.get_next_level() {
                current_level = next_level;
            }
            else {
                // This should never happen, since the last level is guaranteed to work
                loop {}
            }
        }
    }
}