use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;

pub use crate::ipc::sf::pm::*;

pub struct InformationInterface {
    session: sf::Session
}

impl sf::IObject for InformationInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IInformationInterface for InformationInterface {
    fn get_program_id(&mut self, process_id: u64) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id) => (program_id: u64))
    }
}

impl service::IClientObject for InformationInterface {
    fn new(session: sf::Session) -> Self {
        Self { session }
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

pub struct DebugMonitorInterface {
    session: sf::Session
}

impl sf::IObject for DebugMonitorInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IDebugMonitorInterface for DebugMonitorInterface {
    fn get_application_process_id_deprecated(&mut self) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 5] () => (process_id: u64))
    }

    fn get_application_process_id(&mut self) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 4] () => (process_id: u64))
    }
}

impl service::IClientObject for DebugMonitorInterface {
    fn new(session: sf::Session) -> Self {
        Self { session }
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