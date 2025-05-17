use crate::ipc::sf::sm;
use crate::result::*;
use crate::service;

pub use crate::ipc::sf::usb::hs::*;

impl service::IService for ClientRootSession {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("usb:hs")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
