use crate::result::*;
use crate::ipc;
use crate::ipc::sf;
use crate::service;
use crate::version;

pub use crate::ipc::sf::sm::*;

impl service::INamedPort for UserInterface {
    fn get_name() -> &'static str {
        nul!("sm:")
    }

    fn post_initialize(&mut self) -> Result<()> {
        if version::get_version() >= version::Version::new(12,0,0) {
            self.session.object_info.protocol = ipc::CommandProtocol::Tipc;
        }

        self.register_client(sf::ProcessId::new(), Default::default())
    }
}