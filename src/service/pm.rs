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
        ipc_server_make_command_table! {
            get_program_id: 0
        }
    }
}

impl service::IClientObject for InformationInterface {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
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