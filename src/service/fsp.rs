use crate::result::*;
use crate::ipc::sf;
use crate::mem;

pub use crate::ipc::sf::fsp::*;

ipc_sf_client_object_define_default_impl!(Directory);

impl IDirectory for Directory {
    fn read(&mut self, out_entries: sf::OutMapAliasBuffer<DirectoryEntry>) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 0] (out_entries) => (read_count: u64))
    }

    fn get_entry_count(&mut self) -> Result<u64> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => (count: u64))
    }
}

ipc_sf_client_object_define_default_impl!(File);

impl IFile for File {
    fn read(&mut self, option: FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 0] (option, offset, size, buf) => (read_size: usize))
    }

    fn write(&mut self, option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer<u8>) -> Result<()> {
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

    fn operate_range_with_buffer(&mut self, operation_id: OperationId, offset: usize, size: usize, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 6] (operation_id, offset, size, in_buf, out_buf) => ())
    }
}

ipc_sf_client_object_define_default_impl!(FileSystem);

impl IFileSystem for FileSystem {
    fn create_file(&mut self, attribute: FileAttribute, size: usize, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (attribute, size, path_buf) => ())
    }

    fn delete_file(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (path_buf) => ())
    }

    fn create_directory(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] (path_buf) => ())
    }
    
    fn delete_directory(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (path_buf) => ())
    }

    fn delete_directory_recursively(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (path_buf) => ())
    }

    fn rename_file(&mut self, old_path_buf: sf::InFixedPointerBuffer<Path>, new_path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 5] (old_path_buf, new_path_buf) => ())
    }

    fn rename_directory(&mut self, old_path_buf: sf::InFixedPointerBuffer<Path>, new_path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 6] (old_path_buf, new_path_buf) => ())
    }

    fn get_entry_type(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<DirectoryEntryType> {
        ipc_client_send_request_command!([self.session.object_info; 7] (path_buf) => (entry_type: DirectoryEntryType))
    }
    
    fn open_file(&mut self, mode: FileOpenMode, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<mem::Shared<dyn IFile>> {
        ipc_client_send_request_command!([self.session.object_info; 8] (mode, path_buf) => (file: mem::Shared<File>))
    }

    fn open_directory(&mut self, mode: DirectoryOpenMode, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<mem::Shared<dyn IDirectory>> {
        ipc_client_send_request_command!([self.session.object_info; 9] (mode, path_buf) => (dir: mem::Shared<Directory>))
    }

    fn commit(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] () => ())
    }

    fn get_free_space_size(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 11] (path_buf) => (size: usize))
    }

    fn get_total_space_size(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 12] (path_buf) => (size: usize))
    }

    fn clean_directory_recursively(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 13] (path_buf) => ())
    }

    fn get_file_time_stamp_raw(&mut self, path_buf: sf::InFixedPointerBuffer<Path>) -> Result<FileTimeStampRaw> {
        ipc_client_send_request_command!([self.session.object_info; 14] (path_buf) => (time_stamp: FileTimeStampRaw))
    }

    fn query_entry(&mut self, path_buf: sf::InFixedPointerBuffer<Path>, query_id: QueryId, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 15] (path_buf, query_id, in_buf, out_buf) => ())
    }
}

pub mod srv;