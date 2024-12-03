use crate::result::*;
use crate::version;

ipc_sf_define_default_interface_client!(PsmServer);
ipc_sf_define_interface_trait! {
	trait PsmServer {
        get_battery_charge_percentage [0, version::VersionInterval::all()]: () => (charge: u32) (charge: u32);   
    }
}