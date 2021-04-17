use crate::result::*;
use crate::ipc::cmif::sf;
use crate::ipc::tipc::sf::sm;

// Interfaces related to core serverside IPC (for control requests and MitM support)

pub trait IHipcManager {
    nipc_cmif_control_interface_define_command!(convert_current_object_to_domain: () => (domain_object_id: u32));
    nipc_cmif_control_interface_define_command!(copy_from_current_domain: (domain_object_id: u32) => (handle: sf::MoveHandle));
    nipc_cmif_control_interface_define_command!(clone_current_object: () => (cloned_handle: sf::MoveHandle));
    nipc_cmif_control_interface_define_command!(query_pointer_buffer_size: () => (pointer_buffer_size: u16));
    nipc_cmif_control_interface_define_command!(clone_current_object_ex: (tag: u32) => (cloned_handle: sf::MoveHandle));
}

pub trait IMitmQueryServer {
    nipc_cmif_interface_define_command!(should_mitm: (info: sm::MitmProcessInfo) => (should_mitm: bool));
}