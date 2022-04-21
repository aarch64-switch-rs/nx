use crate::ipc::server;
use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::ipc::sf::fsp;
use crate::ipc::sf::fsp::IDirectory;
use crate::ipc::sf::fsp::IFile;
use crate::ipc::sf::fsp::IFileSystem;

// These types are helper object types to translate from our fs objects to IPC fs objects

pub struct Directory {
    dir_obj: mem::Shared<dyn super::Directory>
}

impl Directory {
    pub fn new(dir_obj: mem::Shared<dyn super::Directory>) -> Self {
        Self { dir_obj }
    }
}

impl sf::IObject for Directory {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IDirectory for Directory {
    fn read(&mut self, out_entries: sf::OutMapAliasBuffer<fsp::DirectoryEntry>) -> Result<u64> {
        self.dir_obj.get().read(out_entries.get_mut_slice())
    }

    fn get_entry_count(&mut self) -> Result<u64> {
        self.dir_obj.get().get_entry_count()
    }
}

impl server::ISessionObject for Directory {}

pub struct File {
    file_obj: mem::Shared<dyn super::File>
}

impl File {
    pub fn new(file_obj: mem::Shared<dyn super::File>) -> Self {
        Self { file_obj }
    }
}

impl sf::IObject for File {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IFile for File {
    fn read(&mut self, option: fsp::FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<usize> {
        self.file_obj.get().read(offset, buf.get_address(), size.min(buf.get_size()), option)
    }

    fn write(&mut self, option: fsp::FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer<u8>) -> Result<()> {
        self.file_obj.get().write(offset, buf.get_address(), size.min(buf.get_size()), option)
    }

    fn flush(&mut self) -> Result<()> {
        self.file_obj.get().flush()
    }

    fn set_size(&mut self, size: usize) -> Result<()> {
        self.file_obj.get().set_size(size)
    }

    fn get_size(&mut self) -> Result<usize> {
        self.file_obj.get().get_size()
    }

    fn operate_range(&mut self, operation_id: fsp::OperationId, offset: usize, size: usize) -> Result<fsp::FileQueryRangeInfo> {
        self.file_obj.get().operate_range(operation_id, offset, size)
    }

    fn operate_range_with_buffer(&mut self, operation_id: fsp::OperationId, offset: usize, size: usize, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<()> {
        self.file_obj.get().operate_range_with_buffer(operation_id, offset, size, in_buf.get_address(), in_buf.get_size(), out_buf.get_address(), out_buf.get_size())
    }
}

impl server::ISessionObject for File {}

pub struct FileSystem {
    fs_obj: mem::Shared<dyn super::FileSystem>
}

impl FileSystem {
    pub fn new(fs_obj: mem::Shared<dyn super::FileSystem>) -> Self {
        Self { fs_obj }
    }
}

impl sf::IObject for FileSystem {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IFileSystem for FileSystem {
    fn create_file(&mut self, attribute: fsp::FileAttribute, size: usize, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().create_file(fs_path, attribute, size)
    }

    fn delete_file(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().delete_file(fs_path)
    }

    fn create_directory(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().create_directory(fs_path)
    }
    
    fn delete_directory(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().delete_directory(fs_path)
    }

    fn delete_directory_recursively(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().delete_directory_recursively(fs_path)
    }

    fn rename_file(&mut self, old_path_buf: sf::InFixedPointerBuffer<fsp::Path>, new_path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let old_fs_path = old_path_buf.get_var().get_string()?;
        let new_fs_path = new_path_buf.get_var().get_string()?;
        self.fs_obj.get().rename_file(old_fs_path, new_fs_path)
    }

    fn rename_directory(&mut self, old_path_buf: sf::InFixedPointerBuffer<fsp::Path>, new_path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let old_fs_path = old_path_buf.get_var().get_string()?;
        let new_fs_path = new_path_buf.get_var().get_string()?;
        self.fs_obj.get().rename_directory(old_fs_path, new_fs_path)
    }

    fn get_entry_type(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<fsp::DirectoryEntryType> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().get_entry_type(fs_path)
    }
    
    fn open_file(&mut self, mode: fsp::FileOpenMode, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<mem::Shared<dyn fsp::IFile>> {
        let fs_path = path_buf.get_var().get_string()?;
        let file_obj = self.fs_obj.get().open_file(fs_path, mode)?;
        Ok(mem::Shared::new(File::new(file_obj)))
    }

    fn open_directory(&mut self, mode: fsp::DirectoryOpenMode, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<mem::Shared<dyn fsp::IDirectory>> {
        let fs_path = path_buf.get_var().get_string()?;
        let dir_obj = self.fs_obj.get().open_directory(fs_path, mode)?;
        Ok(mem::Shared::new(Directory::new(dir_obj)))
    }

    fn commit(&mut self) -> Result<()> {
        self.fs_obj.get().commit()
    }

    fn get_free_space_size(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<usize> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().get_free_space_size(fs_path)
    }

    fn get_total_space_size(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<usize> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().get_total_space_size(fs_path)
    }

    fn clean_directory_recursively(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().clean_directory_recursively(fs_path)
    }

    fn get_file_time_stamp_raw(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<fsp::FileTimeStampRaw> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().get_file_time_stamp_raw(fs_path)
    }

    fn query_entry(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>, query_id: fsp::QueryId, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.get().query_entry(fs_path, query_id, in_buf.get_address(), in_buf.get_size(), out_buf.get_address(), out_buf.get_size())
    }
}

impl server::ISessionObject for FileSystem {}