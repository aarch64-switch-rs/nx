use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::ipc::sf::mii;
use crate::service;

pub use crate::ipc::sf::set::*;

ipc_sf_client_object_define_default_impl!(SystemSettingsServer);

impl ISystemSettingsServer for SystemSettingsServer {
    fn get_firmware_version(&mut self, out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (out_version) => ())
    }

    fn get_firmware_version_2(&mut self, out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (out_version) => ())
    }

    fn get_mii_author_id(&mut self) -> Result<mii::CreateId> {
        ipc_client_send_request_command!([self.session.object_info; 90] () => (id: mii::CreateId))
    }
}

impl service::IService for SystemSettingsServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("set:sys")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}