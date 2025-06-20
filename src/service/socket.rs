use crate::ipc::sf::sm;
use crate::result::Result;
use crate::service;

pub use crate::ipc::sf::socket::*;

ipc_client_define_client_default!(SystemSocketService);
impl ISocketClient for SystemSocketService {}

impl service::IService for SystemSocketService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("bsd:s")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

ipc_client_define_client_default!(AppletSocketService);
impl ISocketClient for AppletSocketService {}

impl service::IService for AppletSocketService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("bsd:a")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

ipc_client_define_client_default!(UserSocketService);
impl ISocketClient for UserSocketService {}

impl service::IService for UserSocketService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("bsd:u")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}


pub enum BsdSrvkind {
    /// "bsd:u"
    User,
    /// "bsd:a"
    Applet,
    /// "bsd:s"
    System,
}

