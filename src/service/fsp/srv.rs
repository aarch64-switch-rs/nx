use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::ipc::client;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::fsp::srv::*;

pub struct FileSystemProxy {
    session: sf::Session
}

impl sf::IObject for FileSystemProxy {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IFileSystemProxy for FileSystemProxy {
    fn set_current_process(&mut self, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (process_id) => ())
    }

    fn open_sd_card_filesystem(&mut self) -> Result<mem::Shared<dyn super::IFileSystem>> {
        ipc_client_send_request_command!([self.session.object_info; 18] () => (sd_filesystem: mem::Shared<super::FileSystem>))
    }

    fn output_access_log_to_sd_card(&mut self, access_log: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1006] (access_log) => ())
    }
}

impl client::IClientObject for FileSystemProxy {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for FileSystemProxy {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("fsp-srv")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.set_current_process(sf::ProcessId::new())
    }
}