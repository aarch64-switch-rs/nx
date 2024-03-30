use super::*;

pub const SHMEM_SIZE: usize = 0x40000;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct RingLifo<T: Copy, const S: usize> {
    pub vptr: u64,
    pub buf_count: u64,
    pub tail: u64,
    pub count: u64,
    pub items: [T; S]
}

impl<T: Copy, const S: usize> RingLifo<T, S> {
    pub fn get_tail_item(&self) -> &T {
        &self.items[self.tail as usize]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DebugPadState {
    pub sampling_number: u64,
    pub attrs: DebugPadAttribute,
    pub buttons: DebugPadButton,
    pub analog_stick_r: AnalogStickState,
    pub analog_stick_l: AnalogStickState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DebugPadStateAtomicStorage {
    pub sampling_number: u64,
    pub state: DebugPadState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DebugPadSharedMemoryFormat {
    pub lifo: RingLifo<DebugPadStateAtomicStorage, 17>,
    pub pad: [u8; 0x138]
}
const_assert!(core::mem::size_of::<DebugPadSharedMemoryFormat>() == 0x400);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct TouchScreenState {
    pub sampling_number: u64,
    pub count: u32,
    pub reserved: [u8; 4],
    pub touches: [TouchState; 16]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct TouchScreenStateAtomicStorage {
    pub sampling_number: u64,
    pub state: TouchScreenState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct TouchScreenSharedMemoryFormat {
    pub lifo: RingLifo<TouchScreenStateAtomicStorage, 17>,
    pub pad: [u8; 0x3C8]
}
const_assert!(core::mem::size_of::<TouchScreenSharedMemoryFormat>() == 0x3000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct MouseState {
    pub sampling_number: u64,
    pub x: u32,
    pub y: u32,
    pub delta_x: u32,
    pub delta_y: u32,
    pub wheel_delta_x: u32,
    pub wheel_delta_y: u32,
    pub buttons: MouseButton,
    pub attributes: MouseAttribute
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct MouseStateAtomicStorage {
    pub sampling_number: u64,
    pub state: MouseState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct MouseSharedMemoryFormat {
    pub lifo: RingLifo<MouseStateAtomicStorage, 17>,
    pub pad: [u8; 0xB0]
}
const_assert!(core::mem::size_of::<MouseSharedMemoryFormat>() == 0x400);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct KeyboardState {
    pub sampling_number: u64,
    pub modifiers: KeyboardModifier,
    pub keys: KeyboardKey
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct KeyboardStateAtomicStorage {
    pub sampling_number: u64,
    pub state: KeyboardState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct KeyboardSharedMemoryFormat {
    pub lifo: RingLifo<KeyboardStateAtomicStorage, 17>,
    pub pad: [u8; 0x28]
}
const_assert!(core::mem::size_of::<KeyboardSharedMemoryFormat>() == 0x400);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct BasicXpadState {
    pub sampling_number: u64,
    pub attributes: BasicXpadAttribute,
    pub buttons: BasicXpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct BasicXpadStateAtomicStorage {
    pub sampling_number: u64,
    pub state: BasicXpadState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct BasicXpadSharedMemoryEntry {
    pub lifo: RingLifo<BasicXpadStateAtomicStorage, 17>,
    pub pad: [u8; 0x138]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct BasicXpadSharedMemoryFormat {
    pub entries: [BasicXpadSharedMemoryEntry; 4]
}
const_assert!(core::mem::size_of::<BasicXpadSharedMemoryFormat>() == 0x1000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DigitizerState {
    pub sampling_number: u64,
    pub unk_1: [u32; 2],
    pub attributes: DigitizerAttribute,
    pub buttons: DigitizerButton,
    pub unk_2: [u32; 16]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DigitizerStateAtomicStorage {
    pub sampling_number: u64,
    pub state: DigitizerState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct DigitizerSharedMemoryFormat {
    pub lifo: RingLifo<DigitizerStateAtomicStorage, 17>,
    pub pad: [u8; 0x980]
}
const_assert!(core::mem::size_of::<DigitizerSharedMemoryFormat>() == 0x1000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct HomeButtonState {
    pub sampling_number: u64,
    pub buttons: HomeButton
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct HomeButtonStateAtomicStorage {
    pub sampling_number: u64,
    pub state: HomeButtonState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct HomeButtonSharedMemoryFormat {
    pub lifo: RingLifo<HomeButtonStateAtomicStorage, 17>,
    pub pad: [u8; 0x48]
}
const_assert!(core::mem::size_of::<HomeButtonSharedMemoryFormat>() == 0x200);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SleepButtonState {
    pub sampling_number: u64,
    pub buttons: SleepButton
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SleepButtonStateAtomicStorage {
    pub sampling_number: u64,
    pub state: SleepButtonState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SleepButtonSharedMemoryFormat {
    pub lifo: RingLifo<SleepButtonStateAtomicStorage, 17>,
    pub pad: [u8; 0x48]
}
const_assert!(core::mem::size_of::<SleepButtonSharedMemoryFormat>() == 0x200);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CaptureButtonState {
    pub sampling_number: u64,
    pub buttons: CaptureButton
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CaptureButtonStateAtomicStorage {
    pub sampling_number: u64,
    pub state: CaptureButtonState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CaptureButtonSharedMemoryFormat {
    pub lifo: RingLifo<CaptureButtonStateAtomicStorage, 17>,
    pub pad: [u8; 0x48]
}
const_assert!(core::mem::size_of::<CaptureButtonSharedMemoryFormat>() == 0x200);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InputDetectorState {
    pub source_state: InputSourceState,
    pub sampling_number: u64
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InputDetectorStateAtomicStorage {
    pub sampling_number: u64,
    pub state: InputDetectorState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InputDetectorSharedMemoryEntry {
    pub lifo: RingLifo<InputDetectorStateAtomicStorage, 2>,
    pub pad: [u8; 0x30]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InputDetectorSharedMemoryFormat {
    pub entries: [InputDetectorSharedMemoryEntry; 16]
}
const_assert!(core::mem::size_of::<InputDetectorSharedMemoryFormat>() == 0x800);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct UniquePadConfig {
    pub pad_type: UniquePadType,
    pub interface: UniquePadInterface,
    pub serial_number: UniquePadSerialNumber,
    pub controller_number: u32,
    pub is_active: bool,
    pub reserved: [u8; 3],
    pub sampling_number: u64
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct UniquePadConfigAtomicStorage {
    pub sampling_number: u64,
    pub config: UniquePadConfig
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct AnalogStickCalibrationStateImpl {
    pub state: AnalogStickState,
    pub flags: AnalogStickCalibrationFlags,
    pub stage: AnalogStickManualCalibrationStage,
    pub sampling_number: u64
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct AnalogStickCalibrationStateImplAtomicStorage {
    pub sampling_number: u64,
    pub state: AnalogStickCalibrationStateImpl
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SixAxisSensorUserCalibrationState {
    pub flags: SixAxisSensorUserCalibrationFlags,
    pub reserved: [u8; 4],
    pub stage: SixAxisSensorUserCalibrationStage,
    pub sampling_number: u64
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SixAxisSensorUserCalibrationStateAtomicStorage {
    pub sampling_number: u64,
    pub state: SixAxisSensorUserCalibrationState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct UniquePadSharedMemoryEntry {
    pub config_lifo: RingLifo<UniquePadConfigAtomicStorage, 2>,
    pub analog_stick_calibration_state_impls: [RingLifo<AnalogStickCalibrationStateImplAtomicStorage, 2>; 2],
    pub six_axis_sensor_user_calibration_state: RingLifo<SixAxisSensorUserCalibrationStateAtomicStorage, 2>,
    pub unique_pad_config_mutex: [u8; 0x40],
    pub pad: [u8; 0x220]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct UniquePadSharedMemoryFormat {
    pub entries: [UniquePadSharedMemoryEntry; 16]
}
const_assert!(core::mem::size_of::<UniquePadSharedMemoryFormat>() == 0x4000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadFullKeyColorState {
    pub attribute: ColorAttribute,
    pub color: NpadControllerColor
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadJoyColorState {
    pub attribute: ColorAttribute,
    pub left_joy_color: NpadControllerColor,
    pub right_joy_color: NpadControllerColor
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadFullKeyState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadFullKeyStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadFullKeyState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadHandheldState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadHandheldStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadHandheldState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadJoyDualState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadJoyDualStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadJoyDualState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadJoyLeftState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadJoyLeftStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadJoyLeftState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadJoyRightState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadJoyRightStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadJoyRightState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSystemState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSystemStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadSystemState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadPalmaState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadPalmaStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadPalmaState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSystemExtState {
    pub sampling_number: u64,
    pub buttons: NpadButton,
    pub analog_stick_l: AnalogStickState,
    pub analog_stick_r: AnalogStickState,
    pub attributes: NpadAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSystemExtStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadSystemExtState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SixAxisSensorState {
    pub delta_time: u64,
    pub sampling_number: u64,
    pub acceleration_x: u32,
    pub acceleration_y: u32,
    pub acceleration_z: u32,
    pub angular_velocity_x: u32,
    pub angular_velocity_y: u32,
    pub angular_velocity_z: u32,
    pub angle_x: u32,
    pub angle_y: u32,
    pub angle_z: u32,
    pub direction: DirectionState,
    pub attributes: SixAxisSensorAttribute,
    pub reserved: [u8; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct SixAxisSensorStateAtomicStorage {
    pub sampling_number: u64,
    pub state: SixAxisSensorState
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NfcXcdDeviceHandleStateImpl {
    pub handle: u64, // TODO: xcd::DeviceHandle
    pub is_available: bool,
    pub is_activated: bool,
    pub reserved: [u8; 6],
    pub sampling_number: u64
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NfcXcdDeviceHandleStateImplAtomicStorage {
    pub sampling_number: u64,
    pub state: NfcXcdDeviceHandleStateImpl
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadGcTriggerState {
    pub sampling_number: u64,
    pub trigger_l: u32,
    pub trigger_r: u32
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadGcTriggerStateAtomicStorage {
    pub sampling_number: u64,
    pub state: NpadGcTriggerState
}

// V1: 1.0.0-3.0.2
// V2: 4.0.0-8.1.0
// V3: 9.0.0-12.1.0
// V4: 13.0.0-

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryEntryV1 {
    pub style_tag: NpadStyleTag,
    pub joy_assignment_mode: NpadJoyAssignmentMode,
    pub full_key_color: NpadFullKeyColorState,
    pub joy_color: NpadJoyColorState,
    pub full_key_lifo: RingLifo<NpadFullKeyStateAtomicStorage, 17>,
    pub handheld_lifo: RingLifo<NpadHandheldStateAtomicStorage, 17>,
    pub joy_dual_lifo: RingLifo<NpadJoyDualStateAtomicStorage, 17>,
    pub joy_left_lifo: RingLifo<NpadJoyLeftStateAtomicStorage, 17>,
    pub joy_right_lifo: RingLifo<NpadJoyRightStateAtomicStorage, 17>,
    pub system_lifo: RingLifo<NpadSystemStateAtomicStorage, 17>,
    pub system_ext_lifo: RingLifo<NpadSystemExtStateAtomicStorage, 17>,
    pub full_key_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub handheld_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub device_type: DeviceType,
    pub reserved: [u8; 4],
    pub system_properties: NpadSystemProperties,
    pub system_button_properties: NpadSystemButtonProperties,
    pub battery_level_joy_dual: NpadBatteryLevel,
    pub battery_level_joy_left: NpadBatteryLevel,
    pub battery_level_joy_right: NpadBatteryLevel,
    pub nfc_xcd_device_handle_lifo: RingLifo<NfcXcdDeviceHandleStateImplAtomicStorage, 2>,
    pub mutex: [u8; 0x40],
    pub gc_trigger_lifo: RingLifo<NpadGcTriggerStateAtomicStorage, 17>,
    pub lark_type_l_and_main: NpadLarkType,
    pub lark_type_r: NpadLarkType,
    pub lucia_type: NpadLuciaType,
    pub lager_type: NpadLagerType,
    pub pad: [u8; 0xBF0]
}
const_assert!(core::mem::size_of::<NpadSharedMemoryEntryV1>() == 0x5000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryEntryV2 {
    pub style_tag: NpadStyleTag,
    pub joy_assignment_mode: NpadJoyAssignmentMode,
    pub full_key_color: NpadFullKeyColorState,
    pub joy_color: NpadJoyColorState,
    pub full_key_lifo: RingLifo<NpadFullKeyStateAtomicStorage, 17>,
    pub handheld_lifo: RingLifo<NpadHandheldStateAtomicStorage, 17>,
    pub joy_dual_lifo: RingLifo<NpadJoyDualStateAtomicStorage, 17>,
    pub joy_left_lifo: RingLifo<NpadJoyLeftStateAtomicStorage, 17>,
    pub joy_right_lifo: RingLifo<NpadJoyRightStateAtomicStorage, 17>,
    pub palma_lifo: RingLifo<NpadPalmaStateAtomicStorage, 17>,
    pub system_ext_lifo: RingLifo<NpadSystemExtStateAtomicStorage, 17>,
    pub full_key_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub handheld_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub device_type: DeviceType,
    pub reserved: [u8; 4],
    pub system_properties: NpadSystemProperties,
    pub system_button_properties: NpadSystemButtonProperties,
    pub battery_level_joy_dual: NpadBatteryLevel,
    pub battery_level_joy_left: NpadBatteryLevel,
    pub battery_level_joy_right: NpadBatteryLevel,
    pub nfc_xcd_device_handle_lifo: RingLifo<NfcXcdDeviceHandleStateImplAtomicStorage, 2>,
    pub mutex: [u8; 0x40],
    pub gc_trigger_lifo: RingLifo<NpadGcTriggerStateAtomicStorage, 17>,
    pub lark_type_l_and_main: NpadLarkType,
    pub lark_type_r: NpadLarkType,
    pub lucia_type: NpadLuciaType,
    pub lager_type: NpadLagerType,
    pub pad: [u8; 0xBF0]
}
const_assert!(core::mem::size_of::<NpadSharedMemoryEntryV2>() == 0x5000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryEntryV3 {
    pub style_tag: NpadStyleTag,
    pub joy_assignment_mode: NpadJoyAssignmentMode,
    pub full_key_color: NpadFullKeyColorState,
    pub joy_color: NpadJoyColorState,
    pub full_key_lifo: RingLifo<NpadFullKeyStateAtomicStorage, 17>,
    pub handheld_lifo: RingLifo<NpadHandheldStateAtomicStorage, 17>,
    pub joy_dual_lifo: RingLifo<NpadJoyDualStateAtomicStorage, 17>,
    pub joy_left_lifo: RingLifo<NpadJoyLeftStateAtomicStorage, 17>,
    pub joy_right_lifo: RingLifo<NpadJoyRightStateAtomicStorage, 17>,
    pub palma_lifo: RingLifo<NpadPalmaStateAtomicStorage, 17>,
    pub system_ext_lifo: RingLifo<NpadSystemExtStateAtomicStorage, 17>,
    pub full_key_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub handheld_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub device_type: DeviceType,
    pub reserved: [u8; 4],
    pub system_properties: NpadSystemProperties,
    pub system_button_properties: NpadSystemButtonProperties,
    pub battery_level_joy_dual: NpadBatteryLevel,
    pub battery_level_joy_left: NpadBatteryLevel,
    pub battery_level_joy_right: NpadBatteryLevel,
    pub applet_footer_ui_attributes: AppletFooterUiAttribute,
    pub applet_footer_ui_type: AppletFooterUiType,
    pub reserved_2: [u8; 0x7B],
    pub gc_trigger_lifo: RingLifo<NpadGcTriggerStateAtomicStorage, 17>,
    pub lark_type_l_and_main: NpadLarkType,
    pub lark_type_r: NpadLarkType,
    pub lucia_type: NpadLuciaType,
    pub lager_type: NpadLagerType,
    pub pad: [u8; 0xC10]
}
const_assert!(core::mem::size_of::<NpadSharedMemoryEntryV3>() == 0x5000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryEntryV4 {
    pub style_tag: NpadStyleTag,
    pub joy_assignment_mode: NpadJoyAssignmentMode,
    pub full_key_color: NpadFullKeyColorState,
    pub joy_color: NpadJoyColorState,
    pub full_key_lifo: RingLifo<NpadFullKeyStateAtomicStorage, 17>,
    pub handheld_lifo: RingLifo<NpadHandheldStateAtomicStorage, 17>,
    pub joy_dual_lifo: RingLifo<NpadJoyDualStateAtomicStorage, 17>,
    pub joy_left_lifo: RingLifo<NpadJoyLeftStateAtomicStorage, 17>,
    pub joy_right_lifo: RingLifo<NpadJoyRightStateAtomicStorage, 17>,
    pub palma_lifo: RingLifo<NpadPalmaStateAtomicStorage, 17>,
    pub system_ext_lifo: RingLifo<NpadSystemExtStateAtomicStorage, 17>,
    pub full_key_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub handheld_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_dual_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_left_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub joy_right_six_axis_sensor_lifo: RingLifo<SixAxisSensorStateAtomicStorage, 17>,
    pub device_type: DeviceType,
    pub reserved: [u8; 4],
    pub system_properties: NpadSystemProperties,
    pub system_button_properties: NpadSystemButtonProperties,
    pub battery_level_joy_dual: NpadBatteryLevel,
    pub battery_level_joy_left: NpadBatteryLevel,
    pub battery_level_joy_right: NpadBatteryLevel,
    pub applet_footer_ui_attributes: AppletFooterUiAttribute,
    pub applet_footer_ui_type: AppletFooterUiType,
    pub reserved_2: [u8; 0x7B],
    pub gc_trigger_lifo: RingLifo<NpadGcTriggerStateAtomicStorage, 17>,
    pub lark_type_l_and_main: NpadLarkType,
    pub lark_type_r: NpadLarkType,
    pub lucia_type: NpadLuciaType,
    pub lager_type: NpadLagerType,
    pub six_axis_sensor_properties: [SixAxisSensorProperties; 6],
    pub pad: [u8; 0xC08]
}
const_assert!(core::mem::size_of::<NpadSharedMemoryEntryV4>() == 0x5000);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryFormatV1 {
    pub entries: [NpadSharedMemoryEntryV1; 10]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryFormatV2 {
    pub entries: [NpadSharedMemoryEntryV2; 10]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryFormatV3 {
    pub entries: [NpadSharedMemoryEntryV3; 10]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct NpadSharedMemoryFormatV4 {
    pub entries: [NpadSharedMemoryEntryV4; 10]
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct GestureDummyState {
    pub sampling_number: u64,
    pub context_number: u8,
    pub gesture_type: GestureType,
    pub direction: GestureDirection,
    pub x: u32,
    pub y: u32,
    pub delta_x: u32,
    pub delta_y: u32,
    pub velocity_x: f32,
    pub velocity_y: f32,
    pub attributes: GestureAttribute,
    pub scale: u32,
    pub rotation_angle: u32,
    pub point_count: u32,
    pub points: [GesturePoint; 4]
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct GestureDummyStateAtomicStorage {
    pub sampling_number: u64,
    pub state: GestureDummyState
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct GestureSharedMemoryFormat {
    pub lifo: RingLifo<GestureDummyStateAtomicStorage, 17>,
    pub pad: [u8; 0xF8]
}
const_assert!(core::mem::size_of::<GestureSharedMemoryFormat>() == 0x800);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct ConsoleSixAxisSensorSharedMemoryFormat {
    pub sampling_number: u64,
    pub is_seven_six_axis_sensor_at_rest: bool,
    pub pad: [u8; 3],
    pub verticalization_error: u32,
    pub gyro_bias: [u8; 0xC]
}
const_assert!(core::mem::size_of::<ConsoleSixAxisSensorSharedMemoryFormat>() == 0x1C);

// V1: 1.0.0-3.0.2
// V2: 4.0.0-4.1.0
// V3: 5.0.0-8.1.0/8.1.1
// V4: 9.0.0-9.2.0
// V5: 10.0.0-12.1.0
// V6: 13.0.0-

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct SharedMemoryFormatV1 {
    pub debug_pad: DebugPadSharedMemoryFormat,
    pub touch_screen: TouchScreenSharedMemoryFormat,
    pub mouse: MouseSharedMemoryFormat,
    pub keyboard: KeyboardSharedMemoryFormat,
    pub basic_xpad: BasicXpadSharedMemoryFormat,
    pub home_button: HomeButtonSharedMemoryFormat,
    pub sleep_button: SleepButtonSharedMemoryFormat,
    pub capture_button: CaptureButtonSharedMemoryFormat,
    pub input_detector: InputDetectorSharedMemoryFormat,
    pub unique_pad: UniquePadSharedMemoryFormat,
    pub npad: NpadSharedMemoryFormatV1,
    pub gesture: GestureSharedMemoryFormat
}

impl SharedMemoryFormatV1 {
    pub const VERSION_INTERVAL: version::VersionInterval = version::VersionInterval::from_to(version::Version::new(1,0,0), version::Version::new(3,0,2));
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct SharedMemoryFormatV2 {
    pub debug_pad: DebugPadSharedMemoryFormat,
    pub touch_screen: TouchScreenSharedMemoryFormat,
    pub mouse: MouseSharedMemoryFormat,
    pub keyboard: KeyboardSharedMemoryFormat,
    pub basic_xpad: BasicXpadSharedMemoryFormat,
    pub home_button: HomeButtonSharedMemoryFormat,
    pub sleep_button: SleepButtonSharedMemoryFormat,
    pub capture_button: CaptureButtonSharedMemoryFormat,
    pub input_detector: InputDetectorSharedMemoryFormat,
    pub unique_pad: UniquePadSharedMemoryFormat,
    pub npad: NpadSharedMemoryFormatV2,
    pub gesture: GestureSharedMemoryFormat
}

impl SharedMemoryFormatV2 {
    pub const VERSION_INTERVAL: version::VersionInterval = version::VersionInterval::from_to(version::Version::new(4,0,0), version::Version::new(4,1,0));
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct SharedMemoryFormatV3 {
    pub debug_pad: DebugPadSharedMemoryFormat,
    pub touch_screen: TouchScreenSharedMemoryFormat,
    pub mouse: MouseSharedMemoryFormat,
    pub keyboard: KeyboardSharedMemoryFormat,
    pub basic_xpad: BasicXpadSharedMemoryFormat,
    pub home_button: HomeButtonSharedMemoryFormat,
    pub sleep_button: SleepButtonSharedMemoryFormat,
    pub capture_button: CaptureButtonSharedMemoryFormat,
    pub input_detector: InputDetectorSharedMemoryFormat,
    pub pad: [u8; 0x4000],
    pub npad: NpadSharedMemoryFormatV2,
    pub gesture: GestureSharedMemoryFormat,
    pub console_six_axis_sensor: ConsoleSixAxisSensorSharedMemoryFormat
}

impl SharedMemoryFormatV3 {
    pub const VERSION_INTERVAL: version::VersionInterval = version::VersionInterval::from_to(version::Version::new(5,0,0), version::Version::new(8,1,1));
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct SharedMemoryFormatV4 {
    pub debug_pad: DebugPadSharedMemoryFormat,
    pub touch_screen: TouchScreenSharedMemoryFormat,
    pub mouse: MouseSharedMemoryFormat,
    pub keyboard: KeyboardSharedMemoryFormat,
    pub basic_xpad: BasicXpadSharedMemoryFormat,
    pub home_button: HomeButtonSharedMemoryFormat,
    pub sleep_button: SleepButtonSharedMemoryFormat,
    pub capture_button: CaptureButtonSharedMemoryFormat,
    pub input_detector: InputDetectorSharedMemoryFormat,
    pub pad: [u8; 0x4000],
    pub npad: NpadSharedMemoryFormatV3,
    pub gesture: GestureSharedMemoryFormat,
    pub console_six_axis_sensor: ConsoleSixAxisSensorSharedMemoryFormat
}

impl SharedMemoryFormatV4 {
    pub const VERSION_INTERVAL: version::VersionInterval = version::VersionInterval::from_to(version::Version::new(9,0,0), version::Version::new(9,2,0));
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct SharedMemoryFormatV5 {
    pub debug_pad: DebugPadSharedMemoryFormat,
    pub touch_screen: TouchScreenSharedMemoryFormat,
    pub mouse: MouseSharedMemoryFormat,
    pub keyboard: KeyboardSharedMemoryFormat,
    pub digitizer: DigitizerSharedMemoryFormat,
    pub home_button: HomeButtonSharedMemoryFormat,
    pub sleep_button: SleepButtonSharedMemoryFormat,
    pub capture_button: CaptureButtonSharedMemoryFormat,
    pub input_detector: InputDetectorSharedMemoryFormat,
    pub pad: [u8; 0x4000],
    pub npad: NpadSharedMemoryFormatV3,
    pub gesture: GestureSharedMemoryFormat,
    pub console_six_axis_sensor: ConsoleSixAxisSensorSharedMemoryFormat
}

impl SharedMemoryFormatV5 {
    pub const VERSION_INTERVAL: version::VersionInterval = version::VersionInterval::from_to(version::Version::new(10,0,0), version::Version::new(12,1,0));
}

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct SharedMemoryFormatV6 {
    pub debug_pad: DebugPadSharedMemoryFormat,
    pub touch_screen: TouchScreenSharedMemoryFormat,
    pub mouse: MouseSharedMemoryFormat,
    pub keyboard: KeyboardSharedMemoryFormat,
    pub digitizer: DigitizerSharedMemoryFormat,
    pub home_button: HomeButtonSharedMemoryFormat,
    pub sleep_button: SleepButtonSharedMemoryFormat,
    pub capture_button: CaptureButtonSharedMemoryFormat,
    pub input_detector: InputDetectorSharedMemoryFormat,
    pub pad: [u8; 0x4000],
    pub npad: NpadSharedMemoryFormatV4,
    pub gesture: GestureSharedMemoryFormat,
    pub console_six_axis_sensor: ConsoleSixAxisSensorSharedMemoryFormat
}

impl SharedMemoryFormatV6 {
    pub const VERSION_INTERVAL: version::VersionInterval = version::VersionInterval::from(version::Version::new(13,0,0));
}