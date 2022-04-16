use crate::result::*;
use crate::results;
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

#[allow(unreachable_patterns)]
pub fn convert_error_code(err: ErrorCode) -> Result<()> {
    match err {
        ErrorCode::Success => Ok(()),
        ErrorCode::NotImplemented => Err(results::lib::gpu::ResultNvErrorCodeNotImplemented::make()),
        ErrorCode::NotSupported => Err(results::lib::gpu::ResultNvErrorCodeNotSupported::make()),
        ErrorCode::NotInitialized => Err(results::lib::gpu::ResultNvErrorCodeNotInitialized::make()),
        ErrorCode::InvalidParameter => Err(results::lib::gpu::ResultNvErrorCodeInvalidParameter::make()),
        ErrorCode::TimeOut => Err(results::lib::gpu::ResultNvErrorCodeTimeOut::make()),
        ErrorCode::InsufficientMemory => Err(results::lib::gpu::ResultNvErrorCodeInsufficientMemory::make()),
        ErrorCode::ReadOnlyAttribute => Err(results::lib::gpu::ResultNvErrorCodeReadOnlyAttribute::make()),
        ErrorCode::InvalidState => Err(results::lib::gpu::ResultNvErrorCodeInvalidState::make()),
        ErrorCode::InvalidAddress => Err(results::lib::gpu::ResultNvErrorCodeInvalidAddress::make()),
        ErrorCode::InvalidSize => Err(results::lib::gpu::ResultNvErrorCodeInvalidSize::make()),
        ErrorCode::InvalidValue => Err(results::lib::gpu::ResultNvErrorCodeInvalidValue::make()),
        ErrorCode::AlreadyAllocated => Err(results::lib::gpu::ResultNvErrorCodeAlreadyAllocated::make()),
        ErrorCode::Busy => Err(results::lib::gpu::ResultNvErrorCodeBusy::make()),
        ErrorCode::ResourceError => Err(results::lib::gpu::ResultNvErrorCodeResourceError::make()),
        ErrorCode::CountMismatch => Err(results::lib::gpu::ResultNvErrorCodeCountMismatch::make()),
        ErrorCode::SharedMemoryTooSmall => Err(results::lib::gpu::ResultNvErrorCodeSharedMemoryTooSmall::make()),
        ErrorCode::FileOperationFailed => Err(results::lib::gpu::ResultNvErrorCodeFileOperationFailed::make()),
        ErrorCode::IoctlFailed => Err(results::lib::gpu::ResultNvErrorCodeIoctlFailed::make()),
        _ => Err(results::lib::gpu::ResultNvErrorCodeInvalid::make()),
    }
}

ipc_sf_define_interface_trait! {
    trait INvDrvServices {
        open [0, version::VersionInterval::all()]: (path: sf::InMapAliasBuffer<u8>) => (fd: Fd, error_code: ErrorCode);
        ioctl [1, version::VersionInterval::all()]: (fd: Fd, id: IoctlId, in_buf: sf::InAutoSelectBuffer<u8>, out_buf: sf::OutAutoSelectBuffer<u8>) => (error_code: ErrorCode);
        close [2, version::VersionInterval::all()]: (fd: Fd) => (error_code: ErrorCode);
        initialize [3, version::VersionInterval::all()]: (transfer_mem_size: u32, self_process_handle: sf::CopyHandle, transfer_mem_handle: sf::CopyHandle) => (error_code: ErrorCode);
    }
}