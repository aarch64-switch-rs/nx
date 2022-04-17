use crate::ipc::server;
use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use alloc::string::String;
use crate::ipc::sf::fsp;
use crate::ipc::sf::fsp::IFileSystem;
use crate::fs::sf as fs_sf;

pub struct FileSystem {
    sub_dir: String
}

impl FileSystem {
    pub fn new(sub_dir: String) -> Self {
        Self { sub_dir }
    }

    fn make_path(&self, path_buf: &sf::InPointerBuffer<fsp::Path>) -> Result<String> {
        let fs_path = path_buf.get_var().get_string()?;
        Ok(format!("{}/{}", self.sub_dir, fs_path))
    }
}

impl sf::IObject for FileSystem {
    ipc_sf_object_impl_default_command_metadata!();
}

impl IFileSystem for FileSystem {
    fn create_file(&mut self, attribute: fsp::FileAttribute, size: usize, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let path = self.make_path(&path_buf)?;
        super::create_file(path, size, attribute)
    }

    fn delete_file(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let path = self.make_path(&path_buf)?;
        super::delete_file(path)
    }

    fn create_directory(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let path = self.make_path(&path_buf)?;
        super::create_directory(path)
    }
    
    fn delete_directory(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let path = self.make_path(&path_buf)?;
        super::delete_directory(path)
    }

    fn delete_directory_recursively(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let path = self.make_path(&path_buf)?;
        super::delete_directory_recursively(path)
    }

    fn rename_file(&mut self, old_path_buf: sf::InPointerBuffer<fsp::Path>, new_path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let old_path = self.make_path(&old_path_buf)?;
        let new_path = self.make_path(&new_path_buf)?;
        super::rename_file(old_path, new_path)
    }

    fn rename_directory(&mut self, old_path_buf: sf::InPointerBuffer<fsp::Path>, new_path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let old_path = self.make_path(&old_path_buf)?;
        let new_path = self.make_path(&new_path_buf)?;
        super::rename_directory(old_path, new_path)
    }

    fn get_entry_type(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<fsp::DirectoryEntryType> {
        let path = self.make_path(&path_buf)?;
        super::get_entry_type(path)
    }
    
    fn open_file(&mut self, mode: fsp::FileOpenMode, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<mem::Shared<dyn fsp::IFile>> {
        let path = self.make_path(&path_buf)?;
        let file_accessor = super::open_file(path, super::convert_file_open_mode_to_option(mode))?;
        Ok(mem::Shared::new(fs_sf::File::new(file_accessor.get_object())))
    }

    fn open_directory(&mut self, mode: fsp::DirectoryOpenMode, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<mem::Shared<dyn fsp::IDirectory>> {
        let path = self.make_path(&path_buf)?;
        let dir_accessor = super::open_directory(path, mode)?;
        Ok(mem::Shared::new(fs_sf::Directory::new(dir_accessor.get_object())))
    }

    fn commit(&mut self) -> Result<()> {
        super::commit(self.sub_dir.clone())
    }

    fn get_free_space_size(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<usize> {
        let path = self.make_path(&path_buf)?;
        super::get_free_space_size(path)
    }

    fn get_total_space_size(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<usize> {
        let path = self.make_path(&path_buf)?;
        super::get_total_space_size(path)
    }

    fn clean_directory_recursively(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<()> {
        let path = self.make_path(&path_buf)?;
        super::clean_directory_recursively(path)
    }

    fn get_file_time_stamp_raw(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>) -> Result<fsp::FileTimeStampRaw> {
        let path = self.make_path(&path_buf)?;
        super::get_file_time_stamp_raw(path)
    }

    fn query_entry(&mut self, path_buf: sf::InPointerBuffer<fsp::Path>, query_id: fsp::QueryId, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) -> Result<()> {
        let path = self.make_path(&path_buf)?;
        super::query_entry(path, query_id, in_buf.get_address(), in_buf.get_size(), out_buf.get_address(), out_buf.get_size())
    }
}

impl server::ISessionObject for FileSystem {}