use crate::result::*;
use crate::ipc::sf::{self, ncm, sm};
use crate::service;

pub use crate::ipc::sf::pm::*;

ipc_client_define_object_default!(InformationInterface);

impl IInformationInterface for InformationInterface {
    fn get_program_id(&mut self, process_id: u64) -> Result<ncm::ProgramId> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id) => (program_id: ncm::ProgramId))
    }
}

impl service::IService for InformationInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("pm:info")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

ipc_client_define_object_default!(DebugMonitorInterface);

impl IDebugMonitorInterface for DebugMonitorInterface {
    fn get_application_process_id_deprecated(&mut self) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 5] () => (process_id: u64))
    }

    fn get_application_process_id(&mut self) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 4] () => (process_id: u64))
    }
}

impl service::IService for DebugMonitorInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("pm:dmnt")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}