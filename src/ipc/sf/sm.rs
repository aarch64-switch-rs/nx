use crate::ipc::sf;
use crate::version;
use crate::util;

pub mod mitm;

pub mod rc;

use nx_derive::{Request, Response};

#[derive(Request, Response, Copy, Clone, Eq)]
#[repr(C)]
pub union ServiceName {
    pub value: u64,
    pub name: [u8; 8],
}
const_assert!(core::mem::size_of::<ServiceName>() == 0x8);

impl ServiceName {
    pub const fn from(value: u64) -> Self {
        Self { value }
    }

    pub const fn new(name: &str) -> Self {
        let mut raw_name: [u8; 8] = [0; 8];

        let name_bytes = name.as_bytes();
        let copy_len = util::const_usize_min(8, name_bytes.len());

        unsafe {
            core::ptr::copy_nonoverlapping(name_bytes.as_ptr(), raw_name.as_mut_ptr(), copy_len)
        }

        Self { name: raw_name }
    }

    pub const fn empty() -> Self {
        Self::from(0)
    }

    pub const fn is_empty(&self) -> bool {
        unsafe { self.value == 0 }
    }
}

impl PartialEq for ServiceName {
    fn eq(&self, other: &Self) -> bool {
        unsafe { self.value == other.value }
    }
}

impl core::fmt::Debug for ServiceName {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        unsafe { self.name.fmt(f) }
    }
}

ipc_sf_define_default_client_for_interface!(UserInterface);
ipc_sf_define_interface_trait! {
    trait UserInterface {
        register_client [0, version::VersionInterval::all()]: (process_id: sf::ProcessId) => ();
        get_service_handle [1, version::VersionInterval::all()]: (name: ServiceName) => (service_handle: sf::MoveHandle);
        register_service [2, version::VersionInterval::all()]: (name: ServiceName, is_light: bool, max_sessions: i32) => (port_handle: sf::MoveHandle);
        unregister_service [3, version::VersionInterval::all()]: (name: ServiceName) => ();
        detach_client [4, version::VersionInterval::from(version::Version::new(11,0,0)), mut]: (process_id: sf::ProcessId) => ();
        atmosphere_install_mitm [65000, version::VersionInterval::all()]: (name: ServiceName) =>  (port_handle: sf::MoveHandle, query_handle: sf::MoveHandle);
        atmosphere_uninstall_mitm [65001, version::VersionInterval::all()]: (name: ServiceName) => ();
        atmosphere_acknowledge_mitm_session [65003, version::VersionInterval::all()]: (name: ServiceName) =>  (info: mitm::MitmProcessInfo, session_handle: sf::MoveHandle);
        atmosphere_has_mitm [65004, version::VersionInterval::all()]: (name: ServiceName) => (has: bool);
        atmosphere_wait_mitm [65005, version::VersionInterval::all()]: (name: ServiceName) => ();
        atmosphere_declare_future_mitm [65006, version::VersionInterval::all()]: (name: ServiceName) => ();
        atmosphere_clear_future_mitm [65007, version::VersionInterval::all()]: (name: ServiceName) => ();
        atmosphere_has_service [65100, version::VersionInterval::all()]: (name: ServiceName) => (has: bool);
        atmosphere_wait_service [65101, version::VersionInterval::all()]: (name: ServiceName) => ();
    }
}
