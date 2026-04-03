pub mod rc;

use crate::{ipc::sf::EnumAsPrimitiveType, util::{ArrayString, Uuid}};
use nx_derive::{Request, Response};

use crate::ipc::sf::{
    self, AppletResourceUserId, CopyHandle, InAutoSelectBuffer, InMapAliasBuffer,
    OutAutoSelectBuffer, OutMapAliasBuffer,
};

pub type PosixTime = i64;
pub type TimeZoneLocationName = ArrayString<0x24>;

/// A monotonic time source
#[derive(Request, Response, Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct TimeZoneRule {
    _rule: [u8;0x4000]
}

impl Debug for TimeZoneRule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeZoneRule")
            .finish_non_exhaustive()
    }
}

/// A monotonic time source
#[derive(Request, Response, Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct SteadyClockTimePoint {
    /// Unix timestamp in seconds
    pub time_point: i64,
    /// The UUID of the time source
    pub source: Uuid,
}

/// A standard time source
/// 
/// This contains the monotonic time source in the `steady_time_point` field, but subsequent calls to retrieve
/// a `SystemClockContext` may go backwards in time due to DST or timezone changes.
#[derive(Request, Response, Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct SystemClockContext {
    pub posix_time: PosixTime,
    pub steady_time_point: SteadyClockTimePoint,
}

#[derive(Request, Response, Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct CalendarTime {
    pub year: u16,
    pub month: u8,
    pub day: u8,
    pub hour: u8,
    pub minute: u8,
    pub second: u8,
    _pad: u8,
}

#[derive(Request, Response, Default, Clone, Copy, Debug)]
#[repr(C)]
pub struct TimeCalendarAdditionalInfo {
    /// 0-based day-of-week.
    pub week_day: u32,
    /// 0-based day-of-year.
    pub year_day: u32,
    /// Timezone name string.
    pub timezone_name: ArrayString<8>,
    /// 0 = no DST, 1 = DST.
    pub daylight_saving_time: u32,
    /// Seconds relative to UTC for this timezone.
    pub utc_offset: i32,
}

#[nx_derive::ipc_trait]
pub trait SystemClock {
    #[ipc_rid(0)]
    fn get_current_time(&self) -> PosixTime;
    #[ipc_rid(1)]
    fn set_current_time(&mut self, new_time: PosixTime);
    #[ipc_rid(2)]
    fn get_system_clock_context(&self) -> SystemClockContext;
    #[ipc_rid(3)]
    fn set_system_clock_context(&mut self, new_context: SystemClockContext);
    #[ipc_rid(4)]
    fn get_operation_event_readable_handle(&self) -> crate::svc::Handle;
}

#[nx_derive::ipc_trait]
pub trait TimeZoneService {}

#[nx_derive::ipc_trait]
pub trait SteadyClock {}

#[nx_derive::ipc_trait]
pub trait Time {
    #[ipc_rid(0)]
    #[return_session]
    fn get_standard_user_system_clock(&self) -> SystemClock;
    #[ipc_rid(1)]
    #[return_session]
    fn get_standard_network_system_clock(&self) -> SystemClock;
    #[ipc_rid(2)]
    #[return_session]
    fn get_standard_steady_clock(&self) -> SteadyClock;
    #[ipc_rid(3)]
    #[return_session]
    fn get_time_zone_service(&self) -> TimeZoneService;
    #[ipc_rid(4)]
    #[return_session]
    fn get_standard_local_system_clock(&self) -> SystemClock;
    #[ipc_rid(5)]
    #[return_session]
    #[version(version::VersionInterval::from(version::Version::new(4, 0, 0)))]
    fn get_ephemeral_network_system_clock(&self) -> SystemClock;
    #[ipc_rid(20)]
    #[version(version::VersionInterval::from(version::Version::new(6, 0, 0)))]
    fn get_shared_memory_native_handle(&self) -> CopyHandle;
    #[ipc_rid(30)]
    #[version(version::VersionInterval::from_to(
        version::Version::new(6, 0, 0),
        version::Version::new(8, 1, 0)
    ))]
    fn get_standard_network_clock_operation_event_readable_handle(&self);
    #[ipc_rid(31)]
    #[version(version::VersionInterval::from_to(
        version::Version::new(6, 0, 0),
        version::Version::new(8, 1, 0)
    ))]
    fn get_ephemeral_network_clock_operation_event_readable_handle(&self);
    #[ipc_rid(50)]
    #[version(version::VersionInterval::from(version::Version::new(4, 0, 0)))]
    fn set_standard_steady_clock_internal_offset(&self);
    #[ipc_rid(51)]
    #[version(version::VersionInterval::from(version::Version::new(9, 0, 0)))]
    fn get_standard_steady_clock_rtc_value(&self);
    #[ipc_rid(100)]
    fn is_standard_user_system_clock_automatic_correction_enabled(&self);
    #[ipc_rid(101)]
    fn set_standard_user_system_clock_automatic_correction_enabled(&self);
    #[ipc_rid(102)]
    #[version(version::VersionInterval::from(version::Version::new(5, 0, 0)))]
    fn get_standard_user_system_clock_initial_year(&self);
    #[ipc_rid(200)]
    #[version(version::VersionInterval::from(version::Version::new(3, 0, 0)))]
    fn is_standard_network_system_clock_accuracy_sufficient(&self);
    #[ipc_rid(201)]
    #[version(version::VersionInterval::from(version::Version::new(6, 0, 0)))]
    fn get_standard_user_system_clock_automatic_correction_updated_time(&self);
    #[ipc_rid(300)]
    #[version(version::VersionInterval::from(version::Version::new(4, 0, 0)))]
    fn calculate_monotonic_system_clock_base_time_point(&self);
    #[ipc_rid(400)]
    #[version(version::VersionInterval::from(version::Version::new(4, 0, 0)))]
    fn get_clock_snapshot(&self);
    #[ipc_rid(401)]
    #[version(version::VersionInterval::from(version::Version::new(4, 0, 0)))]
    fn get_clock_snapshot_from_system_clock_context(&self);
    #[ipc_rid(500)]
    #[version(version::VersionInterval::from(version::Version::new(4, 0, 0)))]
    fn calculate_standard_user_system_clock_difference_by_user(&self);
    #[ipc_rid(501)]
    #[version(version::VersionInterval::from(version::Version::new(4, 0, 0)))]
    fn calculate_span_between(&self);
    #[ipc_rid(600)]
    #[version(version::VersionInterval::from(version::Version::new(19, 0, 0)))]
    fn get_initial_launch_end_time(&self);
    #[ipc_rid(601)]
    #[version(version::VersionInterval::from(version::Version::new(22, 0, 0)))]
    fn notify_initial_launch_settings_done(&self);
}
