//! HBL (homebrew loader) ABI support and utils

use crate::svc::Handle;
use crate::svc::INVALID_HANDLE;
use crate::version;
use crate::util;
use crate::result::*;

/// Represents the entry value keys for a hbl ABI context
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum AbiConfigEntryKey {
    #[default]
    EndOfList = 0,
    MainThreadHandle = 1,
    NextLoadPath = 2,
    OverrideHeap = 3,
    OverrideService = 4,
    Argv = 5,
    SyscallAvailableHint = 6,
    AppletType = 7,
    AppletWorkaround = 8,
    Reserved9 = 9,
    ProcessHandle = 10,
    LastLoadResult = 11,
    RandomSeed = 14,
    UserIdStorage = 15,
    HosVersion = 16
}

define_bit_enum! {
    /// Represents optional flags for config entries
    AbiConfigEntryFlags (u32) {
        /// Mandatory entry
        Mandatory = bit!(0)
    }
}

define_bit_enum! {
    /// Represents optional flag values for the specific case of [`AbiConfigEntryKey::AppletType`] config entries
    AbiConfigAppletFlags (u32) {
        ApplicationOverride = bit!(0)
    }
}

/// Represents an ABI config entry layout
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct AbiConfigEntry {
    /// The entry type identifier
    pub key: AbiConfigEntryKey,
    /// The entry flags
    pub flags: AbiConfigEntryFlags,
    /// The entry-specific values
    pub value: [u64; 2]
}

/// Represents the hbl-ABI format of the system version
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Version {
    value: u32
}

impl Version {
    /// Represents the [`u64`] magic present in [`AbiConfigEntryKey::HosVersion`] entry values if Atmosphere is currently running
    pub const ATMOSPHERE_OS_IMPL_MAGIC: u64 = u64::from_be_bytes(*b"ATMOSPHR");

    /// Represents the bit set in the [`Version`] value if Atmosphere is the current OS implementation
    pub const IS_ATMOSPHERE_BIT: u32 = bit!(31);

    /// Creates an empty [`Version`], whose value will be `0.0.0`
    #[inline]
    pub const fn empty() -> Self {
        Self { value: 0 }
    }

    /// Creates a [`Version`] from a raw value and the magic representing the current OS implementation
    /// 
    /// # Arguments
    /// 
    /// * `value`: The raw value
    /// * `os_impl_magic`: The magic value
    #[inline]
    pub const fn new(value: u32, os_impl_magic: u64) -> Self {
        let actual_value = match os_impl_magic == Self::ATMOSPHERE_OS_IMPL_MAGIC {
            true => value | Self::IS_ATMOSPHERE_BIT,
            false => value
        };

        Self { value: actual_value }
    }

    /// Gets the major component of the [`Version`]
    #[inline]
    pub const fn get_major(&self) -> u8 {
        ((self.value >> 16) & 0xFF) as u8
    }

    /// Gets the minor component of the [`Version`]
    #[inline]
    pub const fn get_minor(&self) -> u8 {
        ((self.value >> 8) & 0xFF) as u8
    }

    /// Gets the micro component of the [`Version`]
    #[inline]
    pub const fn get_micro(&self) -> u8 {
        (self.value & 0xFF) as u8
    }

    /// Gets whether Atmosphere is the current OS implementation
    #[inline]
    pub const fn is_atmosphere(&self) -> bool {
        (self.value & Self::IS_ATMOSPHERE_BIT) != 0
    }

    /// Gets a [`Version`][`version::Version`] type from this [`Version`]
    #[inline]
    pub const fn to_version(&self) -> version::Version {
        version::Version::new(self.get_major(), self.get_minor(), self.get_micro())
    }
}

static mut G_LAST_LOAD_RESULT: ResultCode = ResultCode::new(0); // TODO: const result traits for ResultSuccess?

pub(crate) fn set_last_load_result(rc: ResultCode) {
    unsafe {
        G_LAST_LOAD_RESULT = rc;
    }
}

