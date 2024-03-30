//! `ioctl` command definitions and support

use crate::service::nv;
use super::*;

/// Represents one of the available fds
/// 
/// Note that only the ones used so far in this library are present
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum IoctlFd {
    NvHost,
    NvMap,
    NvHostCtrl
}

/// Represents a type trait defining an `ioctl` command
pub trait Ioctl {
    /// Gets the [`IoctlId`][`nv::IoctlId`] of this command
    fn get_id() -> nv::IoctlId;

    /// Gets the [`IoctlFd`] of this command
    fn get_fd() -> IoctlFd;
}

/// Represents the `Create` command for [`NvMap`][`IoctlFd::NvMap`] fd
/// 
/// See <https://switchbrew.org/wiki/NV_services#NVMAP_IOC_CREATE>
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct NvMapCreate {
    /// The input map size
    pub size: u32,
    /// The output handle
    pub handle: u32
}

impl Ioctl for NvMapCreate {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapCreate
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

/// Represents the `FromId` command for [`NvMap`][`IoctlFd::NvMap`] fd
/// 
/// See <https://switchbrew.org/wiki/NV_services#NVMAP_IOC_FROM_ID>
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct NvMapFromId {
    /// The input ID
    pub id: u32,
    /// The output handle
    pub handle: u32
}

impl Ioctl for NvMapFromId {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapFromId
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

/// Represents flags used in [`NvMapAlloc`] commands
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum AllocFlags {
    #[default]
    ReadOnly = 0,
    ReadWrite = 1
}

/// Represents the `Alloc` command for [`NvMap`][`IoctlFd::NvMap`] fd
/// 
/// See <https://switchbrew.org/wiki/NV_services#NVMAP_IOC_ALLOC>
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct NvMapAlloc {
    /// The input handle
    pub handle: u32,
    /// The input heao mask
    pub heap_mask: u32,
    /// The input [`AllocFlags`]
    pub flags: AllocFlags,
    /// The input align
    pub align: u32,
    /// The input [`Kind`]
    pub kind: Kind,
    /// Padding
    pub pad: [u8; 4],
    /// The input address
    pub address: usize
}

impl Ioctl for NvMapAlloc {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapAlloc
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

/// Represents the `GetId` command for [`NvMap`][`IoctlFd::NvMap`] fd
/// 
/// See <https://switchbrew.org/wiki/NV_services#NVMAP_IOC_GET_ID>
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct NvMapGetId {
    /// The output ID
    pub id: u32,
    /// The input handle
    pub handle: u32
}

impl Ioctl for NvMapGetId {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapGetId
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

/// Represents the `SyncptWait` command for [`NvHostCtrl`][`IoctlFd::NvHostCtrl`] fd
/// 
/// See <https://switchbrew.org/wiki/NV_services#NVHOST_IOCTL_CTRL_SYNCPT_WAIT>
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct NvHostCtrlSyncptWait {
    /// The input [`Fence`]
    pub fence: Fence,
    /// The input timeout
    pub timeout: i32
}

impl Ioctl for NvHostCtrlSyncptWait {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvHostCtrlSyncptWait
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvHostCtrl
    }
}