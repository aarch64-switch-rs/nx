use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::nfp::*;

pub struct User {
    session: sf::Session
}

impl sf::IObject for User {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            initialize: 0,
            finalize: 1,
            list_devices: 2,
            start_detection: 3,
            stop_detection: 4,
            mount: 5,
            unmount: 6,
            open_application_area: 7,
            get_application_area: 8,
            set_application_area: 9,
            flush: 10,
            restore: 11,
            create_application_area: 12,
            get_tag_info: 13,
            get_register_info: 14,
            get_common_info: 15,
            get_model_info: 16,
            attach_activate_event: 17,
            attach_deactivate_event: 18,
            get_state: 19,
            get_device_state: 20,
            get_npad_id: 21,
            get_application_area_size: 22,
            attach_availability_change_event: 23,
            recreate_application_area: 24
        }
    }
}

impl service::IClientObject for User {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
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

    fn mount(&mut self, device_handle: DeviceHandle, device_type: DeviceType, mount_target: MountTarget) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 5] (device_handle, device_type, mount_target) => ())
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

    // TODO
}