use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;

pub use crate::ipc::sf::fatal::*;

pub struct Service {
    session: sf::Session
}

impl sf::IObject for Service {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IService for Service {
    fn throw_with_policy(&mut self, rc: ResultCode, policy: Policy, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (rc, policy, process_id) => ())
    }
}

impl service::IClientObject for Service {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl service::IService for Service {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("fatal:u")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}