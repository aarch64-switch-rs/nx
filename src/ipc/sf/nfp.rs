use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;
use crate::ipc::sf::applet;
use crate::ipc::sf::mii;
use crate::version;

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

ipc_sf_define_interface_trait! {
    trait IUser {
        initialize [0, version::VersionInterval::all()]: (aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) => ();
        finalize [1, version::VersionInterval::all()]: () => ();
        list_devices [2, version::VersionInterval::all()]: (out_devices: sf::OutPointerBuffer<DeviceHandle>) => (count: u32);
        start_detection [3, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        stop_detection [4, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        mount [5, version::VersionInterval::all()]: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) => ();
        unmount [6, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        open_application_area [7, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId) => ();
        get_application_area [8, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) => (size: u32);
        set_application_area [9, version::VersionInterval::all()]: (device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) => ();
        flush [10, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        restore [11, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        create_application_area [12, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ();
        get_tag_info [13, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) => ();
        get_register_info [14, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) => ();
        get_common_info [15, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) => ();
        get_model_info [16, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) => ();
        attach_activate_event [17, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (activate_event: sf::CopyHandle);
        attach_deactivate_event [18, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (deactivate_event: sf::CopyHandle);
        get_state [19, version::VersionInterval::all()]: () => (state: State);
        get_device_state [20, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (device_state: DeviceState);
        get_npad_id [21, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (npad_id: u32);
        get_application_area_size [22, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (size: u32);
        attach_availability_change_event [23, version::VersionInterval::from(version::Version::new(3,0,0))]: () => (availability_change_event: sf::CopyHandle);
        recreate_application_area [24, version::VersionInterval::from(version::Version::new(3,0,0))]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ();
    }
}

ipc_sf_define_interface_trait! {
    trait IUserManager {
        create_user_interface [0, version::VersionInterval::all()]: () => (user_interface: mem::Shared<dyn IUser>);
    }
}

ipc_sf_define_interface_trait! {
    trait ISystem {
        initialize_system [0, version::VersionInterval::all()]: (aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) => ();
        finalize_system [1, version::VersionInterval::all()]: () => ();
        list_devices [2, version::VersionInterval::all()]: (out_devices: sf::OutPointerBuffer<DeviceHandle>) => (count: u32);
        start_detection [3, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        stop_detection [4, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        mount [5, version::VersionInterval::all()]: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) => ();
        unmount [6, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        flush [10, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        restore [11, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        get_tag_info [13, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) => ();
        get_register_info [14, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) => ();
        get_common_info [15, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) => ();
        get_model_info [16, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) => ();
        attach_activate_event [17, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (activate_event: sf::CopyHandle);
        attach_deactivate_event [18, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (deactivate_event: sf::CopyHandle);
        get_state [19, version::VersionInterval::all()]: () => (state: State);
        get_device_state [20, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (device_state: DeviceState);
        get_npad_id [21, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (npad_id: u32);
        attach_availability_change_event [23, version::VersionInterval::all()]: () => (availability_change_event: sf::CopyHandle);
        format [100, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        get_admin_info [101, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) => ();
        get_register_info_private [102, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) => ();
        set_register_info_private [103, version::VersionInterval::all()]: (device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) => ();
        delete_register_info [104, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        delete_application_area [105, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        exists_application_area [106, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (exists: bool);
    }
}

ipc_sf_define_interface_trait! {
    trait ISystemManager {
        create_system_interface [0, version::VersionInterval::all()]: () => (system_interface: mem::Shared<dyn ISystem>);
    }
}

ipc_sf_define_interface_trait! {
    trait IDebug {
        initialize_debug [0, version::VersionInterval::all()]: (aruid: applet::AppletResourceUserId, process_id: sf::ProcessId, mcu_data: sf::InMapAliasBuffer<McuVersionData>) => ();
        finalize_debug [1, version::VersionInterval::all()]: () => ();
        list_devices [2, version::VersionInterval::all()]: (out_devices: sf::OutPointerBuffer<DeviceHandle>) => (count: u32);
        start_detection [3, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        stop_detection [4, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        mount [5, version::VersionInterval::all()]: (device_handle: DeviceHandle, model_type: ModelType, mount_target: MountTarget) => ();
        unmount [6, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        open_application_area [7, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId) => ();
        get_application_area [8, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_data: sf::OutMapAliasBuffer<u8>) => (size: u32);
        set_application_area [9, version::VersionInterval::all()]: (device_handle: DeviceHandle, data: sf::InMapAliasBuffer<u8>) => ();
        flush [10, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        restore [11, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        create_application_area [12, version::VersionInterval::all()]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ();
        get_tag_info [13, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_tag_info: sf::OutFixedPointerBuffer<TagInfo>) => ();
        get_register_info [14, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info: sf::OutFixedPointerBuffer<RegisterInfo>) => ();
        get_common_info [15, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_common_info: sf::OutFixedPointerBuffer<CommonInfo>) => ();
        get_model_info [16, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_model_info: sf::OutFixedPointerBuffer<ModelInfo>) => ();
        attach_activate_event [17, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (activate_event: sf::CopyHandle);
        attach_deactivate_event [18, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (deactivate_event: sf::CopyHandle);
        get_state [19, version::VersionInterval::all()]: () => (state: State);
        get_device_state [20, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (device_state: DeviceState);
        get_npad_id [21, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (npad_id: u32);
        get_application_area_size [22, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (size: u32);
        attach_availability_change_event [23, version::VersionInterval::from(version::Version::new(3,0,0))]: () => (availability_change_event: sf::CopyHandle);
        recreate_application_area [24, version::VersionInterval::from(version::Version::new(3,0,0))]: (device_handle: DeviceHandle, access_id: AccessId, data: sf::InMapAliasBuffer<u8>) => ();
        format [100, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        get_admin_info [101, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_admin_info: sf::OutFixedPointerBuffer<AdminInfo>) => ();
        get_register_info_private [102, version::VersionInterval::all()]: (device_handle: DeviceHandle, out_register_info_private: sf::OutFixedPointerBuffer<RegisterInfoPrivate>) => ();
        set_register_info_private [103, version::VersionInterval::all()]: (device_handle: DeviceHandle, register_info_private: sf::InFixedPointerBuffer<RegisterInfoPrivate>) => ();
        delete_register_info [104, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        delete_application_area [105, version::VersionInterval::all()]: (device_handle: DeviceHandle) => ();
        exists_application_area [106, version::VersionInterval::all()]: (device_handle: DeviceHandle) => (exists: bool);
        // TODO: remaining commands
    }
}

ipc_sf_define_interface_trait! {
    trait IDebugManager {
        create_debug_interface [0, version::VersionInterval::all()]: () => (debug_interface: mem::Shared<dyn IDebug>);
    }
}