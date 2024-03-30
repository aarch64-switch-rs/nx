use crate::result::*;
use crate::ipc::sf;
use crate::version;

ipc_sf_define_interface_trait! {
    trait IRandomInterface {
        generate_random_bytes [0, version::VersionInterval::all()]: (out_buf: sf::OutMapAliasBuffer<u8>) => ();
    }
}