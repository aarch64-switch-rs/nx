use crate::ipc::sf::sm;
use crate::result::*;
use crate::service;

pub use crate::ipc::sf::set::*;

impl service::IService for SystemSettingsServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("set:sys")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
