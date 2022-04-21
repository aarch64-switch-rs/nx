use crate::ipc::sf::applet;
use crate::ipc::sf::sm;
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
    fn initialize(&mut self, aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id, aruid, mcu_data) => ())
    }

    fn finalize(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => ())
    }

    fn list_devices(&mut self, out_devices: sf::OutPointerBuffer<DeviceHandle>) -> Result<u32> {
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

    fn get_application_area(&mut self, device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 8] (device_handle, out_data) => (size: u32))
    }

    fn set_application_area(&mut self, device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 9] (device_handle, data) => ())
    }

    fn flush(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (device_handle) => ())
    }

    fn restore(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 11] (device_handle) => ())
    }

    fn create_application_area(&mut self, device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 12] (device_handle, access_id, data) => ())
    }

    fn get_tag_info(&mut self, device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 13] (device_handle, out_tag_info) => ())
    }

    fn get_register_info(&mut self, device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 14] (device_handle, out_register_info) => ())
    }

    fn get_common_info(&mut self, device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 15] (device_handle, out_common_info) => ())
    }

    fn get_model_info(&mut self, device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 16] (device_handle, out_model_info) => ())
    }

    fn attach_activate_event(&mut self, device_handle: DeviceHandle) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 17] (device_handle) => (event_handle: sf::CopyHandle))
    }

    fn attach_deactivate_event(&mut self, device_handle: DeviceHandle) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 18] (device_handle) => (event_handle: sf::CopyHandle))
    }

    fn get_state(&mut self) -> Result<State> {
        ipc_client_send_request_command!([self.session.object_info; 19] () => (state: State))
    }

    fn get_device_state(&mut self, device_handle: DeviceHandle) -> Result<DeviceState> {
        ipc_client_send_request_command!([self.session.object_info; 20] (device_handle) => (dev_state: DeviceState))
    }

    fn get_npad_id(&mut self, device_handle: DeviceHandle) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 21] (device_handle) => (npad_id: u32))
    }

    fn get_application_area_size(&mut self, device_handle: DeviceHandle) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 22] (device_handle) => (size: u32))
    }

    fn attach_availability_change_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 23] () => (event_handle: sf::CopyHandle))
    }

    fn recreate_application_area(&mut self, device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 24] (device_handle, access_id, data) => ())
    }
}

impl client::IClientObject for User {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}


pub struct UserManager {
    session: sf::Session
}

impl sf::IObject for UserManager {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IUserManager for UserManager {
    fn create_user_interface(&mut self) -> Result<mem::Shared<dyn IUser>> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (user: mem::Shared<User>))
    }
}

impl client::IClientObject for UserManager {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for UserManager {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nfp:user")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct System {
    session: sf::Session
}

impl sf::IObject for System {
    ipc_sf_object_impl_default_command_metadata!();
}

impl ISystem for System {
    fn initialize_system(&mut self, aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id, aruid, mcu_data) => ())
    }

    fn finalize_system(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => ())
    }

    fn list_devices(&mut self, out_devices: sf::OutPointerBuffer<DeviceHandle>) -> Result<u32> {
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

    fn flush(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (device_handle) => ())
    }

    fn restore(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 11] (device_handle) => ())
    }

    fn get_tag_info(&mut self, device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 13] (device_handle, out_tag_info) => ())
    }

    fn get_register_info(&mut self, device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 14] (device_handle, out_register_info) => ())
    }

    fn get_common_info(&mut self, device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 15] (device_handle, out_common_info) => ())
    }

    fn get_model_info(&mut self, device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 16] (device_handle, out_model_info) => ())
    }

    fn attach_activate_event(&mut self, device_handle: DeviceHandle) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 17] (device_handle) => (event_handle: sf::CopyHandle))
    }

    fn attach_deactivate_event(&mut self, device_handle: DeviceHandle) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 18] (device_handle) => (event_handle: sf::CopyHandle))
    }

    fn get_state(&mut self) -> Result<State> {
        ipc_client_send_request_command!([self.session.object_info; 19] () => (state: State))
    }

    fn get_device_state(&mut self, device_handle: DeviceHandle) -> Result<DeviceState> {
        ipc_client_send_request_command!([self.session.object_info; 20] (device_handle) => (dev_state: DeviceState))
    }

    fn get_npad_id(&mut self, device_handle: DeviceHandle) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 21] (device_handle) => (npad_id: u32))
    }

    fn attach_availability_change_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 23] () => (event_handle: sf::CopyHandle))
    }

    fn format(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 100] (device_handle) => ())
    }

    fn get_admin_info(&mut self, device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 101] (device_handle, out_admin_info) => ())
    }

    fn get_register_info_private(&mut self, device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 102] (device_handle, out_register_info_private) => ())
    }

    fn set_register_info_private(&mut self, device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 103] (device_handle, register_info_private) => ())
    }

    fn delete_register_info(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 104] (device_handle) => ())
    }

    fn delete_application_area(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 105] (device_handle) => ())
    }

    fn exists_application_area(&mut self, device_handle: DeviceHandle) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 105] (device_handle) => (exists: bool))
    }
}

