use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;
use crate::ipc::sf::usb;

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