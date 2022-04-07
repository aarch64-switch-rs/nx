use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::fspsrv::*;

pub struct Directory {
    session: sf::Session
}

impl sf::IObject for Directory {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(read: 0),
            ipc_cmif_interface_make_command_meta!(get_entry_count: 1)
        ]
    }
}

impl service::IClientObject for Directory {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl IDirectory for Directory {
    fn read(&mut self, out_entries: sf::OutMapAliasBuffer) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 0] (out_entries) => (read_count: u64))
    }

    fn get_entry_count(&mut self) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => (count: u64))
    }
}

pub struct File {
    session: sf::Session
}

impl sf::IObject for File {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(read: 0),
            ipc_cmif_interface_make_command_meta!(write: 1),
            ipc_cmif_interface_make_command_meta!(flush: 2),
            ipc_cmif_interface_make_command_meta!(set_size: 3),
            ipc_cmif_interface_make_command_meta!(get_size: 4),
            ipc_cmif_interface_make_command_meta!(operate_range: 5, [(4, 0, 0) =>]),
            ipc_cmif_interface_make_command_meta!(operate_range_with_buffer: 6, [(12, 0, 0) =>])
        ]
    }
}

impl service::IClientObject for File {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl IFile for File {
    fn read(&mut self, option: FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 0] (option, offset, size, buf) => (read_size: usize))
    }

    fn write(&mut self, option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (option, offset, size, buf) => ())
    }

    fn flush(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] () => ())
    }

    fn set_size(&mut self, size: usize) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (size) => ())
    }

    fn get_size(&mut self) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 4] () => (size: usize))
    }

    fn operate_range(&mut self, operation_id: OperationId, offset: usize, size: usize) -> Result<FileQueryRangeInfo> {
        ipc_client_send_request_command!([self.session.object_info; 5] (operation_id, offset, size) => (info: FileQueryRangeInfo))
    }

    fn operate_range_with_buffer(&mut self, operation_id: OperationId, offset: usize, size: usize, in_buf: sf::InNonSecureMapAliasBuffer, out_buf: sf::OutNonSecureMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 6] (operation_id, offset, size, in_buf, out_buf) => ())
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
        vec! [
            ipc_cmif_interface_make_command_meta!(create_file: 0),
            ipc_cmif_interface_make_command_meta!(delete_file: 1),
            ipc_cmif_interface_make_command_meta!(create_directory: 2),
            ipc_cmif_interface_make_command_meta!(delete_directory: 3),
            ipc_cmif_interface_make_command_meta!(delete_directory_recursively: 4),
            ipc_cmif_interface_make_command_meta!(rename_file: 5),
            ipc_cmif_interface_make_command_meta!(rename_directory: 6),
            ipc_cmif_interface_make_command_meta!(get_entry_type: 7),
            ipc_cmif_interface_make_command_meta!(open_file: 8),
            ipc_cmif_interface_make_command_meta!(open_directory: 9),
            ipc_cmif_interface_make_command_meta!(commit: 10),
            ipc_cmif_interface_make_command_meta!(get_free_space_size: 11),
            ipc_cmif_interface_make_command_meta!(get_total_space_size: 12),
            ipc_cmif_interface_make_command_meta!(clean_directory_recursively: 13, [(3, 0, 0) =>]),
            ipc_cmif_interface_make_command_meta!(get_file_time_stamp_raw: 14, [(3, 0, 0) =>]),
            ipc_cmif_interface_make_command_meta!(query_entry: 15, [(4, 0, 0) =>])
        ]
    }
}

impl service::IClientObject for FileSystem {
    fn new(session: sf::Session) -> Self {
        Self { session }
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

    fn rename_file(&mut self, old_path_buf: sf::InPointerBuffer, new_path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 5] (old_path_buf, new_path_buf) => ())
    }

    fn rename_directory(&mut self, old_path_buf: sf::InPointerBuffer, new_path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 6] (old_path_buf, new_path_buf) => ())
    }

    fn get_entry_type(&mut self, path_buf: sf::InPointerBuffer) -> Result<DirectoryEntryType> {
        ipc_client_send_request_command!([self.session.object_info; 7] (path_buf) => (entry_type: DirectoryEntryType))
    }
    
    fn open_file(&mut self, mode: FileOpenMode, path_buf: sf::InPointerBuffer) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 8] (mode, path_buf) => (file: mem::Shared<File>))
    }

    fn open_directory(&mut self, mode: DirectoryOpenMode, path_buf: sf::InPointerBuffer) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 9] (mode, path_buf) => (dir: mem::Shared<Directory>))
    }

    fn commit(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] () => ())
    }

    fn get_free_space_size(&mut self, path_buf: sf::InPointerBuffer) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 11] (path_buf) => (size: usize))
    }

    fn get_total_space_size(&mut self, path_buf: sf::InPointerBuffer) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 12] (path_buf) => (size: usize))
    }

    fn clean_directory_recursively(&mut self, path_buf: sf::InPointerBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 13] (path_buf) => ())
    }

    fn get_file_time_stamp_raw(&mut self, path_buf: sf::InPointerBuffer) -> Result<FileTimeStampRaw> {
        ipc_client_send_request_command!([self.session.object_info; 14] (path_buf) => (time_stamp: FileTimeStampRaw))
    }

    fn query_entry(&mut self, path_buf: sf::InPointerBuffer, query_id: QueryId, in_buf: sf::InNonSecureMapAliasBuffer, out_buf: sf::OutNonSecureMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 15] (path_buf, query_id, in_buf, out_buf) => ())
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
        vec! [
            ipc_cmif_interface_make_command_meta!(set_current_process: 1),
            ipc_cmif_interface_make_command_meta!(open_sd_card_filesystem: 18),
            ipc_cmif_interface_make_command_meta!(output_access_log_to_sd_card: 1006)
        ]
    }
}

impl service::IClientObject for FileSystemProxy {
    fn new(session: sf::Session) -> Self {
        Self { session }
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