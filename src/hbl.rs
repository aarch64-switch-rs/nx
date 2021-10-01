use crate::svc::Handle;
use crate::svc::INVALID_HANDLE;
use crate::version;
use crate::util;
use crate::result::*;

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

bit_enum! {
    AbiConfigEntryFlags (u32) {
        Mandatory = bit!(0)
    }
}

bit_enum! {
    AbiConfigAppletFlags (u32) {
        ApplicationOverride = bit!(0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct AbiConfigEntry {
    pub key: AbiConfigEntryKey,
    pub flags: AbiConfigEntryFlags,
    pub value: [u64; 2],
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Version {
    value: u32
}

impl Version {
    pub const IS_ATMOSPHERE_MAGIC: u64 = u64::from_be_bytes(*b"ATMOSPHR");
    pub const IS_ATMOSPHERE_BIT: u32 = bit!(31);

    pub const fn empty() -> Self {
        Self { value: 0 }
    }

    pub const fn new(value: u32, is_ams_magic: u64) -> Self {
        let actual_value = match is_ams_magic == Self::IS_ATMOSPHERE_MAGIC {
            true => value | Self::IS_ATMOSPHERE_BIT,
            false => value
        };

        Self { value: actual_value }
    }

    pub const fn get_major(&self) -> u8 {
        ((self.value >> 16) & 0xFF) as u8
    }

    pub const fn get_minor(&self) -> u8 {
        ((self.value >> 8) & 0xFF) as u8
    }

    pub const fn get_micro(&self) -> u8 {
        (self.value & 0xFF) as u8
    }

    pub const fn is_valid(&self) -> bool {
        self.value != 0
    }

    pub const fn is_atmosphere(&self) -> bool {
        (self.value & Self::IS_ATMOSPHERE_BIT) != 0
    }

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

pub fn get_process_handle() -> Handle {
    unsafe {
        G_PROCESS_HANDLE
    }
}

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

pub fn get_next_load_path() -> &'static str {
    unsafe {
        G_NEXT_LOAD_PATH
    }
}

pub fn set_next_load_entry(next_load_path: &'static str, next_load_argv: &'static str) {
    unsafe {
        G_NEXT_LOAD_PATH = util::str_copy(G_NEXT_LOAD_PATH, next_load_path);
        G_NEXT_LOAD_ARGV = util::str_copy(G_NEXT_LOAD_ARGV, next_load_argv);
    }
}

pub fn get_next_load_argv() -> &'static str {
    unsafe {
        G_NEXT_LOAD_ARGV
    }
}

static mut G_RANDOM_SEED: (u64, u64) = (0, 0);

pub(crate) fn set_random_seed(seed: (u64, u64)) {
    unsafe {
        G_RANDOM_SEED = seed;
    }
}

pub fn get_random_seed() -> (u64, u64) {
    unsafe {
        G_RANDOM_SEED
    }
}