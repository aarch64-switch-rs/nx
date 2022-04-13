use crate::result::*;
use crate::version;

ipc_sf_define_interface_trait! {
    trait IPsmServer {
        get_battery_charge_percentage [0, version::VersionInterval::all()]: () => (charge: u32);   
    }
}