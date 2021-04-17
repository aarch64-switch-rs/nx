use crate::result::*;
use crate::ipc::cmif::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::cmif::sf::nfp::*;

pub struct User {
    session: sf::Session
}

impl sf::IObject for User {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            nipc_cmif_interface_make_command_meta!(initialize: 0),
            nipc_cmif_interface_make_command_meta!(finalize: 1),
            nipc_cmif_interface_make_command_meta!(list_devices: 2),
            nipc_cmif_interface_make_command_meta!(start_detection: 3),
            nipc_cmif_interface_make_command_meta!(stop_detection: 4),
            nipc_cmif_interface_make_command_meta!(mount: 5),
            nipc_cmif_interface_make_command_meta!(unmount: 6),
            nipc_cmif_interface_make_command_meta!(open_application_area: 7),
            nipc_cmif_interface_make_command_meta!(get_application_area: 8),
            nipc_cmif_interface_make_command_meta!(set_application_area: 9),
            nipc_cmif_interface_make_command_meta!(flush: 10),
            nipc_cmif_interface_make_command_meta!(restore: 11),
            nipc_cmif_interface_make_command_meta!(create_application_area: 12),
            nipc_cmif_interface_make_command_meta!(get_tag_info: 13),
            nipc_cmif_interface_make_command_meta!(get_register_info: 14),
            nipc_cmif_interface_make_command_meta!(get_common_info: 15),
            nipc_cmif_interface_make_command_meta!(get_model_info: 16),
            nipc_cmif_interface_make_command_meta!(attach_activate_event: 17),
            nipc_cmif_interface_make_command_meta!(attach_deactivate_event: 18),
            nipc_cmif_interface_make_command_meta!(get_state: 19),
            nipc_cmif_interface_make_command_meta!(get_device_state: 20),
            nipc_cmif_interface_make_command_meta!(get_npad_id: 21),
            nipc_cmif_interface_make_command_meta!(get_application_area_size: 22),
            nipc_cmif_interface_make_command_meta!(attach_availability_change_event: 23, [(3, 0, 0) =>]),
            nipc_cmif_interface_make_command_meta!(recreate_application_area: 24, [(3, 0, 0) =>])
        ]
    }
}

impl service::cmif::IClientObject for User {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IUser for User {
    fn initialize(&mut self, process_id: sf::ProcessId, aruid: sf::ProcessId, mcu_data: sf::InMapAliasBuffer) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 0] (process_id, aruid, mcu_data) => ())
    }

    fn finalize(&mut self) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 1] () => ())
    }

    fn list_devices(&mut self, out_devices: sf::OutPointerBuffer) -> Result<u32> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 2] (out_devices) => (count: u32))
    }

    fn start_detection(&mut self, device_handle: DeviceHandle) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 3] (device_handle) => ())
    }

    fn stop_detection(&mut self, device_handle: DeviceHandle) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 4] (device_handle) => ())
    }

    fn mount(&mut self, device_handle: DeviceHandle, device_type: DeviceType, mount_target: MountTarget) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 5] (device_handle, device_type, mount_target) => ())
    }

    fn unmount(&mut self, device_handle: DeviceHandle) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 6] (device_handle) => ())
    }

    fn open_application_area(&mut self, device_handle: DeviceHandle, access_id: AccessId) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 7] (device_handle, access_id) => ())
    }

    fn get_application_area(&mut self, device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer) -> Result<u32> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 8] (device_handle, out_data) => (size: u32))
    }

    fn set_application_area(&mut self, device_handle: DeviceHandle, data: sf::InMapAliasBuffer) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 9] (device_handle, data) => ())
    }

    fn flush(&mut self, device_handle: DeviceHandle) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 10] (device_handle) => ())
    }

    fn restore(&mut self, device_handle: DeviceHandle) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 11] (device_handle) => ())
    }

    // TODO
}