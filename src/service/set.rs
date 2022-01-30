use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::set::*;

pub struct SystemSettingsServer {
    session: sf::Session
}

impl sf::IObject for SystemSettingsServer {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(get_firmware_version: 3),
            ipc_cmif_interface_make_command_meta!(get_firmware_version_2: 4)
        ]
    }
}

impl service::IClientObject for SystemSettingsServer {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl ISystemSettingsServer for SystemSettingsServer {
    fn get_firmware_version(&mut self, out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (out_version) => ())
    }

    fn get_firmware_version_2(&mut self, out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (out_version) => ())
    }
}

impl service::IService for SystemSettingsServer {
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