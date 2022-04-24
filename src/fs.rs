use crate::result::*;
use crate::mem;
use crate::service;
use crate::service::fsp;
use crate::service::fsp::IFileSystem;
use crate::service::fsp::IFile;
use crate::service::fsp::IDirectory;
use crate::service::fsp::srv::IFileSystemProxy;
use crate::sync;
use crate::ipc::sf as ipc_sf;
use alloc::vec::Vec;
use alloc::string::String;
use core::mem as cmem;
use core::ptr;

pub mod rc;
 
// TODO: define this types here and alias them in fsp-srv?

pub type FileReadOption = fsp::FileReadOption;
pub type FileWriteOption = fsp::FileWriteOption;
pub type DirectoryEntry = fsp::DirectoryEntry;
pub type FileAttribute = fsp::FileAttribute;
pub type DirectoryEntryType = fsp::DirectoryEntryType;
pub type FileOpenMode = fsp::FileOpenMode;
pub type DirectoryOpenMode = fsp::DirectoryOpenMode;
pub type FileTimeStampRaw = fsp::FileTimeStampRaw;
pub type QueryId = fsp::QueryId;
pub type OperationId = fsp::OperationId;
pub type FileQueryRangeInfo = fsp::FileQueryRangeInfo;

pub trait File {
    fn read(&mut self, offset: u64, out_buf: *mut u8, out_buf_size: u64, option: FileReadOption) -> Result<u64>;
    fn write(&mut self, offset: u64, buf: *const u8, buf_size: u64, option: FileWriteOption) -> Result<()>; // Write command does not return the written size
    fn flush(&mut self) -> Result<()>;
    fn set_size(&mut self, size: u64) -> Result<()>;
    fn get_size(&mut self) -> Result<u64>;
    fn operate_range(&mut self, operation_id: OperationId, offset: u64, size: u64) -> Result<FileQueryRangeInfo>;
    fn operate_range_with_buffer(&mut self, operation_id: OperationId, offset: u64, size: u64, in_buf: *const u8, in_buf_size: u64, out_buf: *mut u8, out_buf_size: u64) -> Result<()>;
}

pub trait Directory {
    fn read(&mut self, out_entries: &mut [DirectoryEntry]) -> Result<u64>;
    fn get_entry_count(&mut self) -> Result<u64>;
}

pub trait FileSystem {
    fn create_file(&mut self, path: String, attribute: FileAttribute, size: u64) -> Result<()>;
    fn delete_file(&mut self, path: String) -> Result<()>;
    fn create_directory(&mut self, path: String) -> Result<()>;
    fn delete_directory(&mut self, path: String) -> Result<()>;
    fn delete_directory_recursively(&mut self, path: String) -> Result<()>;
    fn rename_file(&mut self, old_path: String, new_path: String) -> Result<()>;
    fn rename_directory(&mut self, old_path: String, new_path: String) -> Result<()>;
    fn get_entry_type(&mut self, path: String) -> Result<DirectoryEntryType>;
    fn open_file(&mut self, path: String, mode: FileOpenMode) -> Result<mem::Shared<dyn File>>;
    fn open_directory(&mut self, path: String, mode: DirectoryOpenMode) -> Result<mem::Shared<dyn Directory>>;
    fn commit(&mut self) -> Result<()>;
    fn get_free_space_size(&mut self, path: String) -> Result<u64>;
    fn get_total_space_size(&mut self, path: String) -> Result<u64>;
    fn clean_directory_recursively(&mut self, path: String) -> Result<()>;
    fn get_file_time_stamp_raw(&mut self, path: String) -> Result<FileTimeStampRaw>;
    fn query_entry(&mut self, path: String, query_id: QueryId, in_buf: *const u8, in_buf_size: u64, out_buf: *mut u8, out_buf_size: u64) -> Result<()>;
}

// Proxy* objects are helper object types to translate from IPC fs objects to our fs objects

pub struct ProxyFile {
    file_obj: mem::Shared<dyn IFile>
}

