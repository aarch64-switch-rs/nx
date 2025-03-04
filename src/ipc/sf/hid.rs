use crate::ipc::sf;
use crate::result::*;
use crate::svc;
use crate::version;

use super::applet::AppletResourceUserId;

pub mod shmem;
use nx_derive::{Request, Response};

define_bit_enum! {
    DebugPadAttribute (u32) {
        IsConnected = bit!(0)
    }
}

define_bit_enum! {
    DebugPadButton (u32) {
        A = bit!(0),
        B = bit!(1),
        X = bit!(2),
        Y = bit!(3),
        L = bit!(4),
        R = bit!(5),
        ZL = bit!(6),
        ZR = bit!(7),
        Start = bit!(8),
        Select = bit!(9),
        Left = bit!(10),
        Up = bit!(11),
        Right = bit!(12),
        Down = bit!(13)
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct AnalogStickState {
    pub x: i32,
    pub y: i32,
}

define_bit_enum! {
    TouchAttribute (u32) {
        None = 0,
        Start = bit!(0),
        End = bit!(1)
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct TouchState {
    pub delta_time: u64,
    pub attributes: TouchAttribute,
    pub finger_id: u32,
    pub x: u32,
    pub y: u32,
    pub diameter_x: u32,
    pub diameter_y: u32,
    pub rotation_angle: u32,
    pub reserved: [u8; 4],
}

define_bit_enum! {
    MouseAttribute (u32) {
        Transferable = bit!(0),
        IsConnected = bit!(1)
    }
}

define_bit_enum! {
    MouseButton (u32) {
        Left = bit!(0),
        Right = bit!(1),
        Middle = bit!(2),
        Forward = bit!(3),
        Back = bit!(4)
    }
}

define_bit_enum! {
    KeyboardModifier (u32) {
        Control = bit!(0),
        Shift = bit!(1),
        LeftAlt = bit!(2),
        RightAlt = bit!(3),
        Gui = bit!(4),
        CapsLock = bit!(8),
        ScrollLock = bit!(9),
        NumLock = bit!(10),
        Katakana = bit!(11),
        Hiragana = bit!(12)
    }
}

/*
define_bit_enum! {
    KeyboardKey (u32) {
        // TODO (is 256-bit, not 32-bit...): https://switchbrew.org/wiki/HID_services#KeyboardKey
    }
}
*/

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct KeyboardKey {
    pub todo: [u8; 0x20],
}

define_bit_enum! {
    BasicXpadAttribute (u32) {
        // TODO: are these known at all?
    }
}

define_bit_enum! {
    BasicXpadButton (u32) {
        // TODO: are these known at all?
    }
}

define_bit_enum! {
    DigitizerAttribute (u32) {
        // TODO: are these known at all?
    }
}

define_bit_enum! {
    DigitizerButton (u32) {
        // TODO: are these known at all?
    }
}

define_bit_enum! {
    HomeButton (u32) {
        // TODO: are these known at all?
    }
}

define_bit_enum! {
    SleepButton (u32) {
        // TODO: are these known at all?
    }
}

define_bit_enum! {
    CaptureButton (u32) {
        // TODO: are these known at all?
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InputSourceState {
    pub timestamp: u64,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum UniquePadType {
    Embedded = 0,
    FullKeyController = 1,
    RightController = 2,
    LeftController = 3,
    DebugPadController = 4,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum UniquePadInterface {
    Embedded = 0,
    Rail = 1,
    Bluetooth = 2,
    Usb = 3,
}

pub type UniquePadSerialNumber = [u8; 0x10];

define_bit_enum! {
    AnalogStickCalibrationFlags (u32) {
        // TODO: are these known at all?
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum AnalogStickManualCalibrationStage {
    ReleaseFromRight = 0,
    ReleaseFromBottom = 1,
    ReleaseFromLeft = 2,
    ReleaseFromTop = 3,
    Rotate = 4,
    Update = 5,
    Completed = 6,
    Clear = 7,
    ClearCompleted = 8,
}

define_bit_enum! {
    SixAxisSensorUserCalibrationFlags (u32) {
        // TODO: are these known at all?
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum SixAxisSensorUserCalibrationStage {
    Measuring = 0,
    Update = 1,
    Completed = 2,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum GestureType {
    Idle = 0,
    Complete = 1,
    Cancel = 2,
    Touch = 3,
    Press = 4,
    Tap = 5,
    Pan = 6,
    Swipe = 7,
    Pinch = 8,
    Rotate = 9,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum GestureDirection {
    None = 0,
    Left = 1,
    Up = 2,
    Right = 3,
    Down = 4,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct GesturePoint {
    pub x: u32,
    pub y: u32,
}

define_bit_enum! {
    GestureAttribute (u32) {
        IsNewTouch = bit!(4),
        IsDoubleTap = bit!(8)
    }
}

define_bit_enum! {
    NpadStyleTag (u32) {
        FullKey = bit!(0), // Pro controller
        Handheld = bit!(1), // Joy-Con controller in handheld mode
        JoyDual = bit!(2), // Joy-Con controller in dual mode
        JoyLeft = bit!(3), // Joy-Con left controller in single mode
        JoyRight = bit!(4), // Joy-Con right controller in single mode
        Gc = bit!(5), // GameCube controller
        Palma = bit!(6), // Poké Ball Plus controller
        Lark = bit!(7), // NES/Famicom controller
        HandheldLark = bit!(8), // NES/Famicom controller (handheld)
        Lucia = bit!(9), // SNES controller
        Lagon = bit!(10), // N64 controller
        Lager = bit!(11), // Sega Genesis controller
        SystemExt = bit!(29), // Generic external controller
        System = bit!(30) // Generic controller
    }
}
/*
#[atomic_enum]
#[repr(u32)]
#[derive(crate::ipc::sf::Request, crate::ipc::sf::Response, PartialEq, Eq, Default)]
pub enum NpadStyleTag {
    #[default]
    None = 0, // The flag was not set for some reason
    FullKey = bit!(0), // Pro controller
    Handheld = bit!(1), // Joy-Con controller in handheld mode
    JoyDual = bit!(2), // Joy-Con controller in dual mode
    JoyLeft = bit!(3), // Joy-Con left controller in single mode
    JoyRight = bit!(4), // Joy-Con right controller in single mode
    Gc = bit!(5), // GameCube controller
    Palma = bit!(6), // Poké Ball Plus controller
    Lark = bit!(7), // NES/Famicom controller
    HandheldLark = bit!(8), // NES/Famicom controller (handheld)
    Lucia = bit!(9), // SNES controller
    Lagon = bit!(10), // N64 controller
    Lager = bit!(11), // Sega Genesis controller
    SystemExt = bit!(29), // Generic external controller
    System = bit!(30) // Generic controller
}
*/
#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i64)]
pub enum NpadJoyDeviceType {
    Left = 0,
    Right = 1,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum NpadIdType {
    No1 = 0, // Players 1-8
    No2 = 1,
    No3 = 2,
    No4 = 3,
    No5 = 4,
    No6 = 5,
    No7 = 6,
    No8 = 7,
    Other = 0x10,
    Handheld = 0x20,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum NpadJoyAssignmentMode {
    Dual = 0,
    Single = 1,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ColorAttribute {
    Ok = 0,
    ReadError = 1,
    NoController = 2,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadControllerColor {
    pub main: u32,
    pub sub: u32,
}

define_bit_enum! {
    NpadButton (u64) {
        A = bit!(0),
        B = bit!(1),
        X = bit!(2),
        Y = bit!(3),
        StickL = bit!(4),
        StickR = bit!(5),
        L = bit!(6),
        R = bit!(7),
        ZL = bit!(8),
        ZR = bit!(9),
        Plus = bit!(10),
        Minus = bit!(11),
        Left = bit!(12),
        Up = bit!(13),
        Right = bit!(14),
        Down = bit!(15),
        StickLLeft = bit!(16),
        StickLUp = bit!(17),
        StickLRight = bit!(18),
        StickLDown = bit!(19),
        StickRLeft = bit!(20),
        StickRUp = bit!(21),
        StickRRight = bit!(22),
        StickRDown = bit!(23),
        LeftSL = bit!(24),
        LeftSR = bit!(25),
        RightSL = bit!(26),
        RightSR = bit!(27),
        Palma = bit!(28),
        Verification = bit!(29),
        HandheldLeftB = bit!(30),
        LagonCLeft = bit!(31),
        LagonCUp = bit!(32),
        LagonCRight = bit!(33),
        LagonCDown = bit!(34)
    }
}

define_bit_enum! {
    NpadAttribute (u32) {
        IsConnected = bit!(0),
        IsWired = bit!(1),
        IsLeftConnected = bit!(2),
        IsLeftWired = bit!(3),
        IsRightConnected = bit!(4),
        IsRightWired = bit!(5)
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DirectionState {
    pub xx: u32,
    pub xy: u32,
    pub xz: u32,
    pub yx: u32,
    pub yy: u32,
    pub yz: u32,
    pub zx: u32,
    pub zy: u32,
    pub zz: u32,
}

define_bit_enum! {
    SixAxisSensorAttribute (u32) {
        IsConnected = bit!(0),
        IsInterpolated = bit!(1)
    }
}

define_bit_enum! {
    DeviceType (u32) {
        FullKey = bit!(0),
        DebugPad = bit!(1),
        HandheldLeft = bit!(2),
        HandheldRight = bit!(3),
        JoyLeft = bit!(4),
        JoyRight = bit!(5),
        Palma = bit!(6),
        LarkHvcLeft = bit!(7),
        LarkHvcRight = bit!(8),
        LarkNesLeft = bit!(9),
        LarkNesRight = bit!(10),
        HandheldLarkHvcLeft = bit!(11),
        HandheldLarkHvcRight = bit!(12),
        HandheldLarkNesLeft = bit!(13),
        HandheldLarkNesRight = bit!(14),
        Lucia = bit!(15),
        Lagon = bit!(16),
        Lager = bit!(17),
        System = bit!(31)
    }
}

define_bit_enum! {
    NpadSystemProperties (u64) {
        IsChargingJoyDual = bit!(0),
        IsChargingJoyLeft = bit!(1),
        IsChargingJoyRight = bit!(2),
        IsPoweredJoyDual = bit!(3),
        IsPoweredJoyLeft = bit!(4),
        IsPoweredJoyRight = bit!(5),
        IsUnsuportedButtonPressedOnNpadSystem = bit!(9),
        IsUnsuportedButtonPressedOnNpadSystemExt = bit!(10),
        IsAbxyButtonOriented = bit!(11),
        IsSlSrButtonOriented = bit!(12),
        IsPlusAvailable = bit!(13),
        IsMinusAvailable = bit!(14),
        IsDirectionalButtonsAvailable = bit!(15)
    }
}

define_bit_enum! {
    NpadSystemButtonProperties (u32) {
        IsUnintendedHomeButtonInputProtectionEnabled = bit!(0)
    }
}

pub type NpadBatteryLevel = u32;

define_bit_enum! {
    AppletFooterUiAttribute (u32) {
        // TODO: are these known at all?
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum AppletFooterUiType {
    None = 0,
    HandheldNone = 1,
    HandheldJoyConLeftOnly = 2,
    HandheldJoyConRightOnly = 3,
    HandheldJoyConLeftJoyConRight = 4,
    JoyDual = 5,
    JoyDualLeftOnly = 6,
    JoyDualRightOnly = 7,
    JoyLeftHorizontal = 8,
    JoyLeftVertical = 9,
    JoyRightHorizontal = 10,
    JoyRightVertical = 11,
    SwitchProController = 12,
    CompatibleProController = 13,
    CompatibleJoyCon = 14,
    LarkHvc1 = 15,
    LarkHvc2 = 16,
    LarkNesLeft = 17,
    LarkNesRight = 18,
    Lucia = 19,
    Verification = 20,
    Lagon = 21,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum NpadLarkType {
    Invalid = 0,
    H1 = 1,
    H2 = 2,
    NL = 3,
    NR = 4,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum NpadLuciaType {
    Invalid = 0,
    J = 1,
    E = 2,
    U = 3,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum NpadLagerType {
    Invalid = 0,
    J = 1,
    E = 2,
    U = 3,
}

define_bit_enum! {
    SixAxisSensorProperties (u8) {
        IsSixAxisSensorDeviceNewlyAssigned = bit!(0),
        IsFirmwareUpdateAvailableForSixAxisSensor = bit!(1)
    }
}

define_bit_enum! {
    LockKeyFlags (u32) {
        NumLockOn = bit!(0),
        NumLockOff = bit!(1),
        NumLockToggle = bit!(2),
        CapsLockOn = bit!(3),
        CapsLockOff = bit!(4),
        CapsLockToggle = bit!(5),
        ScrollLockOn = bit!(6),
        ScrollLockOff = bit!(7),
        ScrollLockToggle = bit!(8)
    }
}

ipc_sf_define_default_interface_client!(AppletResource);
ipc_sf_define_interface_trait! {
    trait AppletResource {
        get_shared_memory_handle [0, version::VersionInterval::all(), mut]: () => (shmem_handle: sf::CopyHandle) (shmem_handle: sf::CopyHandle);
    }
}

ipc_sf_define_default_interface_client!(HidServer);
ipc_sf_define_interface_trait! {
    trait HidServer {
        create_applet_resource [0, version::VersionInterval::all(), mut]: (aruid: AppletResourceUserId) =>  (applet_resource: AppletResource) (applet_resource: session_type!(AppletResource));
        set_supported_npad_style_set [100, version::VersionInterval::all(), mut]: (npad_style_tag: NpadStyleTag, aruid: AppletResourceUserId) =>  () ();
        get_supported_npad_style_set [101, version::VersionInterval::all()]: (aruid: AppletResourceUserId) =>  (npad_style_tag: NpadStyleTag) (npad_style_tag: NpadStyleTag);
        set_supported_npad_id_type [102, version::VersionInterval::all(), mut]: (aruid: AppletResourceUserId, npad_ids: sf::InPointerBuffer<NpadIdType>) =>  () ();
        activate_npad [103, version::VersionInterval::all(), mut]: (aruid: AppletResourceUserId) =>  () ();
        deactivate_npad [104, version::VersionInterval::all(), mut]: (aruid: AppletResourceUserId) =>  () ();
        activate_npad_with_revision [109, version::VersionInterval::all(), mut]: (revision: i32, aruid: AppletResourceUserId) =>  () ();
        set_npad_joy_assignment_mode_single [123, version::VersionInterval::all(), mut]: (npad_id: NpadIdType, aruid: AppletResourceUserId, joy_type: NpadJoyDeviceType) =>  () ();
        set_npad_joy_assignment_mode_dual [124, version::VersionInterval::all(), mut]: (npad_id: NpadIdType, aruid: AppletResourceUserId) =>  () ();
    }
}

ipc_sf_define_default_interface_client!(HidSysServer);
ipc_sf_define_interface_trait! {
    trait HidSysServer {
        send_keyboard_lock_key_event [31, version::VersionInterval::all()]: (flags: LockKeyFlags) => () ();
        acquire_home_button_event_handle [101, version::VersionInterval::all()]: (aruid: AppletResourceUserId) =>  (handle: svc::Handle) (handle: svc::Handle);
        activate_home_button [111, version::VersionInterval::all()]: (aruid: AppletResourceUserId) => () ();
        acquire_sleep_button_event_handle [121, version::VersionInterval::all()]: (aruid: AppletResourceUserId) =>  (handle: svc::Handle) (handle: svc::Handle);
        activate_sleep_button [131, version::VersionInterval::all()]: (aruid: AppletResourceUserId) => () ();
        acquire_capture_button_event_handle [141, version::VersionInterval::all()]: (aruid: AppletResourceUserId) =>  (handle: svc::Handle) (handle: svc::Handle);
        activate_capture_button [151, version::VersionInterval::all()]: (aruid: AppletResourceUserId) => () ();
    }
}
