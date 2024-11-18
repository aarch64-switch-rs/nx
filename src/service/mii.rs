use crate::ipc::sf::sm;
use crate::result::*;
use crate::service;

pub use crate::ipc::sf::mii::*;

impl service::IService for StaticService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("mii:e")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}