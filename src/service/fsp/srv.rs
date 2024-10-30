use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::fsp::srv::*;

impl service::IService for FileSystemProxy {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("fsp-srv")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.set_current_process(sf::ProcessId::new(), Default::default())
    }
}