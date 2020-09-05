use crate::result::*;
use crate::ipc::sf;
use crate::ipc::sf::sm;

// Interfaces related to core serverside IPC (for control requests and MitM support)

pub trait IHipcManager {
    ipc_control_interface_define_command!(convert_current_object_to_domain: () => (domain_object_id: u32));
    ipc_control_interface_define_command!(copy_from_current_domain: (domain_object_id: u32) => (handle: sf::MoveHandle));
    ipc_control_interface_define_command!(clone_current_object: () => (cloned_handle: sf::MoveHandle));
    ipc_control_interface_define_command!(query_pointer_buffer_size: () => (pointer_buffer_size: u16));
    ipc_control_interface_define_command!(clone_current_object_ex: (tag: u32) => (cloned_handle: sf::MoveHandle));
}

pub trait IMitmQueryServer {
    ipc_interface_define_command!(should_mitm: (info: sm::MitmProcessInfo) => (should_mitm: bool));
}