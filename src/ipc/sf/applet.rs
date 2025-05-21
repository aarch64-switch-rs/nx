use crate::ipc::sf;
use crate::result::*;
use crate::svc::Handle;
use crate::version;

pub use super::AppletResourceUserId;

use nx_derive::{Request, Response};

#[derive(Request, Response, Copy, Clone)]
#[repr(C)]
pub struct AppletAttribute {
    flag: u8,
    reserved: [u8; 0x7F],
}

impl AppletAttribute {
    pub const fn zero() -> Self {
        Self {
            flag: 0,
            reserved: [0u8; 0x7F],
        }
    }
}

impl Default for AppletAttribute {
    fn default() -> Self {
        Self::zero()
    }
}

const_assert!(core::mem::size_of::<AppletAttribute>() == 0x80);

#[derive(Request, Response, Copy, Clone)]
#[repr(C)]
pub struct AppletProcessLaunchReason {
    flag: u8,
    _zero: [u8; 2],
    _zero2: u8,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ScreenShotPermission {
    Inherit,
    Enable,
    Disable,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
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
    AppletILA2 = 0x3FE,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum OperationMode {
    Handheld = 0,
    Console = 1,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum PerformanceMode {
    Invalid = -1,
    Normal = 0,
    Boost = 1,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum LibraryAppletMode {
    AllForeground,
    Background,
    NoUi,
    BackgroundIndirectDisplay,
    AllForegroundInitiallyHidden,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub enum AppletMessage {
    /// The applet received an exit request
    ExitRequest = 4,
    /// The FocusState of the applet changed
    FocusStateChanged = 15,
    /// The applet was resumed
    Resume = 16,
    /// The OperationMode of the applet changed
    OperationModeChanged = 30,
    /// The PerformanceMode of the applet changed
    PerformanceMode = 31,
    /// The applet was requested to display
    DisplayRequested = 51,
    /// Capture button was pressed (short)
    CaptureButtonPressedShort = 90,
    /// A screenshot was taken
    ScreenShotTaken = 92,
    /// A screen recoding was saved
    RecordingSaved = 93,
}

ipc_sf_define_default_client_for_interface!(StorageAccessor);
ipc_sf_define_interface_trait! {
    trait StorageAccessor {
        get_size [0, version::VersionInterval::all()]: () => (size: usize) (size: usize);
        write [10, version::VersionInterval::all()]: (offset: usize, buf: sf::InAutoSelectBuffer<u8>) =>  () ();
        read [11, version::VersionInterval::all()]: (offset: usize, buf: sf::OutAutoSelectBuffer<u8>) =>  () ();
    }
}

ipc_sf_define_default_client_for_interface!(Storage);
ipc_sf_define_interface_trait! {
    trait Storage {
        open [0, version::VersionInterval::all()]: () => (storage_accessor: StorageAccessor) (storage_accessor: StorageAccessor);
    }
}

ipc_sf_define_default_client_for_interface!(LibraryAppletAccessor);
ipc_sf_define_interface_trait! {
    trait LibraryAppletAccessor {
        get_applet_state_changed_event [0, version::VersionInterval::all(), mut]: () => (applet_state_changed_event: sf::CopyHandle) (applet_state_changed_event: sf::CopyHandle);
        start [10, version::VersionInterval::all(), mut]: () => () ();
        push_in_data [100, version::VersionInterval::all(), mut]: (storage: Storage) =>  () ();
        pop_out_data [101, version::VersionInterval::all(), mut]: () => (storage: Storage) (storage: Storage);
    }
}

ipc_sf_define_default_client_for_interface!(LibraryAppletCreator);
ipc_sf_define_interface_trait! {
    trait LibraryAppletCreator {
        create_library_applet [0, version::VersionInterval::all()]: (applet_id: AppletId, applet_mode: LibraryAppletMode) =>  (library_applet_accessor: LibraryAppletAccessor) (library_applet_accessor: session_type!(LibraryAppletAccessor));
        create_storage [10, version::VersionInterval::all()]: (size: usize) =>  (storage: Storage) (storage: Storage);
    }
}

ipc_sf_define_default_client_for_interface!(WindowController);
ipc_sf_define_interface_trait! {
    trait WindowController {
        get_applet_resource_user_id [1, version::VersionInterval::all()]: () => (aruid: u64) (aruid: u64);
        acquire_foreground_rights [10, version::VersionInterval::all()]: () => () ();
    }
}

ipc_sf_define_default_client_for_interface!(SelfController);
ipc_sf_define_interface_trait! {
    trait SelfController {
        set_screenshot_permission [10, version::VersionInterval::all()]: (permission: ScreenShotPermission) =>  () ();
        create_managed_display_layer [40, version::VersionInterval::all()]: () =>  (layer_id: u64) (layer_id: u64);
    }
}

ipc_sf_define_default_client_for_interface!(AudioController);
ipc_sf_define_interface_trait! {
    trait AudioController {
        set_expected_master_volume [0, version::VersionInterval::all()]: (main_applet_level: f32, library_applet_level: f32) =>  () ();
        get_main_applet_volume [1, version::VersionInterval::all()]: () => (main_applet_level: f32) (main_applet_level: f32);
        get_library_applet_volume [2, version::VersionInterval::all()]: () => (library_applet_level: f32) (library_applet_level: f32);
        change_main_applet_volume [3, version::VersionInterval::all()]: (main_applet_level: f32, unknown: u64) => () ();
        set_transparent_volume_rate [4, version::VersionInterval::all()]: (rate: f32) => () ();
    }
}

ipc_sf_define_default_client_for_interface!(DisplayController);
ipc_sf_define_interface_trait! {
    trait DisplayController {
        update_caller_applet_capture_image [3, version::VersionInterval::all()]: () =>  () ();
    }
}
ipc_sf_define_default_client_for_interface!(ProcessWindingController);
ipc_sf_define_interface_trait! {
    trait ProcessWindingController {
        get_launch_reason [0, version::VersionInterval::all()]: () => (launch_reason: AppletProcessLaunchReason) (launch_reason: AppletProcessLaunchReason);
    }
}

ipc_sf_define_default_client_for_interface!(CommonStateGetter);
ipc_sf_define_interface_trait! {
    trait CommonStateGetter {
        get_event_handle [0, version::VersionInterval::all()]: () => (handle: Handle) (handle: Handle);
        receive_message [1, version::VersionInterval::all()]: () => (applet_message: AppletMessage) (applet_message: AppletMessage);
        }
}

ipc_sf_define_default_client_for_interface!(LibraryAppletProxy);
ipc_sf_define_interface_trait! {
    trait LibraryAppletProxy {
        get_common_state_getter [0, version::VersionInterval::all()]: () => (common_state_getter: CommonStateGetter) (commond_state_getter: session_type!(CommonStateGetter));
        get_self_controller [1, version::VersionInterval::all()]: () => (self_controller: SelfController) (self_controller: session_type!(SelfController));
        get_window_controller [2, version::VersionInterval::all()]: () => (window_controller: WindowController) (window_controller: session_type!(WindowController));
        get_audio_controller [3, version::VersionInterval::all()]: () => (window_controller: AudioController) (window_controller: session_type!(AudioController));
        get_display_controller [4, version::VersionInterval::all()]: () => (window_controller: DisplayController) (window_controller: session_type!(DisplayController));
        get_process_winding_controller [10, version::VersionInterval::all()]: () => (window_controller: ProcessWindingController) (window_controller: session_type!(ProcessWindingController));
        get_library_applet_creator [11, version::VersionInterval::all()]: () => (library_applet_creator: LibraryAppletCreator) (library_applet_creator: session_type!(AllSystemAppletProxiesService));
    }
}

ipc_sf_define_default_client_for_interface!(ApplicationProxy);
ipc_sf_define_interface_trait! {
    trait ApplicationProxy {
        get_common_state_getter [0, version::VersionInterval::all()]: () => (common_state_getter: CommonStateGetter) (commond_state_getter: session_type!(CommonStateGetter));
        get_self_controller [1, version::VersionInterval::all()]: () => (self_controller: SelfController) (self_controller: session_type!(SelfController));
        get_window_controller [2, version::VersionInterval::all()]: () => (window_controller: WindowController) (window_controller: session_type!(WindowController));
        get_audio_controller [3, version::VersionInterval::all()]: () => (window_controller: AudioController) (window_controller: session_type!(AudioController));
        get_display_controller [4, version::VersionInterval::all()]: () => (window_controller: DisplayController) (window_controller: session_type!(DisplayController));
        get_process_winding_controller [10, version::VersionInterval::all()]: () => (window_controller: ProcessWindingController) (window_controller: session_type!(ProcessWindingController));
        get_library_applet_creator [11, version::VersionInterval::all()]: () => (library_applet_creator: LibraryAppletCreator) (library_applet_creator: session_type!(AllSystemAppletProxiesService));
    }
}

ipc_sf_define_default_client_for_interface!(SystemAppletProxy);
ipc_sf_define_interface_trait! {
    trait SystemAppletProxy {
        get_common_state_getter [0, version::VersionInterval::all()]: () => (common_state_getter: CommonStateGetter) (commond_state_getter: session_type!(CommonStateGetter));
        get_self_controller [1, version::VersionInterval::all()]: () => (self_controller: SelfController) (self_controller: session_type!(SelfController));
        get_window_controller [2, version::VersionInterval::all()]: () => (window_controller: WindowController) (window_controller: session_type!(WindowController));
        get_audio_controller [3, version::VersionInterval::all()]: () => (window_controller: AudioController) (window_controller: session_type!(AudioController));
        get_display_controller [4, version::VersionInterval::all()]: () => (window_controller: DisplayController) (window_controller: session_type!(DisplayController));
        get_process_winding_controller [10, version::VersionInterval::all()]: () => (window_controller: ProcessWindingController) (window_controller: session_type!(ProcessWindingController));
        get_library_applet_creator [11, version::VersionInterval::all()]: () => (library_applet_creator: LibraryAppletCreator) (library_applet_creator: session_type!(AllSystemAppletProxiesService));
    }
}

ipc_sf_define_default_client_for_interface!(OverlayAppletProxy);
ipc_sf_define_interface_trait! {
    trait OverlayAppletProxy {
        get_common_state_getter [0, version::VersionInterval::all()]: () => (common_state_getter: CommonStateGetter) (commond_state_getter: session_type!(CommonStateGetter));
        get_self_controller [1, version::VersionInterval::all()]: () => (self_controller: SelfController) (self_controller: session_type!(SelfController));
        get_window_controller [2, version::VersionInterval::all()]: () => (window_controller: WindowController) (window_controller: session_type!(WindowController));
        get_audio_controller [3, version::VersionInterval::all()]: () => (window_controller: AudioController) (window_controller: session_type!(AudioController));
        get_display_controller [4, version::VersionInterval::all()]: () => (window_controller: DisplayController) (window_controller: session_type!(DisplayController));
        get_process_winding_controller [10, version::VersionInterval::all()]: () => (window_controller: ProcessWindingController) (window_controller: session_type!(ProcessWindingController));
        get_library_applet_creator [11, version::VersionInterval::all()]: () => (library_applet_creator: LibraryAppletCreator) (library_applet_creator: session_type!(AllSystemAppletProxiesService));
    }
}

ipc_sf_define_default_client_for_interface!(SystemApplicationProxy);
ipc_sf_define_interface_trait! {
    trait SystemApplicationProxy {
        get_common_state_getter [0, version::VersionInterval::all()]: () => (common_state_getter: CommonStateGetter) (commond_state_getter: session_type!(CommonStateGetter));
        get_self_controller [1, version::VersionInterval::all()]: () => (self_controller: SelfController) (self_controller: session_type!(SelfController));
        get_window_controller [2, version::VersionInterval::all()]: () => (window_controller: WindowController) (window_controller: session_type!(WindowController));
        get_audio_controller [3, version::VersionInterval::all()]: () => (window_controller: AudioController) (window_controller: session_type!(AudioController));
        get_display_controller [4, version::VersionInterval::all()]: () => (window_controller: DisplayController) (window_controller: session_type!(DisplayController));
        get_process_winding_controller [10, version::VersionInterval::all()]: () => (window_controller: ProcessWindingController) (window_controller: session_type!(ProcessWindingController));
        get_library_applet_creator [11, version::VersionInterval::all()]: () => (library_applet_creator: LibraryAppletCreator) (library_applet_creator: session_type!(AllSystemAppletProxiesService));
    }
}

ipc_sf_define_default_client_for_interface!(AllSystemAppletProxiesService);
ipc_sf_define_interface_trait! {
    trait AllSystemAppletProxiesService {
        open_application_proxy [0, version::VersionInterval::all()]: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle) =>  (library_applet_proxy: ApplicationProxy) (library_applet_proxy: session_type!(ApplicationProxy));
        open_system_applet_proxy [100, version::VersionInterval::all()]: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle) =>  (library_applet_proxy: SystemAppletProxy) (library_applet_proxy: session_type!(SystemAppletProxy));
        open_library_applet_proxy_old [200, version::VersionInterval::from(version::Version::new(3,0,0))]: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle) =>  (library_applet_proxy: LibraryAppletProxy) (library_applet_proxy: session_type!(LibraryAppletProxy));
        open_library_applet_proxy [201, version::VersionInterval::from(version::Version::new(3,0,0))]: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle, applet_attribute: sf::InMapAliasBuffer<AppletAttribute>) =>  (library_applet_proxy: LibraryAppletProxy) (library_applet_proxy: session_type!(LibraryAppletProxy));
        open_overlay_applet_proxy [300, version::VersionInterval::all()]: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle) =>  (library_applet_proxy: OverlayAppletProxy) (library_applet_proxy: session_type!(OverlayAppletProxy));
        open_system_application_proxy [350, version::VersionInterval::all()]: (process_id: sf::ProcessId, self_process_handle: sf::CopyHandle) =>  (library_applet_proxy: SystemApplicationProxy) (library_applet_proxy: session_type!(SystemApplicationProxy));
    }
}

pub trait ProxyCommon {
    fn get_common_state_getter(&self) -> Result<CommonStateGetter>;
    fn get_self_controller(&self) -> Result<SelfController>;
    fn get_window_controller(&self) -> Result<WindowController>;
    fn get_audio_controller(&self) -> Result<AudioController>;
    fn get_display_controller(&self) -> Result<DisplayController>;
    fn get_process_winding_controller(&self) -> Result<ProcessWindingController>;
    fn get_library_applet_creator(&self) -> Result<LibraryAppletCreator>;
}
