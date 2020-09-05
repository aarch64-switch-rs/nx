use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::fspsrv::*;

pub struct File {
    session: sf::Session
}

impl sf::IObject for File {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            read: 0,
            write: 1,
            get_size: 4
        }
    }
}

impl service::IClientObject for File {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IFile for File {
    fn read(&mut self, option: FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 0] (option, offset, size, buf) => (read_size: usize))
    }

    fn write(&mut self, option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (option, offset, size, buf) => ())
    }

    fn get_size(&mut self) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 4] () => (size: usize))
    }
}

pub struct FileSystem {
    session: sf::Session
}

impl sf::IObject for FileSystem {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            create_file: 0,
            delete_file: 1,
            create_directory: 2,
            delete_directory: 3,
            delete_directory_recursively: 4,
            open_file: 8
        }
    }
}

impl service::IClientObject for FileSystem {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IFileSystem for FileSystem {
    fn create_file(&mut self, attribute: FileAttribute, size: usize, path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (attribute, size, path_buf) => ())
    }

    fn delete_file(&mut self, path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (path_buf) => ())
    }

    fn create_directory(&mut self, path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] (path_buf) => ())
    }
    
    fn delete_directory(&mut self, path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (path_buf) => ())
    }

    fn delete_directory_recursively(&mut self, path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (path_buf) => ())
    }

    fn get_entry_type(&mut self, path_buf: sf::InPointerBuffer) -> Result<DirectoryEntryType> {
        ipc_client_send_request_command!([self.session.object_info; 7] (path_buf) => (entry_type: DirectoryEntryType))
    }
    
    fn open_file(&mut self, mode: FileOpenMode, path_buf: sf::InPointerBuffer) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 8] (mode, path_buf) => (file: mem::Shared<File>))
    }
}

pub struct FileSystemProxy {
    session: sf::Session
}

impl sf::IObject for FileSystemProxy {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            set_current_process: 1,
            open_sd_card_filesystem: 18,
            output_access_log_to_sd_card: 1006
        }
    }
}

impl service::IClientObject for FileSystemProxy {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IFileSystemProxy for FileSystemProxy {
    fn set_current_process(&mut self, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (process_id) => ())
    }

    fn open_sd_card_filesystem(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 18] () => (sd_filesystem: mem::Shared<FileSystem>))
    }

    fn output_access_log_to_sd_card(&mut self, access_log: sf::InMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1006] (access_log) => ())
    }
}

impl service::IService for FileSystemProxy {
    fn get_name() -> &'static str {
        nul!("fsp-srv")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.set_current_process(sf::ProcessId::new())
    }
}