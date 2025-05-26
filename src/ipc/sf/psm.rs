use crate::version;

//ipc_sf_define_default_client_for_interface!(Psm);
ipc_sf_define_interface_trait! {
    trait Psm {
        get_battery_charge_percentage [0, version::VersionInterval::all()]: () => (charge: u32);
    }
}
