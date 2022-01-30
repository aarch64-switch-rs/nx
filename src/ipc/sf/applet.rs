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
const_assert!(core::mem::size_of::<AppletAttribute>() == 0x80);

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
    None = 0x00,
    Application = 0x01,
    OverlayApplet = 0x02,
    SystemAppletMenu = 0x03,
    SystemApplication = 0x04,
    LibraryAppletAuth = 0x0A,
    LibraryAppletCabinet = 0x0B,
    LibraryAppletController = 0x0C,
    LibraryAppletDataErase = 0x0D,
    LibraryAppletError = 0x0E,
    LibraryAppletNetConnect = 0x0F,
    LibraryAppletPlayerSelect = 0x10,
    LibraryAppletSwkbd = 0x11,
    LibraryAppletMiiEdit = 0x12,
    LibAppletWeb = 0x13,
    LibAppletShop = 0x14,
    LibraryAppletPhotoViewer = 0x15,
    LibraryAppletSet = 0x16,
    LibraryAppletOfflineWeb = 0x17,
    LibraryAppletLoginShare = 0x18,
    LibraryAppletWifiWebAuth = 0x19,
    LibraryAppletMyPage = 0x1A,
    LibraryAppletGift = 0x1B,
    LibraryAppletUserMigration = 0x1C,
    LibraryAppletPreomiaSys = 0x1D,
    LibraryAppletStory = 0x1E,
    LibraryAppletPreomiaUsr = 0x1F,
    LibraryAppletPreomiaUsrDummy = 0x20,
    LibraryAppletSample = 0x21,
    DevlopmentTool = 0x3E8,
    CombinationLA = 0x3F1,
    AeSystemApplet = 0x3F2,
    AeOverlayApplet = 0x3F3,
    AeStarter = 0x3F4,
    AeLibraryAppletAlone = 0x3F5,
    AeLibraryApplet1 = 0x3F6,
    AeLibraryApplet2 = 0x3F7,
    AeLibraryApplet3 = 0x3F8,
    AeLibraryApplet4 = 0x3F9,
    AppletISA = 0x3FA,
    AppletIOA = 0x3FB,
    AppletISTA = 0x3FC,
    AppletILA1 = 0x3FD,
    AppletILA2 = 0x3FE
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
    ipc_cmif_interface_define_command!(get_size: () => (size: usize));
    ipc_cmif_interface_define_command!(write: (offset: usize, buf: sf::InAutoSelectBuffer) => ());
    ipc_cmif_interface_define_command!(read: (offset: usize, buf: sf::OutAutoSelectBuffer) => ());
}

pub trait IStorage {
    ipc_cmif_interface_define_command!(open: () => (storage_accessor: mem::Shared<dyn sf::IObject>));
}

pub trait ILibraryAppletAccessor {
    ipc_cmif_interface_define_command!(get_applet_state_changed_event: () => (applet_state_changed_event: sf::CopyHandle));
    ipc_cmif_interface_define_command!(start: () => ());
    ipc_cmif_interface_define_command!(push_in_data: (storage: mem::Shared<dyn sf::IObject>) => ());
}

pub trait ILibraryAppletCreator {
    ipc_cmif_interface_define_command!(create_library_applet: (applet_id: AppletId, applet_mode: LibraryAppletMode) => (library_applet_accessor: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(create_storage: (size: usize) => (storage: mem::Shared<dyn sf::IObject>));
}

pub trait IWindowController {
    ipc_cmif_interface_define_command!(acquire_foreground_rights: () => ());
}

pub trait ISelfController {
    ipc_cmif_interface_define_command!(set_screenshot_permission: (permission: ScreenShotPermission) => ());
}

pub trait ILibraryAppletProxy {
    ipc_cmif_interface_define_command!(get_self_controller: () => (self_controller: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(get_window_controller: () => (window_controller: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(get_library_applet_creator: () => (library_applet_creator: mem::Shared<dyn sf::IObject>));
}

pub trait IAllSystemAppletProxiesService {
    ipc_cmif_interface_define_command!(open_library_applet_proxy: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle, applet_attribute: sf::InMapAliasBuffer) => (library_applet_proxy: mem::Shared<dyn sf::IObject>));
}