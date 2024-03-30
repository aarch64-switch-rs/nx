use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;

pub use crate::ipc::sf::spl::*;

ipc_client_define_object_default!(RandomInterface);

impl IRandomInterface for RandomInterface {
    fn generate_random_bytes(&mut self, out_buf: sf::OutMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (out_buf) => ())
    }
}

impl service::IService for RandomInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("csrng")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}