use crate::result::*;
use crate::ipc::sf;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Policy {
    ErrorReportAndErrorScreen,
    ErrorReport,
    ErrorScreen,
}

pub trait IService {
    ipc_interface_define_command!(throw_with_policy: (rc: ResultCode, policy: Policy, process_id: sf::ProcessId) => ());
}