use crate::version;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Derivative)]
#[derivative(Default)]
#[repr(u32)]
pub enum AbiConfigEntryKey {
    #[derivative(Default)]
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
    pub const fn empty() -> Self {
        Self { value: 0 }
    }
    
    pub const fn new(value: u32) -> Self {
        Self { value: value }
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

    pub const fn to_version(&self) -> version::Version {
        version::Version::new(self.get_major(), self.get_minor(), self.get_micro())
    }
}