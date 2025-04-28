use crate::result::*;
use crate::version;

ipc_sf_define_default_interface_client!(Psm);
ipc_sf_define_interface_trait! {
    trait Psm {
        get_battery_charge_percentage [0, version::VersionInterval::all()]: () => (charge: u32) (charge: u32);
    }
}
