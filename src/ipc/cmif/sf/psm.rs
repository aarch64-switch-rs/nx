use crate::result::*;
// use crate::ipc::cmif::sf;

pub trait IPsmServer {
    nipc_cmif_interface_define_command!(get_battery_charge_percentage: () => (charge: u32));
}