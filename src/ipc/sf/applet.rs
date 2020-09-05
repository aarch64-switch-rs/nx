use crate::result::*;
use crate::ipc::sf;
use crate::mem;

pub type AppletResourceUserId = u64;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct AppletAttribute {
    flag: u8,
    reserved: [u8; 0x7F]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ScreenShotPermission {
    Inherit,
    Enable,
    Disable
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum AppletId {
    Application = 0x1,
    OverlayDisp = 0x2,
    Qlaunch = 0x3,
    Starter = 0x4,
    Auth = 0xA,
    Cabinet = 0xB,
    Controller = 0xC,
    DataErase = 0xD,
    Error = 0xE,
    NetConnect = 0xF,
    PlayerSelect = 0x10,
    Swkbd = 0x11,
    MiiEdit = 0x12,
    Web = 0x13,
    Shop = 0x14,
    PhotoViewer = 0x15,
    Set = 0x16,
    OfflineWeb = 0x17,
    LoginShare = 0x18,
    WifiWebAuth = 0x19,
    MyPage = 0x1A,
    // TODO: add non-retail IDs too?
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum LibraryAppletMode {
    AllForeground,
    Background,
    NoUi,
    BackgroundIndirectDisplay,
    AllForegroundInitiallyHidden,
}

pub trait IStorageAccessor {
    ipc_interface_define_command!(get_size: () => (size: usize));
    ipc_interface_define_command!(write: (offset: usize, buf: sf::InAutoSelectBuffer) => ());
    ipc_interface_define_command!(read: (offset: usize, buf: sf::OutAutoSelectBuffer) => ());
}

pub trait IStorage {
    ipc_interface_define_command!(open: () => (storage_accessor: mem::Shared<dyn sf::IObject>));
}

pub trait ILibraryAppletAccessor {
    ipc_interface_define_command!(get_applet_state_changed_event: () => (applet_state_changed_event: sf::CopyHandle));
    ipc_interface_define_command!(start: () => ());
    ipc_interface_define_command!(push_in_data: (storage: mem::Shared<dyn sf::IObject>) => ());
}

pub trait ILibraryAppletCreator {
    ipc_interface_define_command!(create_library_applet: (applet_id: AppletId, applet_mode: LibraryAppletMode) => (library_applet_accessor: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(create_storage: (size: usize) => (storage: mem::Shared<dyn sf::IObject>));
}

pub trait IWindowController {
    ipc_interface_define_command!(acquire_foreground_rights: () => ());
}

pub trait ISelfController {
    ipc_interface_define_command!(set_screenshot_permission: (permission: ScreenShotPermission) => ());
}

pub trait ILibraryAppletProxy {
    ipc_interface_define_command!(get_self_controller: () => (self_controller: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(get_window_controller: () => (window_controller: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(get_library_applet_creator: () => (library_applet_creator: mem::Shared<dyn sf::IObject>));
}

pub trait IAllSystemAppletProxiesService {
    ipc_interface_define_command!(open_library_applet_proxy: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle, applet_attribute: sf::InMapAliasBuffer) => (library_applet_proxy: mem::Shared<dyn sf::IObject>));
}