impl ProxyFile {
    pub fn new(file_obj: mem::Shared<dyn IFile>) -> Self {
        Self {
            file_obj
        }
    }
}

impl File for ProxyFile {
    fn read(&mut self, offset: u64, out_buf: *mut u8, out_buf_size: u64, option: FileReadOption) -> Result<u64> {
        self.file_obj.get().read(option, offset, out_buf_size, ipc_sf::Buffer::from_mut_ptr(out_buf, out_buf_size))
    }

    fn write(&mut self, offset: u64, buf: *const u8, buf_size: u64, option: FileWriteOption) -> Result<()> {
        self.file_obj.get().write(option, offset, buf_size, ipc_sf::Buffer::from_ptr(buf, buf_size))
    }

    fn flush(&mut self) -> Result<()> {
        self.file_obj.get().flush()
    }

    fn set_size(&mut self, size: u64) -> Result<()> {
        self.file_obj.get().set_size(size)
    }

    fn get_size(&mut self) -> Result<u64> {
        self.file_obj.get().get_size()
    }

    fn operate_range(&mut self, operation_id: OperationId, offset: u64, size: u64) -> Result<FileQueryRangeInfo> {
        self.file_obj.get().operate_range(operation_id, offset, size)
    }

    fn operate_range_with_buffer(&mut self, operation_id: OperationId, offset: u64, size: u64, in_buf: *const u8, in_buf_size: u64, out_buf: *mut u8, out_buf_size: u64) -> Result<()> {
        self.file_obj.get().operate_range_with_buffer(operation_id, offset, size, ipc_sf::Buffer::from_ptr(in_buf, in_buf_size), ipc_sf::Buffer::from_mut_ptr(out_buf, out_buf_size))
    }
}

pub struct ProxyDirectory {
    dir_obj: mem::Shared<dyn IDirectory>
}

impl ProxyDirectory {
    pub fn new(dir_obj: mem::Shared<dyn IDirectory>) -> Self {
        Self {
            dir_obj
        }
    }
}

impl Directory for ProxyDirectory {
    fn read(&mut self, out_entries: &mut [DirectoryEntry]) -> Result<u64> {
        self.dir_obj.get().read(ipc_sf::Buffer::from_array(out_entries))
    }

    fn get_entry_count(&mut self) -> Result<u64> {
        self.dir_obj.get().get_entry_count()
    }
}

pub struct ProxyFileSystem {
    fs_obj: mem::Shared<dyn IFileSystem>
}

impl ProxyFileSystem {
    pub fn new(fs_obj: mem::Shared<dyn IFileSystem>) -> Self {
        Self {
            fs_obj
        }
    }
}

