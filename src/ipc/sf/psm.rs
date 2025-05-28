
//ipc_sf_define_default_client_for_interface!(Psm);
#[nx_derive::ipc_trait]
pub trait Psm {
    #[ipc_rid(0)]
    fn get_battery_charge_percentage(&self) -> u32;
}
