use crate::ipc::sf::sm;
use crate::result::*;
use crate::service;

pub use crate::ipc::sf::nfp::*;

ipc_client_define_client_default!(UserManagerService);
impl IUserManagerServiceClient for UserManagerService {}

ipc_client_define_client_default!(DebugManagerService);
impl IDebugManagerServiceClient for DebugManagerService {}

ipc_client_define_client_default!(SystemManagerService);
impl ISystemManagerServiceClient for SystemManagerService {}

impl service::IService for UserManagerService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nfp:user")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl service::IService for SystemManagerService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nfp:sys")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

impl service::IService for DebugManagerService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nfp:dbg")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
