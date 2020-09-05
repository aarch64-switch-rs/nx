use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;

use crate::ipc::sf::applet;
use crate::ipc::sf::mii;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DeviceHandle {
    pub npad_id: u32,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum State {
    NonInitialized = 0,
    Initialized = 1
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum DeviceState {
    Initialized = 0,
    SearchingForTag = 1,
    TagFound = 2,
    TagRemoved = 3,
    TagMounted = 4,
    Unavailable = 5,
    Finalized = 6
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum DeviceType {
    Amiibo = 0
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum MountTarget {
    Rom = 1,
    Ram = 2,
    All = 3
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TagInfo {
    pub uuid: [u8; 10],
    pub uuid_length: u8,
    pub reserved_1: [u8; 0x15],
    pub protocol: u32,
    pub tag_type: u32,
    pub reserved_2: [u8; 0x30]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct RegisterInfo {
    pub mii_charinfo: mii::CharInfo,
    pub first_write_date: Date,
    pub name: util::CString<41>,
    pub font_region: u8,
    pub reserved: [u8; 0x7A]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct CommonInfo {
    pub last_write_date: Date,
    pub write_counter: u16,
    pub version: u16,
    pub application_area_size: u32,
    pub reserved: [u8; 0x34]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ModelInfo {
    pub game_character_id: u16,
    pub character_variant: u8,
    pub figure_type: u8,
    pub model_number: u16,
    pub series: u8,
    pub reserved: [u8; 0x39]
}

pub type AccessId = u32;

pub trait IUser {
    ipc_interface_define_command!(initialize: (aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer) => ());
    ipc_interface_define_command!(finalize: () => ());
    ipc_interface_define_command!(list_devices: (out_devices: sf::OutPointerBuffer) => (count: u32));
    ipc_interface_define_command!(start_detection: (device_handle: DeviceHandle) => ());
    ipc_interface_define_command!(stop_detection: (device_handle: DeviceHandle) => ());
    ipc_interface_define_command!(mount: (device_handle: DeviceHandle, device_type: DeviceType, mount_target: MountTarget) => ());
    ipc_interface_define_command!(unmount: (device_handle: DeviceHandle) => ());
    ipc_interface_define_command!(open_application_area: (device_handle: DeviceHandle, access_id: AccessId) => ());
    ipc_interface_define_command!(get_application_area: (device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer) => (size: u32));
    ipc_interface_define_command!(set_application_area: (device_handle: DeviceHandle, data: sf::InMapAliasBuffer) => ());
    ipc_interface_define_command!(flush: (device_handle: DeviceHandle) => ());
    ipc_interface_define_command!(restore: (device_handle: DeviceHandle) => ());
    ipc_interface_define_command!(create_application_area: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer) => ());
    ipc_interface_define_command!(get_tag_info: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) => ());
    ipc_interface_define_command!(get_register_info: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) => ());
    ipc_interface_define_command!(get_common_info: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) => ());
    ipc_interface_define_command!(get_model_info: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) => ());
    ipc_interface_define_command!(attach_activate_event: (device_handle: DeviceHandle) => (activate_event: sf::CopyHandle));
    ipc_interface_define_command!(attach_deactivate_event: (device_handle: DeviceHandle) => (deactivate_event: sf::CopyHandle));
    ipc_interface_define_command!(get_state: () => (state: State));
    ipc_interface_define_command!(get_device_state: (device_handle: DeviceHandle) => (device_state: DeviceState));
    ipc_interface_define_command!(get_npad_id: (device_handle: DeviceHandle) => (npad_id: u32));
    ipc_interface_define_command!(get_application_area_size: (device_handle: DeviceHandle) => (size: u32));
    ipc_interface_define_command!(attach_availability_change_event: () => (availability_change_event: sf::CopyHandle));
    ipc_interface_define_command!(recreate_application_area: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer) => ());
}

pub trait IUserManager {
    ipc_interface_define_command!(create_user_interface: () => (user_interface: mem::Shared<dyn sf::IObject>));
}