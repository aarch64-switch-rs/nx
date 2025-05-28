use crate::ipc::sf;

//ipc_sf_define_default_client_for_interface!(RandomService);
#[nx_derive::ipc_trait]
pub trait Random {
    #[ipc_rid(0)]
    fn generate_random_bytes(&self, out_buf: sf::OutMapAliasBuffer<u8>);
}
