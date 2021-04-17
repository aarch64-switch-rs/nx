use crate::result::*;
use crate::ipc::cmif::sf;
use crate::service;

pub use crate::ipc::cmif::sf::spl::*;

pub struct RandomInterface {
    session: sf::Session
}

impl sf::IObject for RandomInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(generate_random_bytes: 0)
        ]
    }
}

impl service::cmif::IClientObject for RandomInterface {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IRandomInterface for RandomInterface {
    fn generate_random_bytes(&mut self, out_buf: sf::OutMapAliasBuffer) -> Result<()> {
        ipc_cmif_client_send_request_command!([self.session.object_info; 0] (out_buf) => ())
    }
}

impl service::cmif::IService for RandomInterface {
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