impl FileSystem for ProxyFileSystem {
    fn create_file(&mut self, path: String, attribute: FileAttribute, size: u64) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().create_file(attribute, size, ipc_sf::Buffer::from_var(&sf_path))
    }

    fn delete_file(&mut self, path: String) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().delete_file(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn create_directory(&mut self, path: String) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().create_directory(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn delete_directory(&mut self, path: String) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().delete_directory(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn delete_directory_recursively(&mut self, path: String) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().delete_directory_recursively(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn get_entry_type(&mut self, path: String) -> Result<DirectoryEntryType> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().get_entry_type(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn rename_file(&mut self, old_path: String, new_path: String) -> Result<()> {
        let sf_old_path = fsp::Path::from_string(old_path);
        let sf_new_path = fsp::Path::from_string(new_path);
        self.fs_obj.get().rename_file(ipc_sf::Buffer::from_var(&sf_old_path), ipc_sf::Buffer::from_var(&sf_new_path))
    }

    fn rename_directory(&mut self, old_path: String, new_path: String) -> Result<()> {
        let sf_old_path = fsp::Path::from_string(old_path);
        let sf_new_path = fsp::Path::from_string(new_path);
        self.fs_obj.get().rename_directory(ipc_sf::Buffer::from_var(&sf_old_path), ipc_sf::Buffer::from_var(&sf_new_path))
    }

    fn open_file(&mut self, path: String, mode: FileOpenMode) -> Result<mem::Shared<dyn File>> {
        let sf_path = fsp::Path::from_string(path);
        let file_obj = self.fs_obj.get().open_file(mode, ipc_sf::Buffer::from_var(&sf_path))?;
        Ok(mem::Shared::new(ProxyFile::new(file_obj)))
    }

    fn open_directory(&mut self, path: String, mode: DirectoryOpenMode) -> Result<mem::Shared<dyn Directory>> {
        let sf_path = fsp::Path::from_string(path);
        let dir_obj = self.fs_obj.get().open_directory(mode, ipc_sf::Buffer::from_var(&sf_path))?;
        Ok(mem::Shared::new(ProxyDirectory::new(dir_obj)))
    }

    fn commit(&mut self) -> Result<()> {
        self.fs_obj.get().commit()
    }

    fn get_free_space_size(&mut self, path: String) -> Result<u64> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().get_free_space_size(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn get_total_space_size(&mut self, path: String) -> Result<u64> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().get_total_space_size(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn clean_directory_recursively(&mut self, path: String) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().clean_directory_recursively(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn get_file_time_stamp_raw(&mut self, path: String) -> Result<FileTimeStampRaw> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().get_file_time_stamp_raw(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn query_entry(&mut self, path: String, query_id: QueryId, in_buf: *const u8, in_buf_size: u64, out_buf: *mut u8, out_buf_size: u64) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().query_entry(ipc_sf::Buffer::from_var(&sf_path), query_id, ipc_sf::Buffer::from_ptr(in_buf, in_buf_size), ipc_sf::Buffer::from_mut_ptr(out_buf, out_buf_size))
    }
}

pub enum Whence {
    Start,
    Current,
    End
}

pub struct FileAccessor {
    file: mem::Shared<dyn File>,
    offset: u64
}

impl FileAccessor {
    pub fn new(file: mem::Shared<dyn File>) -> Self {
        Self { file, offset: 0 }
    }

    pub fn get_object(&self) -> mem::Shared<dyn File> {
        self.file.clone()
    }

    pub fn get_size(&mut self) -> Result<u64> {
        self.file.get().get_size()
    }

    pub fn seek(&mut self, offset: u64, whence: Whence) -> Result<()> {
        match whence {
            Whence::Start => self.offset = offset,
            Whence::Current => self.offset += offset,
            Whence::End => {
                let size = self.get_size()?;
                self.offset = size + offset;
            }
        };
        Ok(())
    }

    pub fn read<T>(&mut self, buf: *mut T, buf_size: u64) -> Result<u64> {
        let read_size = self.file.get().read(self.offset, buf as *mut u8, buf_size, FileReadOption::None())?;
        self.offset += buf_size;
        Ok(read_size)
    }

    pub fn read_array<T>(&mut self, arr: &mut [T]) -> Result<u64> {
        self.read(arr.as_mut_ptr(), (arr.len() * cmem::size_of::<T>()) as u64)
    }

    pub fn read_val<T: Copy + Default>(&mut self) -> Result<T> {
        let mut t: T = Default::default();
        self.read(&mut t, cmem::size_of::<T>() as u64)?;
        Ok(t)
    }

    pub fn write<T>(&mut self, buf: *const T, buf_size: u64) -> Result<()> {
        self.file.get().write(self.offset, buf as *const u8, buf_size, FileWriteOption::Flush())?;
        self.offset += buf_size;
        Ok(())
    }

    pub fn write_array<T>(&mut self, arr: &[T]) -> Result<()> {
        self.write(arr.as_ptr(), (arr.len() * cmem::size_of::<T>()) as u64)
    }

    pub fn write_val<T: Copy>(&mut self, t: T) -> Result<()> {
        self.write(&t, (cmem::size_of::<T>()) as u64)
    }
}

pub struct DirectoryAccessor {
    dir: mem::Shared<dyn Directory>
}

impl DirectoryAccessor {
    pub fn new(dir: mem::Shared<dyn Directory>) -> Self {
        Self { dir }
    }

    pub fn get_entry_count(&mut self) -> Result<u64> {
        self.dir.get().get_entry_count()
    }

    pub fn get_object(&self) -> mem::Shared<dyn Directory> {
        self.dir.clone()
    }

    pub fn read_next(&mut self) -> Result<Option<DirectoryEntry>> {
        let mut entries: [DirectoryEntry; 1] = Default::default();
        let read_count = self.dir.get().read(&mut entries)?;
        if read_count == 1 {
            Ok(Some(entries[0]))
        }
        else {
            Ok(None)
        }
    }
}

// --- private stuff ^

enum PathSegmentType {
    Invalid,
    Root,
    Normal
}

struct PathSegment {
    name: String,
    segment_type: PathSegmentType
}

impl PathSegment {
    pub const fn from(name: String, segment_type: PathSegmentType) -> Self {
        Self { name, segment_type }
    }

    pub const fn new() -> Self {
        Self::from(String::new(), PathSegmentType::Invalid)
    }
}

type UnpackedPath = Vec<PathSegment>;

fn unpack_path_impl(path: String) -> UnpackedPath {
    let mut unpacked_path: UnpackedPath = UnpackedPath::new();

    for sub_path in path.split('/') {
        let mut cur_segment = PathSegment::new();
        if sub_path.ends_with(':') {
            cur_segment.segment_type = PathSegmentType::Root;
            cur_segment.name = String::from(sub_path);
            unpacked_path.push(cur_segment);
        }
        else if sub_path == ".." {
            unpacked_path.pop();
        }
        else {
            cur_segment.segment_type = PathSegmentType::Normal;
            cur_segment.name = String::from(sub_path);
            unpacked_path.push(cur_segment);
        }
    }

    unpacked_path
}

fn unpack_path(path: String) -> Result<UnpackedPath> {
    let unpacked_path = unpack_path_impl(path);
    result_return_if!(unpacked_path.is_empty(), 0xBAD);
    Ok(unpacked_path)
}

fn pack_path(unpacked_path: UnpackedPath, add_root: bool) -> String {
    let mut path = String::new();
    if !add_root {
        path.push('/');
    }
    
    for path_segment in unpacked_path {
        match path_segment.segment_type {
            PathSegmentType::Root => {
                if add_root {
                    path = format!("{}{}/", path, path_segment.name);
                }
            },
            PathSegmentType::Normal => path = format!("{}{}/", path, path_segment.name),
            _ => {}
        }
    }
    
    // Minimum path must be "/"
    if path.len() > 1 {
        path.pop();
    }

    path
}

struct FileSystemDevice {
    root_name: PathSegment,
    fs: mem::Shared<dyn FileSystem>
}

impl FileSystemDevice {
    pub fn from(root_name: PathSegment, fs: mem::Shared<dyn FileSystem>) -> Self {
        Self { root_name, fs }
    }
}

static mut G_FSPSRV_SESSION: sync::Locked<mem::Shared<dyn IFileSystemProxy>> = sync::Locked::new(false, mem::Shared::<fsp::srv::FileSystemProxy>::empty());
static mut G_DEVICES: sync::Locked<Vec<FileSystemDevice>> = sync::Locked::new(false, Vec::new());

fn find_device_by_name(name: &PathSegment) -> Result<mem::Shared<dyn FileSystem>> {
    unsafe {
        for device in G_DEVICES.get() {
            if device.root_name.name == name.name {
                return Ok(device.fs.clone());
            }
        }
        rc::ResultDeviceNotFound::make_err()
    }
}

#[inline]
fn get_fspsrv_session_ref() -> &'static mut mem::Shared<dyn IFileSystemProxy> {
    unsafe {
        G_FSPSRV_SESSION.get()
    }
}

bit_enum! {
    FileOpenOption (u32) {
        None = 0,
        Create = bit!(0),
        Read = bit!(1),
        Write = bit!(2),
        Append = bit!(3)
    }
}

pub fn initialize_fspsrv_session() -> Result<()> {
    unsafe {
        G_FSPSRV_SESSION.set(service::new_service_object::<fsp::srv::FileSystemProxy>()?);
    }

    Ok(())
}

pub fn is_fspsrv_session_initialized() -> bool {
    !get_fspsrv_session_ref().is_null()
}

pub fn finalize_fspsrv_session() {
    get_fspsrv_session_ref().reset();
}

pub fn get_fspsrv_session() -> mem::Shared<dyn IFileSystemProxy> {
    get_fspsrv_session_ref().clone()
}

pub fn mount(name: &str, fs: mem::Shared<dyn FileSystem>) -> Result<()> {
    let root_name = PathSegment::from(format!("{}:", name), PathSegmentType::Root);
    unsafe {
        G_DEVICES.get().push(FileSystemDevice::from(root_name, fs));
    }

    Ok(())
}

pub fn mount_fsp_filesystem(name: &str, fs_obj: mem::Shared<dyn IFileSystem>) -> Result<()> {
    let proxy_fs = mem::Shared::new(ProxyFileSystem::new(fs_obj));
    mount(name, proxy_fs)
}

pub fn mount_sd_card(name: &str) -> Result<()> {
    result_return_unless!(is_fspsrv_session_initialized(), super::rc::ResultNotInitialized);
    
    let sd_fs_obj = get_fspsrv_session_ref().get().open_sd_card_filesystem()?;
    mount_fsp_filesystem(name, sd_fs_obj)
}

pub fn unmount(name: &str) {
    let root_name = String::from(name);
    unsafe {
        G_DEVICES.get().retain(|dev| dev.root_name.name != root_name);
    }
}

pub fn unmount_all() {
    unsafe {
        G_DEVICES.get().clear();
    }
}

pub fn get_path_filesystem(path: String) -> Result<mem::Shared<dyn FileSystem>> {
    let unpacked_path = unpack_path(path)?;
    let fs = find_device_by_name(unpacked_path.first().unwrap())?;
    
    Ok(fs)
}

pub fn format_path(path: String) -> Result<(mem::Shared<dyn FileSystem>, String)> {
    let unpacked_path = unpack_path(path)?;
    let fs = find_device_by_name(unpacked_path.first().unwrap())?;
    let processed_path = pack_path(unpacked_path, false);
    
    Ok((fs, processed_path))
}

pub fn create_file(path: String, size: u64, attribute: FileAttribute) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().create_file(processed_path, attribute, size)
}

pub fn delete_file(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().delete_file(processed_path)
}

pub fn create_directory(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().create_directory(processed_path)
}

pub fn delete_directory(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().delete_directory(processed_path)
}

pub fn delete_directory_recursively(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().delete_directory_recursively(processed_path)
}

pub fn clean_directory_recursively(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().clean_directory_recursively(processed_path)
}

pub fn get_entry_type(path: String) -> Result<DirectoryEntryType> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_entry_type(processed_path)
}

pub fn convert_file_open_option_to_mode(option: FileOpenOption) -> FileOpenMode {
    let mut mode = FileOpenMode::None();
    if option.contains(FileOpenOption::Read()) {
        mode |= FileOpenMode::Read();
    }
    if option.contains(FileOpenOption::Write()) {
        mode |= FileOpenMode::Write();
    }
    if option.contains(FileOpenOption::Append()) {
        mode |= FileOpenMode::Append();
    }
    mode
}

pub fn convert_file_open_mode_to_option(mode: FileOpenMode) -> FileOpenOption {
    let mut option = FileOpenOption::None();
    if mode.contains(FileOpenMode::Read()) {
        option |= FileOpenOption::Read();
    }
    if mode.contains(FileOpenMode::Write()) {
        option |= FileOpenOption::Write();
    }
    if mode.contains(FileOpenMode::Append()) {
        option |= FileOpenOption::Append();
    }
    option
}

pub fn rename_file(old_path: String, new_path: String) -> Result<()> {
    let (old_fs, processed_old_path) = format_path(old_path)?;
    let (new_fs, processed_new_path) = format_path(new_path)?;
    result_return_unless!(old_fs == new_fs, rc::ResultNotInSameFileSystem);

    old_fs.get().rename_file(processed_old_path, processed_new_path)
}

pub fn rename_directory(old_path: String, new_path: String) -> Result<()> {
    let (old_fs, processed_old_path) = format_path(old_path)?;
    let (new_fs, processed_new_path) = format_path(new_path)?;
    result_return_unless!(old_fs == new_fs, rc::ResultNotInSameFileSystem);

    old_fs.get().rename_directory(processed_old_path, processed_new_path)
}

pub fn rename(old_path: String, new_path: String) -> Result<()> {
    let (old_fs, processed_old_path) = format_path(old_path)?;
    let (new_fs, processed_new_path) = format_path(new_path)?;
    result_return_unless!(old_fs == new_fs, rc::ResultNotInSameFileSystem);

    let entry_type = old_fs.get().get_entry_type(processed_old_path.clone())?;
    match entry_type {
        DirectoryEntryType::Directory => old_fs.get().rename_directory(processed_old_path, processed_new_path),
        DirectoryEntryType::File => old_fs.get().rename_file(processed_old_path, processed_new_path)
    }
}

pub fn open_file(path: String, option: FileOpenOption) -> Result<FileAccessor> {
    let (fs, processed_path) = format_path(path)?;

    let mode = convert_file_open_option_to_mode(option);
    let file = match fs.get().open_file(processed_path.clone(), mode) {
        Ok(file) => file,
        Err(rc) => {
            if fsp::rc::ResultPathNotFound::matches(rc) && option.contains(FileOpenOption::Create()) {
                // Create the file if it doesn't exist and we were told to do so
                fs.get().create_file(processed_path.clone(), FileAttribute::None(), 0)?;
                fs.get().open_file(processed_path, mode)?
            }
            else {
                return Err(rc);
            }
        }
    };

    let offset : u64 = match option.contains(FileOpenOption::Append()) {
        true => file.get().get_size().unwrap_or(0),
        false => 0
    };
    
    let mut file_acc = FileAccessor::new(file);
    file_acc.seek(offset, Whence::Start)?;
    Ok(file_acc)
}

pub fn open_directory(path: String, mode: DirectoryOpenMode) -> Result<DirectoryAccessor> {
    let (fs, processed_path) = format_path(path)?;

    let dir = fs.get().open_directory(processed_path, mode)?;
    Ok(DirectoryAccessor::new(dir))
}

pub fn commit(path: String) -> Result<()> {
    let unpacked_path = unpack_path(path)?;
    let fs = find_device_by_name(unpacked_path.first().unwrap())?;
    fs.get().commit()
}

pub fn get_free_space_size(path: String) -> Result<u64> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_free_space_size(processed_path)
}

pub fn get_total_space_size(path: String) -> Result<u64> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_total_space_size(processed_path)
}

pub fn get_file_time_stamp_raw(path: String) -> Result<FileTimeStampRaw> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_file_time_stamp_raw(processed_path)
}

pub fn query_entry(path: String, query_id: QueryId, in_buf: *const u8, in_buf_size: u64, out_buf: *mut u8, out_buf_size: u64) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().query_entry(processed_path, query_id, in_buf, in_buf_size, out_buf, out_buf_size)
}

#[inline]
pub fn set_concatenation_file_attribute(path: String) -> Result<()> {
    query_entry(path, QueryId::SetConcatenationFileAttribute, ptr::null(), 0, ptr::null_mut(), 0)
}

pub mod sf;

pub mod subdir;

pub mod pfs0;

pub mod nca;