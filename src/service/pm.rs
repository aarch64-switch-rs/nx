use crate::result::*;
use crate::ipc::sf::{self, ncm, sm};
use crate::service;

pub use crate::ipc::sf::pm::*;

impl service::IService for InformationInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("pm:info")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl service::IService for DebugMonitorInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("pm:dmnt")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}