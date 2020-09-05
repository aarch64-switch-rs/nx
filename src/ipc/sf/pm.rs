use crate::result::*;
// use crate::ipc::sf;

pub trait IInformationInterface {
    ipc_interface_define_command!(get_program_id: (process_id: u64) => (program_id: u64));
}