impl client::IClientObject for System {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}


pub struct SystemManager {
    session: sf::Session
}

impl sf::IObject for SystemManager {
    ipc_sf_object_impl_default_command_metadata!();
}

impl ISystemManager for SystemManager {
    fn create_system_interface(&mut self) -> Result<mem::Shared<dyn ISystem>> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (system: mem::Shared<System>))
    }
}

impl client::IClientObject for SystemManager {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for SystemManager {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nfp:sys")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct Debug {
    session: sf::Session
}

impl sf::IObject for Debug {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IDebug for Debug {
    fn initialize_debug(&mut self, aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id, aruid, mcu_data) => ())
    }

    fn finalize_debug(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => ())
    }

    fn list_devices(&mut self, out_devices: sf::OutPointerBuffer<DeviceHandle>) -> Result<u32> {
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

    fn get_application_area(&mut self, device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 8] (device_handle, out_data) => (size: u32))
    }

    fn set_application_area(&mut self, device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 9] (device_handle, data) => ())
    }

    fn flush(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (device_handle) => ())
    }

    fn restore(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 11] (device_handle) => ())
    }

    fn create_application_area(&mut self, device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 12] (device_handle, access_id, data) => ())
    }

    fn get_tag_info(&mut self, device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 13] (device_handle, out_tag_info) => ())
    }

    fn get_register_info(&mut self, device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 14] (device_handle, out_register_info) => ())
    }

    fn get_common_info(&mut self, device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 15] (device_handle, out_common_info) => ())
    }

    fn get_model_info(&mut self, device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 16] (device_handle, out_model_info) => ())
    }

    fn attach_activate_event(&mut self, device_handle: DeviceHandle) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 17] (device_handle) => (event_handle: sf::CopyHandle))
    }

    fn attach_deactivate_event(&mut self, device_handle: DeviceHandle) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 18] (device_handle) => (event_handle: sf::CopyHandle))
    }

    fn get_state(&mut self) -> Result<State> {
        ipc_client_send_request_command!([self.session.object_info; 19] () => (state: State))
    }

    fn get_device_state(&mut self, device_handle: DeviceHandle) -> Result<DeviceState> {
        ipc_client_send_request_command!([self.session.object_info; 20] (device_handle) => (dev_state: DeviceState))
    }

    fn get_npad_id(&mut self, device_handle: DeviceHandle) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 21] (device_handle) => (npad_id: u32))
    }

    fn get_application_area_size(&mut self, device_handle: DeviceHandle) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 22] (device_handle) => (size: u32))
    }

    fn attach_availability_change_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 23] () => (event_handle: sf::CopyHandle))
    }

    fn recreate_application_area(&mut self, device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 24] (device_handle, access_id, data) => ())
    }

    fn format(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 100] (device_handle) => ())
    }

    fn get_admin_info(&mut self, device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 101] (device_handle, out_admin_info) => ())
    }

    fn get_register_info_private(&mut self, device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 102] (device_handle, out_register_info_private) => ())
    }

    fn set_register_info_private(&mut self, device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 103] (device_handle, register_info_private) => ())
    }

    fn delete_register_info(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 104] (device_handle) => ())
    }

    fn delete_application_area(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 105] (device_handle) => ())
    }

    fn exists_application_area(&mut self, device_handle: DeviceHandle) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 105] (device_handle) => (exists: bool))
    }

    fn get_all(&mut self, device_handle: DeviceHandle, out_data: sf::OutFixedPointerBuffer<NfpData>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 200] (device_handle, out_data) => ())
    }

    fn set_all(&mut self, device_handle: DeviceHandle, data: sf::InFixedPointerBuffer<NfpData>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 201] (device_handle, data) => ())
    }

    fn flush_debug(&mut self, device_handle: DeviceHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 202] (device_handle) => ())
    }

    fn break_tag(&mut self, device_handle: DeviceHandle, break_type: BreakType) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 203] (device_handle, break_type) => ())
    }

    fn read_backup_data(&mut self, device_handle: DeviceHandle, out_buf: sf::OutMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 204] (device_handle, out_buf) => (read_size: u32))
    }

    fn write_backup_data(&mut self, device_handle: DeviceHandle, buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 205] (device_handle, buf) => ())
    }

    fn write_ntf(&mut self, device_handle: DeviceHandle, write_type: WriteType, buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 206] (device_handle, write_type, buf) => ())
    }
}

impl client::IClientObject for Debug {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}


pub struct DebugManager {
    session: sf::Session
}

impl sf::IObject for DebugManager {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IDebugManager for DebugManager {
    fn create_debug_interface(&mut self) -> Result<mem::Shared<dyn IDebug>> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (debug: mem::Shared<Debug>))
    }
}

impl client::IClientObject for DebugManager {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for DebugManager {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nfp:dbg")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}