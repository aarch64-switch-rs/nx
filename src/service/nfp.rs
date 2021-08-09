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
        vec! [
            ipc_cmif_interface_make_command_meta!(initialize: 0),
            ipc_cmif_interface_make_command_meta!(finalize: 1),
            ipc_cmif_interface_make_command_meta!(list_devices: 2),
            ipc_cmif_interface_make_command_meta!(start_detection: 3),
            ipc_cmif_interface_make_command_meta!(stop_detection: 4),
            ipc_cmif_interface_make_command_meta!(mount: 5),
            ipc_cmif_interface_make_command_meta!(unmount: 6),
            ipc_cmif_interface_make_command_meta!(open_application_area: 7),
            ipc_cmif_interface_make_command_meta!(get_application_area: 8),
            ipc_cmif_interface_make_command_meta!(set_application_area: 9),
            ipc_cmif_interface_make_command_meta!(flush: 10),
            ipc_cmif_interface_make_command_meta!(restore: 11),
            ipc_cmif_interface_make_command_meta!(create_application_area: 12),
            ipc_cmif_interface_make_command_meta!(get_tag_info: 13),
            ipc_cmif_interface_make_command_meta!(get_register_info: 14),
            ipc_cmif_interface_make_command_meta!(get_common_info: 15),
            ipc_cmif_interface_make_command_meta!(get_model_info: 16),
            ipc_cmif_interface_make_command_meta!(attach_activate_event: 17),
            ipc_cmif_interface_make_command_meta!(attach_deactivate_event: 18),
            ipc_cmif_interface_make_command_meta!(get_state: 19),
            ipc_cmif_interface_make_command_meta!(get_device_state: 20),
            ipc_cmif_interface_make_command_meta!(get_npad_id: 21),
            ipc_cmif_interface_make_command_meta!(get_application_area_size: 22),
            ipc_cmif_interface_make_command_meta!(attach_availability_change_event: 23, [(3, 0, 0) =>]),
            ipc_cmif_interface_make_command_meta!(recreate_application_area: 24, [(3, 0, 0) =>])
        ]
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

    // TODO: finish
}