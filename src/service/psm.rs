use crate::result::*;
use crate::ipc::sf::{sm};
use crate::service;

pub use crate::ipc::sf::psm::*;

impl service::IService for PsmServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("psm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}