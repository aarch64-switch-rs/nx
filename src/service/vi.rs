use crate::ipc::sf::sm;
use crate::result::*;
use crate::service;

pub use crate::ipc::sf::vi::*;

ipc_client_define_client_default!(ApplicationDisplayRootService);
ipc_client_define_client_default!(ManagerDisplayRootService);
ipc_client_define_client_default!(SystemDisplayRootService);

impl IApplicationDisplayRootClient for ApplicationDisplayRootService {}
impl IManagerDisplayRootClient for ManagerDisplayRootService {}
impl ISystemDisplayRootClient for SystemDisplayRootService {}

impl service::IService for ApplicationDisplayRootService {
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

impl service::IService for SystemDisplayRootService {
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

impl service::IService for ManagerDisplayRootService {
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

impl CommonDisplayRootClient for ApplicationDisplayRootService {
    fn get_display_service(&self) -> Result<ApplicationDisplay> {
        <Self as IApplicationDisplayRootClient>::get_display_service(self, DisplayServiceMode::User)
    }
}

impl CommonDisplayRootClient for SystemDisplayRootService {
    fn get_display_service(&self) -> Result<ApplicationDisplay> {
        <Self as ISystemDisplayRootClient>::get_display_service(self, DisplayServiceMode::Privileged)
    }
}

impl CommonDisplayRootClient for ManagerDisplayRootService {
    fn get_display_service(&self) -> Result<ApplicationDisplay> {
        <Self as IManagerDisplayRootClient>::get_display_service(self, DisplayServiceMode::Privileged)
    }
}