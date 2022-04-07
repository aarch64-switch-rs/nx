use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;

use crate::ipc::sf::applet;
use crate::ipc::sf::mii;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct McuVersionData {
    pub version: u64,
    pub reserved: [u8; 0x18]
}
const_assert!(core::mem::size_of::<McuVersionData>() == 0x20);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DeviceHandle {
    pub id: u32,
    pub reserved: [u8; 4]
}
const_assert!(core::mem::size_of::<DeviceHandle>() == 0x8);

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
pub enum ModelType {
    Amiibo = 0
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum MountTarget {
    Rom = 1,
    Ram = 2,
    All = 3
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8
}
const_assert!(core::mem::size_of::<Date>() == 0x4);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct TagInfo {
    pub uuid: [u8; 10],
    pub uuid_length: u8,
    pub reserved_1: [u8; 0x15],
    pub protocol: u32,
    pub tag_type: u32,
    pub reserved_2: [u8; 0x30]
}
const_assert!(core::mem::size_of::<TagInfo>() == 0x58);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RegisterInfo {
    pub mii_charinfo: mii::CharInfo,
    pub first_write_date: Date,
    pub name: util::CString<41>,
    pub unk: u8,
    pub reserved: [u8; 0x7A]
}
const_assert!(core::mem::size_of::<RegisterInfo>() == 0x100);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CommonInfo {
    pub last_write_date: Date,
    pub write_counter: u16,
    pub version: u8,
    pub pad: u8,
    pub application_area_size: u32,
    pub reserved: [u8; 0x34]
}
const_assert!(core::mem::size_of::<CommonInfo>() == 0x40);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct ModelInfo {
    pub game_character_id: u16,
    pub character_variant: u8,
    pub series: u8,
    pub model_number: u16,
    pub figure_type: u8,
    pub reserved: [u8; 0x39]
}
const_assert!(core::mem::size_of::<ModelInfo>() == 0x40);

pub type AccessId = u32;

bit_enum! {
    AdminInfoFlags (u8) {
        IsInitialized = bit!(0),
        HasApplicationArea = bit!(1),
        Unk_2 = bit!(2),
        Unk_3 = bit!(3)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum ProgramIdConsoleType {
    #[default] Default = 0,
    NintendoWiiU = 1,
    Nintendo3DS = 2,
    NintendoSwitch = 3
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct AdminInfo {
    pub program_id: u64,
    pub access_id: AccessId,
    pub crc32_change_counter: u16,
    pub flags: AdminInfoFlags, // Raw amiibo settings flags without the first 4 bits
    pub unk_0x2: u8, // Always 0x2
    pub console_type: ProgramIdConsoleType,
    pub pad: [u8; 0x7],
    pub reserved: [u8; 0x28]
}
const_assert!(core::mem::size_of::<AdminInfo>() == 0x40);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RegisterInfoPrivate {
    pub mii_store_data: mii::StoreData,
    pub first_write_date: Date,
    pub name: util::CString<41>,
    pub unk: u8,
    pub reserved: [u8; 0x8E]
}
const_assert!(core::mem::size_of::<RegisterInfoPrivate>() == 0x100);

pub trait IUser {
    ipc_cmif_interface_define_command!(initialize: (aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) => ());
    ipc_cmif_interface_define_command!(finalize: () => ());
    ipc_cmif_interface_define_command!(list_devices: (out_devices: sf::OutPointerBuffer<DeviceHandle>) => (count: u32));
    ipc_cmif_interface_define_command!(start_detection: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(stop_detection: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(mount: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) => ());
    ipc_cmif_interface_define_command!(unmount: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(open_application_area: (device_handle: DeviceHandle, access_id: AccessId) => ());
    ipc_cmif_interface_define_command!(get_application_area: (device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) => (size: u32));
    ipc_cmif_interface_define_command!(set_application_area: (device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) => ());
    ipc_cmif_interface_define_command!(flush: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(restore: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(create_application_area: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ());
    ipc_cmif_interface_define_command!(get_tag_info: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) => ());
    ipc_cmif_interface_define_command!(get_register_info: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) => ());
    ipc_cmif_interface_define_command!(get_common_info: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) => ());
    ipc_cmif_interface_define_command!(get_model_info: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) => ());
    ipc_cmif_interface_define_command!(attach_activate_event: (device_handle: DeviceHandle) => (activate_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(attach_deactivate_event: (device_handle: DeviceHandle) => (deactivate_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(get_state: () => (state: State));
    ipc_cmif_interface_define_command!(get_device_state: (device_handle: DeviceHandle) => (device_state: DeviceState));
    ipc_cmif_interface_define_command!(get_npad_id: (device_handle: DeviceHandle) => (npad_id: u32));
    ipc_cmif_interface_define_command!(get_application_area_size: (device_handle: DeviceHandle) => (size: u32));
    ipc_cmif_interface_define_command!(attach_availability_change_event: () => (availability_change_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(recreate_application_area: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ());
}

pub trait IUserManager {
    ipc_cmif_interface_define_command!(create_user_interface: () => (user_interface: mem::Shared<dyn sf::IObject>));
}

pub trait ISystem {
    ipc_cmif_interface_define_command!(initialize_system: (aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) => ());
    ipc_cmif_interface_define_command!(finalize_system: () => ());
    ipc_cmif_interface_define_command!(list_devices: (out_devices: sf::OutPointerBuffer<DeviceHandle>) => (count: u32));
    ipc_cmif_interface_define_command!(start_detection: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(stop_detection: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(mount: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) => ());
    ipc_cmif_interface_define_command!(unmount: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(flush: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(restore: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(get_tag_info: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) => ());
    ipc_cmif_interface_define_command!(get_register_info: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) => ());
    ipc_cmif_interface_define_command!(get_common_info: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) => ());
    ipc_cmif_interface_define_command!(get_model_info: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) => ());
    ipc_cmif_interface_define_command!(attach_activate_event: (device_handle: DeviceHandle) => (activate_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(attach_deactivate_event: (device_handle: DeviceHandle) => (deactivate_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(get_state: () => (state: State));
    ipc_cmif_interface_define_command!(get_device_state: (device_handle: DeviceHandle) => (device_state: DeviceState));
    ipc_cmif_interface_define_command!(get_npad_id: (device_handle: DeviceHandle) => (npad_id: u32));
    ipc_cmif_interface_define_command!(attach_availability_change_event: () => (availability_change_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(format: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(get_admin_info: (device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) => ());
    ipc_cmif_interface_define_command!(get_register_info_private: (device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) => ());
    ipc_cmif_interface_define_command!(set_register_info_private: (device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) => ());
    ipc_cmif_interface_define_command!(delete_register_info: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(delete_application_area: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(exists_application_area: (device_handle: DeviceHandle) => (exists: bool));
}

pub trait ISystemManager {
    ipc_cmif_interface_define_command!(create_system_interface: () => (system_interface: mem::Shared<dyn sf::IObject>));
}

pub trait IDebug {
    ipc_cmif_interface_define_command!(initialize_debug: (aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) => ());
    ipc_cmif_interface_define_command!(finalize_debug: () => ());
    ipc_cmif_interface_define_command!(list_devices: (out_devices: sf::OutPointerBuffer<DeviceHandle>) => (count: u32));
    ipc_cmif_interface_define_command!(start_detection: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(stop_detection: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(mount: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) => ());
    ipc_cmif_interface_define_command!(unmount: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(open_application_area: (device_handle: DeviceHandle, access_id: AccessId) => ());
    ipc_cmif_interface_define_command!(get_application_area: (device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) => (size: u32));
    ipc_cmif_interface_define_command!(set_application_area: (device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) => ());
    ipc_cmif_interface_define_command!(flush: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(restore: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(create_application_area: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ());
    ipc_cmif_interface_define_command!(get_tag_info: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) => ());
    ipc_cmif_interface_define_command!(get_register_info: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) => ());
    ipc_cmif_interface_define_command!(get_common_info: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) => ());
    ipc_cmif_interface_define_command!(get_model_info: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) => ());
    ipc_cmif_interface_define_command!(attach_activate_event: (device_handle: DeviceHandle) => (activate_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(attach_deactivate_event: (device_handle: DeviceHandle) => (deactivate_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(get_state: () => (state: State));
    ipc_cmif_interface_define_command!(get_device_state: (device_handle: DeviceHandle) => (device_state: DeviceState));
    ipc_cmif_interface_define_command!(get_npad_id: (device_handle: DeviceHandle) => (npad_id: u32));
    ipc_cmif_interface_define_command!(get_application_area_size: (device_handle: DeviceHandle) => (size: u32));
    ipc_cmif_interface_define_command!(attach_availability_change_event: () => (availability_change_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(recreate_application_area: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ());
    ipc_cmif_interface_define_command!(format: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(get_admin_info: (device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) => ());
    ipc_cmif_interface_define_command!(get_register_info_private: (device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) => ());
    ipc_cmif_interface_define_command!(set_register_info_private: (device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) => ());
    ipc_cmif_interface_define_command!(delete_register_info: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(delete_application_area: (device_handle: DeviceHandle) => ());
    ipc_cmif_interface_define_command!(exists_application_area: (device_handle: DeviceHandle) => (exists: bool));
    // TODO: remaining commands
}

pub trait IDebugManager {
    ipc_cmif_interface_define_command!(create_debug_interface: () => (debug_interface: mem::Shared<dyn sf::IObject>));
}