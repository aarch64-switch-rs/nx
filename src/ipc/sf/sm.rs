use crate::result::*;
use crate::ipc::sf;
use crate::input;

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ServiceName {
    pub value: u64,
}

impl ServiceName {
    pub const fn from(value: u64) -> Self {
        Self { value: value }
    }
    
    pub const fn new(name: &str) -> Self {
        // Note: for the name to be valid, it should end with at least a NUL terminator (use the nul!("name") macro present in this crate for that)
        let value = unsafe { *(name.as_ptr() as *const u64) };
        Self::from(value)
    }

    pub const fn is_empty(&self) -> bool {
        self.value == 0
    }

    pub const fn empty() -> Self {
        Self::from(0)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MitmProcessInfo {
    pub process_id: u64,
    pub program_id: u64,
    pub keys_held: input::Key,
    pub override_flags: u64
}

pub trait IUserInterface {
    ipc_interface_define_command!(initialize: (process_id: sf::ProcessId) => ());
    ipc_interface_define_command!(get_service_handle: (name: ServiceName) => (service_handle: sf::MoveHandle));
    ipc_interface_define_command!(register_service: (name: ServiceName, is_light: bool, max_sessions: i32) => (port_handle: sf::MoveHandle));
    ipc_interface_define_command!(unregister_service: (name: ServiceName) => ());
    ipc_interface_define_command!(atmosphere_install_mitm: (name: ServiceName) => (port_handle: sf::MoveHandle, query_handle: sf::MoveHandle));
    ipc_interface_define_command!(atmosphere_uninstall_mitm: (name: ServiceName) => ());
    ipc_interface_define_command!(atmosphere_acknowledge_mitm_session: (name: ServiceName) => (info: MitmProcessInfo, session_handle: sf::MoveHandle));
    ipc_interface_define_command!(atmosphere_has_service: (name: ServiceName) => (has: bool));
}