use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::ipc::client;
use crate::service;

pub use crate::ipc::sf::spl::*;

pub struct RandomInterface {
    session: sf::Session
}

impl sf::IObject for RandomInterface {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IRandomInterface for RandomInterface {
    fn generate_random_bytes(&mut self, out_buf: sf::OutMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (out_buf) => ())
    }
}

impl client::IClientObject for RandomInterface {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for RandomInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("csrng")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}