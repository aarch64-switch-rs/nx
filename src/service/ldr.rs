use crate::ipc::sf::{ncm, sm};
use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::ldr::*;

impl service::IService for ShellInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("ldr:shel")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
