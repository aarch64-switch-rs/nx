use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::hid::*;

ipc_sf_client_object_define_default_impl!(AppletResource);

impl IAppletResource for AppletResource {
    fn get_shared_memory_handle(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (shmem_handle: sf::CopyHandle))
    }
}

ipc_sf_client_object_define_default_impl!(HidServer);

impl IHidServer for HidServer {
    fn create_applet_resource(&mut self, aruid: sf::ProcessId) -> Result<mem::Shared<dyn IAppletResource>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (aruid) => (applet_resource: mem::Shared<AppletResource>))
    }

    fn set_supported_npad_style_set(&mut self, aruid: sf::ProcessId, npad_style_tag: NpadStyleTag) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 100] (npad_style_tag, aruid) => ())
    }

    fn set_supported_npad_id_type(&mut self, aruid: sf::ProcessId, npad_ids: sf::InPointerBuffer<NpadIdType>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 102] (aruid, npad_ids) => ())
    }

    fn activate_npad(&mut self, aruid: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 103] (aruid) => ())
    }

    fn deactivate_npad(&mut self, aruid: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 104] (aruid) => ())
    }

    fn set_npad_joy_assignment_mode_single(&mut self, aruid: sf::ProcessId, npad_id: NpadIdType, joy_type: NpadJoyDeviceType) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 123] (npad_id, aruid, joy_type) => ())
    }

    fn set_npad_joy_assignment_mode_dual(&mut self, aruid: sf::ProcessId, npad_id: NpadIdType) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 124] (npad_id, aruid) => ())
    }
}

impl service::IService for HidServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("hid")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}