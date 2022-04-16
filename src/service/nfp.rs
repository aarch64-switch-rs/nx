use crate::result::*;
use crate::ipc::sf;
use crate::ipc::client;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::nfp::*;

pub struct User {
    session: sf::Session
}

impl sf::IObject for User {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IUser for User {
    fn initialize(&mut self, process_id: sf::ProcessId, aruid: sf::ProcessId, mcu_data: sf::InMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id, aruid, mcu_data) => ())
    }

    fn finalize(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => ())
    }

    fn list_devices(&mut self, out_devices: sf::OutPointerBuffer) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 2] (out_devices) => (count: u32))
    }

    fn start_detection(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (device_handle) => ())
    }

    fn stop_detection(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (device_handle) => ())
    }

    fn mount(&mut self, device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 5] (device_handle, model_type, mount_target) => ())
    }

    fn unmount(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 6] (device_handle) => ())
    }

    fn open_application_area(&mut self, device_handle: DeviceHandle, access_id: AccessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 7] (device_handle, access_id) => ())
    }

    fn get_application_area(&mut self, device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 8] (device_handle, out_data) => (size: u32))
    }

    fn set_application_area(&mut self, device_handle: DeviceHandle, data: sf::InMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 9] (device_handle, data) => ())
    }

    fn flush(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (device_handle) => ())
    }

    fn restore(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 11] (device_handle) => ())
    }

    // TODO: finish
}

impl client::IClientObject for User {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}