use crate::result::*;
use crate::ipc::sf::{sm};
use crate::service;

pub use crate::ipc::sf::dispdrv::*;

impl service::IService for HOSBinderDriver {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("dispdrv")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}