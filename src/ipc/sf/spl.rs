use crate::ipc::sf;
use crate::version;

ipc_sf_define_interface_trait! {
    trait Random {
        generate_random_bytes [0, version::VersionInterval::all()]: (out_buf: sf::OutMapAliasBuffer<u8>) => ();
    }
}
