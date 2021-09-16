use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::pm::*;

pub struct InformationInterface {
    session: sf::Session
}

impl sf::IObject for InformationInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(get_program_id: 0)
        ]
    }
}

impl service::IClientObject for InformationInterface {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl IInformationInterface for InformationInterface {
    fn get_program_id(&mut self, process_id: u64) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id) => (program_id: u64))
    }
}

impl service::IService for InformationInterface {
    fn get_name() -> &'static str {
        nul!("pm:info")
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

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(get_application_process_id: 5)
        ]
    }
}

impl service::IClientObject for DebugMonitorInterface {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl IDebugMonitorInterface for DebugMonitorInterface {
    fn get_application_process_id(&mut self) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 5] () => (process_id: u64))
    }
}

impl service::IService for DebugMonitorInterface {
    fn get_name() -> &'static str {
        nul!("pm:dmnt")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}