use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::spl::*;

pub struct RandomInterface {
    session: sf::Session
}

impl sf::IObject for RandomInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_interface_make_command_meta!(generate_random_bytes: 0)
        ]
    }
}

impl service::IClientObject for RandomInterface {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IRandomInterface for RandomInterface {
    fn generate_random_bytes(&mut self, out_buf: sf::OutMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (out_buf) => ())
    }
}

impl service::IService for RandomInterface {
    fn get_name() -> &'static str {
        nul!("csrng")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}