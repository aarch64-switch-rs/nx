use crate::result::*;
use crate::ipc::sf;

pub trait IRandomInterface {
    ipc_interface_define_command!(generate_random_bytes: (out_buf: sf::OutMapAliasBuffer) => ());
}