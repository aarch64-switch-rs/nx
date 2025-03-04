use crate::ipc::sf;
use crate::result::*;
use crate::version;

ipc_sf_define_default_interface_client!(RandomInterface);
ipc_sf_define_interface_trait! {
    trait RandomInterface {
        generate_random_bytes [0, version::VersionInterval::all()]: (out_buf: sf::OutMapAliasBuffer<u8>) =>  () ();
    }
}
