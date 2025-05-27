use crate::version;

ipc_sf_define_interface_trait! {
    trait Psm {
        get_battery_charge_percentage [0, version::VersionInterval::all()]: () => (charge: u32);
    }
}
