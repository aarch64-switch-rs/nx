use crate::result::*;
use crate::ipc::cmif::sf;

pub trait IRandomInterface {
    nipc_cmif_interface_define_command!(generate_random_bytes: (out_buf: sf::OutMapAliasBuffer) => ());
}