use super::*;
use crate::service;

pub use crate::ipc::sf::ncm::*;

impl service::IService for ContentManager {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("ncm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
