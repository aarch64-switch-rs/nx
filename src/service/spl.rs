use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;

pub use crate::ipc::sf::spl::*;

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