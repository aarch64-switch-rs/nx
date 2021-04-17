use crate::result::*;
use crate::ipc::cmif::sf;
use crate::service;

pub use crate::ipc::cmif::sf::psm::*;

pub struct PsmServer {
    session: sf::Session
}

impl sf::IObject for PsmServer {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            nipc_cmif_interface_make_command_meta!(get_battery_charge_percentage: 0)
        ]
    }
}

impl service::cmif::IClientObject for PsmServer {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IPsmServer for PsmServer {
    fn get_battery_charge_percentage(&mut self) -> Result<u32> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 0] () => (charge: u32))
    }
}

impl service::cmif::IService for PsmServer {
    fn get_name() -> &'static str {
        nul!("psm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}