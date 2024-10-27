use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::hid::*;

impl service::IService for HidServer {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("hid")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}