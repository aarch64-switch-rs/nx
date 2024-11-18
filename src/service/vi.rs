use crate::ipc::sf::sm;
use crate::result::*;
use crate::service;

pub use crate::ipc::sf::vi::*;

impl service::IService for ApplicationRootService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("vi:u")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl service::IService for SystemRootService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("vi:s")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl service::IService for ManagerRootService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("vi:m")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}