use super::*;
use crate::service;
use self::sf::ncm;

pub use crate::ipc::sf::lr::*;

impl service::IService for LocationResolverManager {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("lr")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
