use crate::result::*;
// use crate::ipc::sf;

pub trait IPsmServer {
    ipc_cmif_interface_define_command!(get_battery_charge_percentage: () => (charge: u32));
}