use crate::ipc::sf;
use crate::ipc::sf::applet;
use crate::ipc::sf::hid;
use crate::ipc::sf::mii;
use crate::result::*;
use crate::util;
use crate::version;

use super::ncm;

use nx_derive::{Request, Response};

pub mod rc;

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct McuVersionData {
    pub version: u64,
    pub reserved: [u8; 0x18],
}
const_assert!(core::mem::size_of::<McuVersionData>() == 0x20);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DeviceHandle {
    pub id: u32,
    pub reserved: [u8; 4],
}
const_assert!(core::mem::size_of::<DeviceHandle>() == 0x8);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum State {
    NonInitialized = 0,
    Initialized = 1,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum DeviceState {
    Initialized = 0,
    SearchingForTag = 1,
    TagFound = 2,
    TagRemoved = 3,
    TagMounted = 4,
    Unavailable = 5,
    Finalized = 6,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ModelType {
    Amiibo = 0,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum MountTarget {
    Rom = 1,
    Ram = 2,
    All = 3,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}
const_assert!(core::mem::size_of::<Date>() == 0x4);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct TagInfo {
    pub uuid: [u8; 10],
    pub uuid_length: u8,
    pub reserved_1: [u8; 0x15],
    pub protocol: u32,
    pub tag_type: u32,
    pub reserved_2: [u8; 0x30],
}
const_assert!(core::mem::size_of::<TagInfo>() == 0x58);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RegisterInfo {
    pub mii_charinfo: mii::CharInfo,
    pub first_write_date: Date,
    pub name: util::ArrayString<41>,
    pub font_region: u8,
    pub reserved: [u8; 0x7A],
}
const_assert!(core::mem::size_of::<RegisterInfo>() == 0x100);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CommonInfo {
    pub last_write_date: Date,
    pub write_counter: u16,
    pub version: u8,
    pub pad: u8,
    pub application_area_size: u32,
    pub reserved: [u8; 0x34],
}
const_assert!(core::mem::size_of::<CommonInfo>() == 0x40);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct ModelInfo {
    pub game_character_id: u16,
    pub character_variant: u8,
    pub series: u8,
    pub model_number: u16,
    pub figure_type: u8,
    pub reserved: [u8; 0x39],
}
const_assert!(core::mem::size_of::<ModelInfo>() == 0x40);

pub type AccessId = u32;

define_bit_enum! {
    AdminInfoFlags (u8) { // Note: plain amiibo flags shifted 4 bits (original bits 0-3 are discarded)
        IsInitialized = bit!(0),
        HasApplicationArea = bit!(1),
        Unk_2 = bit!(2),
        Unk_3 = bit!(3)
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum ConsoleFamily {
    // Note: unofficial name
    #[default]
    Default = 0,
    NintendoWiiU = 1,
    Nintendo3DS = 2,
    NintendoSwitch = 3,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct AdminInfo {
    pub program_id: ncm::ProgramId,
    pub access_id: AccessId,
    pub crc32_change_counter: u16,
    pub flags: AdminInfoFlags,
    pub tag_type: u8,
    pub console_family: ConsoleFamily,
    pub pad: [u8; 0x7],
    pub reserved: [u8; 0x28],
}
const_assert!(core::mem::size_of::<AdminInfo>() == 0x40);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RegisterInfoPrivate {
    pub mii_store_data: mii::StoreData,
    pub first_write_date: Date,
    pub name: util::ArrayString<41>,
    pub unk: u8,
    pub reserved: [u8; 0x8E],
}
const_assert!(core::mem::size_of::<RegisterInfoPrivate>() == 0x100);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NfpData {
    // TODO: finish REing this type...
    pub magic: u8,
    pub maybe_pad: u8,
    pub write_counter: u8,
    pub maybe_pad_2: u8,
    pub settings_crc: u32,
    pub maybe_reserved: [u8; 56],
    pub last_write_date: Date,
    pub application_write_counter: u16,
    pub version: u16,
    pub app_area_size: u32,
    pub maybe_pad_3: [u8; 4],
    pub maybe_reserved_2: [u8; 0x30],
    pub mii_3ds_format: [u8; 0x60],
    pub unk_v5: [u8; 0x8],
    pub first_write_date: Date,
    pub name: util::ArrayWideString<11>,
    pub unk_v6: u8,
    pub unk_v7: u8,
    pub register_info_crc: u32,
    pub unk_v9: u64,
    pub unk_v10: u64,
    pub unk_v11: u32,
    pub maybe_reserved_3: [u8; 100],
    pub admin_info: AdminInfo,
    pub app_area: [u8; 0xD8],
}
const_assert!(core::mem::size_of::<NfpData>() == 0x298);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum BreakType {
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum WriteType {
    Unk0 = 0,
    Unk1 = 1,
}

ipc_sf_define_default_interface_client!(User);
ipc_sf_define_interface_trait! {
    trait User {
        initialize [0, version::VersionInterval::all()]: (aruid: applet::AppletResourceUserId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) =>  () ();
        initialize_2 [400, version::VersionInterval::all()]: (aruid: applet::AppletResourceUserId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) =>  () ();
        finalize [1, version::VersionInterval::all()]: () => () ();
        list_devices [2, version::VersionInterval::all()]: (out_devices: sf::OutPointerBuffer<DeviceHandle>) =>  (count: u32) (count: u32);
        start_detection [3, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        stop_detection [4, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        mount [5, version::VersionInterval::all()]: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) =>  () ();
        unmount [6, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        open_application_area [7, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId) =>  () ();
        get_application_area [8, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) =>  (size: u32) (size: u32);
        set_application_area [9, version::VersionInterval::all()]: (device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) =>  () ();
        flush [10, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        restore [11, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        create_application_area [12, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) =>  () ();
        get_tag_info [13, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) =>  () ();
        get_register_info [14, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) =>  () ();
        get_common_info [15, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) =>  () ();
        get_model_info [16, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) =>  () ();
        attach_activate_event [17, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        attach_deactivate_event [18, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        get_state [19, version::VersionInterval::all()]: () => (state: State) (state: State);
        get_device_state [20, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (device_state: DeviceState) (device_state: DeviceState);
        get_npad_id [21, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (npad_id: hid::NpadIdType) (npad_id: hid::NpadIdType);
        get_application_area_size [22, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (size: u32) (size: u32);
        attach_availability_change_event [23, version::VersionInterval::from(version::Version::new(3,0,0))]: () => (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        recreate_application_area [24, version::VersionInterval::from(version::Version::new(3,0,0))]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) =>  () ();
    }
}

ipc_sf_define_default_interface_client!(UserManager);
ipc_sf_define_interface_trait! {
    trait UserManager {
        create_user_interface [0, version::VersionInterval::all()]: () => (user_interface: User) (user_interface: session_type!(User));
    }
}

ipc_sf_define_default_interface_client!(System);
ipc_sf_define_interface_trait! {
    trait System {
        initialize_system [0, version::VersionInterval::all()]: (aruid: applet::AppletResourceUserId, _reserved: u64, mcu_data: sf::InMapAliasBuffer<McuVersionData>) =>  () ();
        finalize_system [1, version::VersionInterval::all()]: () => () ();
        list_devices [2, version::VersionInterval::all()]: (out_devices: sf::OutPointerBuffer<DeviceHandle>) =>  (count: u32) (count: u32);
        start_detection [3, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        stop_detection [4, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        mount [5, version::VersionInterval::all()]: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) =>  () ();
        unmount [6, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        flush [10, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        restore [11, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        get_tag_info [13, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) =>  () ();
        get_register_info [14, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) =>  () ();
        get_common_info [15, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) =>  () ();
        get_model_info [16, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) =>  () ();
        attach_activate_event [17, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        attach_deactivate_event [18, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        get_state [19, version::VersionInterval::all()]: () => (state: State) (state: State);
        get_device_state [20, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (device_state: DeviceState) (device_state: DeviceState);
        get_npad_id [21, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (npad_id: hid::NpadIdType) (npad_id: hid::NpadIdType);
        attach_availability_change_event [23, version::VersionInterval::all()]: () => (availability_change_event: sf::CopyHandle) (availability_change_event: sf::CopyHandle);
        format [100, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        get_admin_info [101, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) =>  () ();
        get_register_info_private [102, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) =>  () ();
        set_register_info_private [103, version::VersionInterval::all()]: (device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) =>  () ();
        delete_register_info [104, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        delete_application_area [105, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        exists_application_area [106, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (exists: bool) (exists: bool);
    }
}

ipc_sf_define_default_interface_client!(SystemManager);
ipc_sf_define_interface_trait! {
    trait SystemManager {
        create_system_interface [0, version::VersionInterval::all()]: () => (system_interface: System) (system_interface: session_type!(System));
    }
}

ipc_sf_define_default_interface_client!(Debug);
ipc_sf_define_interface_trait! {
    trait Debug {
        initialize_debug [0, version::VersionInterval::all()]: (aruid: applet::AppletResourceUserId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) =>  () ();
        finalize_debug [1, version::VersionInterval::all()]: () => () ();
        list_devices [2, version::VersionInterval::all()]: (out_devices: sf::OutPointerBuffer<DeviceHandle>) =>  (count: u32) (count: u32);
        start_detection [3, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        stop_detection [4, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        mount [5, version::VersionInterval::all()]: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) =>  () ();
        unmount [6, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        open_application_area [7, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId) =>  () ();
        get_application_area [8, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) =>  (size: u32) (size: u32);
        set_application_area [9, version::VersionInterval::all()]: (device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) =>  () ();
        flush [10, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        restore [11, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        create_application_area [12, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) =>  () ();
        get_tag_info [13, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) =>  () ();
        get_register_info [14, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) =>  () ();
        get_common_info [15, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) =>  () ();
        get_model_info [16, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) =>  () ();
        attach_activate_event [17, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        attach_deactivate_event [18, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        get_state [19, version::VersionInterval::all()]: () => (state: State) (state: State);
        get_device_state [20, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (device_state: DeviceState) (device_state: DeviceState);
        get_npad_id [21, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (npad_id: hid::NpadIdType) (npad_id: hid::NpadIdType);
        get_application_area_size [22, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (size: u32) (size: u32);
        attach_availability_change_event [23, version::VersionInterval::from(version::Version::new(3,0,0))]: () => (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        recreate_application_area [24, version::VersionInterval::from(version::Version::new(3,0,0))]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) =>  () ();
        format [100, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        get_admin_info [101, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) =>  () ();
        get_register_info_private [102, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) =>  () ();
        set_register_info_private [103, version::VersionInterval::all()]: (device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) =>  () ();
        delete_register_info [104, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        delete_application_area [105, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        exists_application_area [106, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  (exists: bool) (exists: bool);
        get_all [200, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_data: sf::OutFixedPointerBuffer<NfpData>) =>  () ();
        set_all [201, version::VersionInterval::all()]: (device_handle: DeviceHandle, data: sf::InFixedPointerBuffer<NfpData>) =>  () ();
        flush_debug [202, version::VersionInterval::all()]: (device_handle: DeviceHandle) =>  () ();
        break_tag [203, version::VersionInterval::all()]: (device_handle: DeviceHandle, break_type: BreakType) =>  () ();
        read_backup_data [204, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_buf: sf::OutMapAliasBuffer<u8>) =>  (read_size: u32) (read_size: u32);
        write_backup_data [205, version::VersionInterval::all()]: (device_handle: DeviceHandle, buf: sf::InMapAliasBuffer<u8>) =>  () ();
        write_ntf [206, version::VersionInterval::all()]: (device_handle: DeviceHandle, write_type: WriteType, buf: sf::InMapAliasBuffer<u8>) =>  () ();
    }
}

ipc_sf_define_default_interface_client!(DebugManager);
ipc_sf_define_interface_trait! {
    trait DebugManager {
        create_debug_interface [0, version::VersionInterval::all()]: () => (debug_interface: Debug) (debug_interface: session_type!(Debug));
    }
}
