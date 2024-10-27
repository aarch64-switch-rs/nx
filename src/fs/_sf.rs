//! Helper object types to translate from our FS objects to IPC FS interfaces

use crate::ipc::client::IClientObject;
use crate::ipc::server;
use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::ipc::sf::fsp;
use crate::ipc::sf::fsp::IDirectory;
use crate::ipc::sf::fsp::IFile;
use crate::ipc::sf::fsp::IFileSystem;
use crate::service;

/// Represents a wrapper IPC [`IDirectory`] object around a [`Directory`][`super::Directory`] object
pub struct Directory {
    dir_obj: mem::Shared<dyn super::Directory>,
    dummy_session: sf::Session
}

impl Directory {
    /// Creates a new [`Directory`]
    /// 
    /// # Arguments
    /// 
    /// * `dir_obj`: The [`Directory`][`super::Directory`] object to wrap
    pub fn new(dir_obj: mem::Shared<dyn super::Directory>) -> Self {
        Self {
            dir_obj,
            dummy_session: sf::Session::new()
        }
    }
}

impl sf::IObject for Directory {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.dummy_session
    }
}

impl IDirectory for Directory {
    fn read(&mut self, mut out_entries: sf::OutMapAliasBuffer<fsp::DirectoryEntry>) -> Result<u64> {
        self.dir_obj.lock().read(out_entries.get_mut_slice())
    }

    fn get_entry_count(&mut self) -> Result<u64> {
        self.dir_obj.lock().get_entry_count()
    }
}

impl server::ISessionObject for Directory {}

/// Represents a wrapper IPC [`IFile`] object around a [`File`][`super::File`] object
pub struct File {
    file_obj: mem::Shared<dyn super::File>,
    dummy_session: sf::Session
}

impl File {
    /// Creates a new [`File`]
    /// 
    /// # Arguments
    /// 
    /// * `file_obj`: The [`File`][`super::File`] object to wrap
    pub fn new(file_obj: mem::Shared<dyn super::File>) -> Self {
        Self {
            file_obj,
            dummy_session: sf::Session::new()
        }
    }
}

impl sf::IObject for File {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.dummy_session
    }
}

impl IFile for File {
    fn read(&mut self, option: fsp::FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<usize> {
        self.file_obj.lock().read(offset, buf.get_address(), size.min(buf.get_size()), option)
    }

    fn write(&mut self, option: fsp::FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer<u8>) -> Result<()> {
        self.file_obj.lock().write(offset, buf.get_address(), size.min(buf.get_size()), option)
    }

    fn flush(&mut self) -> Result<()> {
        self.file_obj.lock().flush()
    }

    fn set_size(&mut self, size: usize) -> Result<()> {
        self.file_obj.lock().set_size(size)
    }

    fn get_size(&mut self) -> Result<usize> {
        self.file_obj.lock().get_size()
    }

    fn operate_range(&mut self, operation_id: fsp::OperationId, offset: usize, size: usize) -> Result<fsp::FileQueryRangeInfo> {
        self.file_obj.lock().operate_range(operation_id, offset, size)
    }

    fn operate_range_with_buffer(&mut self, operation_id: fsp::OperationId, offset: usize, size: usize, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<()> {
        self.file_obj.lock().operate_range_with_buffer(operation_id, offset, size, in_buf.get_address(), in_buf.get_size(), out_buf.get_address(), out_buf.get_size())
    }
}

impl server::ISessionObject for File {}

/// Represents a wrapper IPC [`IFileSystem`] object around a [`FileSystem`][`super::FileSystem`] object
pub struct FileSystem {
    fs_obj: mem::Shared<dyn super::FileSystem>,
    dummy_session: sf::Session
}

impl FileSystem {
    /// Creates a new [`FileSystem`]
    /// 
    /// # Arguments
    /// 
    /// * `fs_obj`: The [`FileSystem`][`super::FileSystem`] object to wrap
    pub fn new(fs_obj: mem::Shared<dyn super::FileSystem>) -> Self {
        Self {
            fs_obj,
            dummy_session: sf::Session::new()
        }
    }
}

impl sf::IObject for FileSystem {
    ipc_sf_object_impl_default_command_metadata!();

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.dummy_session
    }
}
/*
impl IFileSystem for FileSystem {
    fn create_file(&mut self, attribute: fsp::FileAttribute, size: usize, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().create_file(fs_path, attribute, size)
    }

    fn delete_file(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().delete_file(fs_path)
    }

    fn create_directory(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().create_directory(fs_path)
    }
    
    fn delete_directory(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().delete_directory(fs_path)
    }

    fn delete_directory_recursively(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().delete_directory_recursively(fs_path)
    }

    fn rename_file(&mut self, old_path_buf: sf::InFixedPointerBuffer<fsp::Path>, new_path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let old_fs_path = old_path_buf.get_var().get_string()?;
        let new_fs_path = new_path_buf.get_var().get_string()?;
        self.fs_obj.lock().rename_file(old_fs_path, new_fs_path)
    }

    fn rename_directory(&mut self, old_path_buf: sf::InFixedPointerBuffer<fsp::Path>, new_path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let old_fs_path = old_path_buf.get_var().get_string()?;
        let new_fs_path = new_path_buf.get_var().get_string()?;
        self.fs_obj.lock().rename_directory(old_fs_path, new_fs_path)
    }

    fn get_entry_type(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<fsp::DirectoryEntryType> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().get_entry_type(fs_path)
    }
    
    fn open_file(&mut self, mode: fsp::FileOpenMode, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<service::fsp::File> {
        let fs_path = path_buf.get_var().get_string()?;
        let file_obj = self.fs_obj.lock().open_file(fs_path, mode)?;
        Ok(service::fsp::File::new(file_obj))
    }

    fn open_directory(&mut self, mode: fsp::DirectoryOpenMode, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<service::fsp::Directory> {
        let fs_path = path_buf.get_var().get_string()?;
        let dir_obj = self.fs_obj.lock().open_directory(fs_path, mode)?;
        Ok(service::fsp::Directory::new(dir_obj))
    }

    fn commit(&mut self) -> Result<()> {
        self.fs_obj.lock().commit()
    }

    fn get_free_space_size(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<usize> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().get_free_space_size(fs_path)
    }

    fn get_total_space_size(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<usize> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().get_total_space_size(fs_path)
    }

    fn clean_directory_recursively(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().clean_directory_recursively(fs_path)
    }

    fn get_file_time_stamp_raw(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>) -> Result<fsp::FileTimeStampRaw> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().get_file_time_stamp_raw(fs_path)
    }

    fn query_entry(&mut self, path_buf: sf::InFixedPointerBuffer<fsp::Path>, query_id: fsp::QueryId, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<()> {
        let fs_path = path_buf.get_var().get_string()?;
        self.fs_obj.lock().query_entry(fs_path, query_id, in_buf.get_address(), in_buf.get_size(), out_buf.get_address(), out_buf.get_size())
    }
}

impl server::ISessionObject for FileSystem {}
 */