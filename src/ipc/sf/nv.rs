use crate::result::*;
use crate::ipc::sf;
use crate::version;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum ErrorCode {
    #[default]
    Success = 0,
    NotImplemented = 1,
    NotSupported = 2,
    NotInitialized = 3,
    InvalidParameter = 4,
    TimeOut = 5,
    InsufficientMemory = 6,
    ReadOnlyAttribute = 7,
    InvalidState = 8,
    InvalidAddress = 9,
    InvalidSize = 10,
    InvalidValue = 11,
    AlreadyAllocated = 13,
    Busy = 14,
    ResourceError = 15,
    CountMismatch = 16,
    SharedMemoryTooSmall = 0x1000,
    FileOperationFailed = 0x30003,
    IoctlFailed = 0x3000F
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum IoctlId {
    NvMapCreate = 0xC0080101,
    NvMapFromId = 0xC0080103,
    NvMapAlloc = 0xC0200104,
    NvMapFree = 0xC0180105,
    NvMapParam = 0xC00C0109,
    NvMapGetId = 0xC008010E,

    NvHostCtrlSyncptWait = 0xC00C0016
}

pub type Fd = u32;

ipc_sf_define_interface_trait! {
    trait INvDrvServices {
        open [0, version::VersionInterval::all()]: (path: sf::InMapAliasBuffer<u8>) => (fd: Fd, error_code: ErrorCode);
        ioctl [1, version::VersionInterval::all()]: (fd: Fd, id: IoctlId, in_buf: sf::InAutoSelectBuffer<u8>, out_buf: sf::OutAutoSelectBuffer<u8>) => (error_code: ErrorCode);
        close [2, version::VersionInterval::all()]: (fd: Fd) => (error_code: ErrorCode);
        initialize [3, version::VersionInterval::all()]: (transfer_mem_size: u32, self_process_handle: sf::CopyHandle, transfer_mem_handle: sf::CopyHandle) => (error_code: ErrorCode);
    }
}