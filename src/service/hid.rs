use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::hid::*;

pub struct AppletResource {
    session: sf::Session
}

impl sf::IObject for AppletResource {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IAppletResource for AppletResource {
    fn get_shared_memory_handle(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (shmem_handle: sf::CopyHandle))
    }
}

impl service::IClientObject for AppletResource {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

pub struct HidServer {
    session: sf::Session
}

impl sf::IObject for HidServer {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IHidServer for HidServer {
    fn create_applet_resource(&mut self, aruid: sf::ProcessId) -> Result<mem::Shared<dyn IAppletResource>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (aruid) => (applet_resource: mem::Shared<AppletResource>))
    }

    fn set_supported_npad_style_set(&mut self, aruid: sf::ProcessId, npad_style_tag: NpadStyleTag) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 100] (npad_style_tag, aruid) => ())
    }

    fn set_supported_npad_id_type(&mut self, aruid: sf::ProcessId, controllers: sf::InPointerBuffer<ControllerId>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 102] (aruid, controllers) => ())
    }

    fn activate_npad(&mut self, aruid: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 103] (aruid) => ())
    }

    fn deactivate_npad(&mut self, aruid: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 104] (aruid) => ())
    }

    fn set_npad_joy_assignment_mode_single(&mut self, aruid: sf::ProcessId, controller: ControllerId, joy_type: NpadJoyDeviceType) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 123] (controller, aruid, joy_type) => ())
    }

    fn set_npad_joy_assignment_mode_dual(&mut self, aruid: sf::ProcessId, controller: ControllerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 124] (controller, aruid) => ())
    }
}

impl service::IClientObject for HidServer {
    fn new(session: sf::Session) -> Self {
        Self { session }
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