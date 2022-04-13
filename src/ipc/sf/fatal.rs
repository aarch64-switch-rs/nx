use crate::result::*;
use crate::ipc::sf;
use crate::version;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Policy {
    ErrorReportAndErrorScreen,
    ErrorReport,
    ErrorScreen,
}

ipc_sf_define_interface_trait! {
    trait IService {
        throw_with_policy [1, version::VersionInterval::all()]: (rc: ResultCode, policy: Policy, process_id: sf::ProcessId) => ();
    }
}