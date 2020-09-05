use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::fatal::*;

pub struct Service {
    session: sf::Session
}

impl sf::IObject for Service {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            throw_with_policy: 1
        }
    }
}

impl service::IClientObject for Service {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IService for Service {
    fn throw_with_policy(&mut self, rc: ResultCode, policy: Policy, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (rc, policy, process_id) => ())
    }
}

impl service::IService for Service {
    fn get_name() -> &'static str {
        nul!("fatal:u")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}