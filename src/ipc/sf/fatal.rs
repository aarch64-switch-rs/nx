use crate::result::*;
use crate::ipc::sf;
use crate::ipc::sf::CmifPidPlaceholder;
use crate::version;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum FatalPolicy {
    ErrorReportAndErrorScreen,
    ErrorReport,
    ErrorScreen,
}
//api_mark_request_command_parameters_types_as_copy!(FatalPolicy);

ipc_sf_define_default_interface_client!(Service);
ipc_sf_define_interface_trait! {
	trait Service {
        throw_fatal_with_policy [1, version::VersionInterval::all()]: (process_id: sf::ProcessId, rc: ResultCode, policy: FatalPolicy, _placeholder: CmifPidPlaceholder) => ();
    }
}