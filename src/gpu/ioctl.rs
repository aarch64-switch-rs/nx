use crate::service::nv;
use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum IoctlFd {
    NvHost,
    NvMap,
    NvHostCtrl,
}

pub trait Ioctl {
    fn get_id() -> nv::IoctlId;
    fn get_fd() -> IoctlFd;
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NvMapCreate {
    pub size: u32,
    pub handle: u32,
}

impl Ioctl for NvMapCreate {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapCreate
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NvMapFromId {
    pub id: u32,
    pub handle: u32,
}

impl Ioctl for NvMapFromId {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapFromId
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum AllocFlags {
    ReadOnly,
    ReadWrite,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NvMapAlloc {
    pub handle: u32,
    pub heap_mask: u32,
    pub flags: AllocFlags,
    pub align: u32,
    pub kind: Kind,
    pub pad: [u8; 4],
    pub address: *mut u8,
}

impl Ioctl for NvMapAlloc {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapAlloc
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NvMapGetId {
    pub id: u32,
    pub handle: u32,
}

impl Ioctl for NvMapGetId {
    fn get_id() -> nv::IoctlId {
        nv::IoctlId::NvMapGetId
    }

    fn get_fd() -> IoctlFd {
        IoctlFd::NvMap
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct NvHostCtrlSyncptWait {
    pub fence: Fence,
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