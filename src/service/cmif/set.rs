use crate::result::*;
use crate::ipc::cmif::sf;
use crate::service;

pub use crate::ipc::cmif::sf::set::*;

pub struct SystemSettingsServer {
    session: sf::Session
}

impl sf::IObject for SystemSettingsServer {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            nipc_cmif_interface_make_command_meta!(get_firmware_version: 3)
        ]
    }
}

impl service::cmif::IClientObject for SystemSettingsServer {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl ISystemSettingsServer for SystemSettingsServer {
    fn get_firmware_version(&mut self, out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 3] (out_version) => ())
    }
}

impl service::cmif::IService for SystemSettingsServer {
    fn get_name() -> &'static str {
        nul!("set:sys")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}