/// Gets the last load [`ResultCode`]
/// 
/// This value represents the [`ResultCode`] of the last homebrew NRO executed before the current one
/// 
/// This value will only be set/useful if the current code is running through HBL
pub fn get_last_load_result() -> ResultCode {
    unsafe {
        G_LAST_LOAD_RESULT
    }
}

static mut G_PROCESS_HANDLE: Handle = INVALID_HANDLE;

pub(crate) fn set_process_handle(handle: Handle) {
    unsafe {
        G_PROCESS_HANDLE = handle;
    }
}

/// Gets the current process handle
/// 
/// This value will only be set/useful if the current code is running through HBL 
pub fn get_process_handle() -> Handle {
    unsafe {
        G_PROCESS_HANDLE
    }
}

/// Represents the applet types for HBL
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(i32)]
pub enum AppletType {
    #[default]
    None = -2,
    Default = -1,
    Application = 0,
    SystemApplet = 1,
    LibraryApplet = 2,
    OverlayApplet = 3,
    SystemApplication = 4
}

static mut G_APPLET_TYPE: AppletType = AppletType::None;

pub(crate) fn set_applet_type(applet_type: AppletType) {
    unsafe {
        G_APPLET_TYPE = applet_type;
    }
}

/// Gets the current applet type (according to HBL)
/// 
/// This value will only be set/useful if the current code is running through HBL
pub fn get_applet_type() -> AppletType {
    unsafe {
        G_APPLET_TYPE
    }
}

static mut G_LOADER_INFO: &'static str = "";

pub(crate) fn set_loader_info(loader_info: &'static str) {
    unsafe {
        G_LOADER_INFO = loader_info;
    }
}

/// Gets the loader information string, about HBL
/// 
/// This value will only be set/useful if the current code is running through HBL
pub fn get_loader_info() -> &'static str {
    unsafe {
        G_LOADER_INFO
    }
}

static mut G_NEXT_LOAD_PATH: &'static str = "";
static mut G_NEXT_LOAD_ARGV: &'static str = "";

pub(crate) fn set_next_load_entry_ptr(next_load_path: &'static str, next_load_argv: &'static str) {
    unsafe {
        G_NEXT_LOAD_PATH = next_load_path;
        G_NEXT_LOAD_ARGV = next_load_argv;
    }
}

/// Gets the next load path, AKA the path of the homebrew NRO which will be executed after this one exits
/// 
/// This value will only be set/useful if the current code is running through HBL
pub fn get_next_load_path() -> &'static str {
    unsafe {
        G_NEXT_LOAD_PATH
    }
}

/// Gets the next load argv, AKA the argv of the homebrew NRO which will be executed after this one exits
/// 
/// This value will only be set/useful if the current code is running through HBL
pub fn get_next_load_argv() -> &'static str {
    unsafe {
        G_NEXT_LOAD_ARGV
    }
}

/// Sets the next homebrew NRO (path and argv) to execute after this one exits
/// 
/// This will only make any effect if the current code is running through HB
/// 
/// # Arguments
/// 
/// * `next_load_path`: NRO path
/// * `next_load_argv`: NRO argv
pub fn set_next_load_entry(next_load_path: &'static str, next_load_argv: &'static str) {
    unsafe {
        // Note: using this system to copy the strings since we must preserve the string pointers HBL sent us
        G_NEXT_LOAD_PATH = util::str_copy(G_NEXT_LOAD_PATH, next_load_path);
        G_NEXT_LOAD_ARGV = util::str_copy(G_NEXT_LOAD_ARGV, next_load_argv);
    }
}

static mut G_RANDOM_SEED: (u64, u64) = (0, 0);

pub(crate) fn set_random_seed(seed: (u64, u64)) {
    unsafe {
        G_RANDOM_SEED = seed;
    }
}

/// Gets the random seed values sent by HBL
/// 
/// This values will only be set/useful if the current code is running through HBL
pub fn get_random_seed() -> (u64, u64) {
    unsafe {
        G_RANDOM_SEED
    }
}