use crate::ipc::sf;
use crate::result::*;
use crate::version;

use nx_derive::{Request, Response};

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum FatalPolicy {
    ErrorReportAndErrorScreen,
    ErrorReport,
    ErrorScreen,
}


ipc_sf_define_interface_trait! {
    trait Fatal {
        throw_fatal_with_policy [1, version::VersionInterval::all()]: (rc: ResultCode, policy: FatalPolicy, process_id: sf::ProcessId) =>  () ();
    }
}
