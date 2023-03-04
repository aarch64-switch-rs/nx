//! FileSystem support

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
use crate::svc;
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

/// Represents a file
pub trait File {
    /// Reads data from the file, returning the actual read size
    /// 
    /// # Arguments
    /// 
    /// * `offset`: The absolute offset
    /// * `out_buf`: The out data address to fill
    /// * `out_buf_size`: The size of data to read
    /// * `option`: [`FileReadOption`] value
    fn read(&mut self, offset: usize, out_buf: *mut u8, out_buf_size: usize, option: FileReadOption) -> Result<usize>;

    /// Writes data to a file (this one doesn't return the actual written size, thanks N)
    /// 
    /// # Arguments
    /// 
    /// * `offset`: The absolute offset
    /// * `buf`: The data address to write from
    /// * `buf_size`: The size of data to write
    /// * `option`: [`FileWriteOption`] value
    fn write(&mut self, offset: usize, buf: *const u8, buf_size: usize, option: FileWriteOption) -> Result<()>;

    /// Flushes the file
    fn flush(&mut self) -> Result<()>;

    /// Sets the file size
    /// 
    /// This effectively truncates the file
    /// 
    /// # Arguments
    /// 
    /// * `size`: The new file size
    fn set_size(&mut self, size: usize) -> Result<()>;

    /// Gets the current file size
    fn get_size(&mut self) -> Result<usize>;

    /// Performs a range-operation on the file, returning corresponding result data
    /// 
    /// # Arguments
    /// 
    /// * `operation_id`: [`OperationId`] value
    /// * `offset`: The absolute offset
    /// * `size`: The file data size in which to operate
    fn operate_range(&mut self, operation_id: OperationId, offset: usize, size: usize) -> Result<FileQueryRangeInfo>;

    /// Performs a range-operation on the file with custom input/output data
    /// 
    /// # Arguments
    /// 
    /// * `operation_id`: [`OperationId`] value
    /// * `offset`: The absolute offset
    /// * `size`: The file data size in which to operate
    /// * `in_buf`: Input data
    /// * `in_buf_size`: Input data size
    /// * `out_buf`: Output data
    /// * `out_buf_size`: Output data size
    fn operate_range_with_buffer(&mut self, operation_id: OperationId, offset: usize, size: usize, in_buf: *const u8, in_buf_size: usize, out_buf: *mut u8, out_buf_size: usize) -> Result<()>;
}

/// Represents a directory
pub trait Directory {
    /// Reads existing entries, returning the actual number of read entries
    /// 
    /// The max number of entries to read is determined by the output array size and the actually existing entry count
    /// 
    /// # Arguments
    /// 
    /// * `out_entries`: The out [`DirectoryEntry`] array to fill
    fn read(&mut self, out_entries: &mut [DirectoryEntry]) -> Result<u64>;

    /// Gets the [`Directory`]'s entry count
    fn get_entry_count(&mut self) -> Result<u64>;
}

/// Represents a filesystem
pub trait FileSystem {
    /// Creates a file
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    /// * `size`: The initial file size
    /// * `attribute`: The file attribute
    fn create_file(&mut self, path: String, attribute: FileAttribute, size: usize) -> Result<()>;

    /// Deletes a file
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    fn delete_file(&mut self, path: String) -> Result<()>;

    /// Creates a directory
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    fn create_directory(&mut self, path: String) -> Result<()>;

    /// Deletes a directory
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    fn delete_directory(&mut self, path: String) -> Result<()>;

    /// Deletes a directory and all its children files/directories
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    fn delete_directory_recursively(&mut self, path: String) -> Result<()>;

    /// Renames a file
    /// 
    /// # Arguments
    /// 
    /// * `old_path`: The old path to use
    /// * `new_path`: The new path to use
    fn rename_file(&mut self, old_path: String, new_path: String) -> Result<()>;

    /// Renames a directory
    /// 
    /// # Arguments
    /// 
    /// * `old_path`: The old path to use
    /// * `new_path`: The new path to use
    fn rename_directory(&mut self, old_path: String, new_path: String) -> Result<()>;

    /// Gets a path's [`DirectoryEntryType`]
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    fn get_entry_type(&mut self, path: String) -> Result<DirectoryEntryType>;

    /// Opens a [`File`]
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    /// * `mode`: The open mode
    fn open_file(&mut self, path: String, mode: FileOpenMode) -> Result<mem::Shared<dyn File>>;

    /// Opens a [`Directory`]
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    /// * `mode`: The open mode
    fn open_directory(&mut self, path: String, mode: DirectoryOpenMode) -> Result<mem::Shared<dyn Directory>>;

    /// Commits the filesystem
    fn commit(&mut self) -> Result<()>;

    /// Gets the free space size at a given path
    /// 
    /// # Argument
    /// 
    /// * `path`: The path to use
    fn get_free_space_size(&mut self, path: String) -> Result<usize>;

    /// Gets the total space size at a given path
    /// 
    /// # Argument
    /// 
    /// * `path`: The path to use
    fn get_total_space_size(&mut self, path: String) -> Result<usize>;

    /// Deletes all the children files/directories inside a directory
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    fn clean_directory_recursively(&mut self, path: String) -> Result<()>;

    /// Gets the [`FileTimeStampRaw`] of a file
    /// 
    /// # Arguments
    /// 
    /// * `path`: The path to use
    fn get_file_time_stamp_raw(&mut self, path: String) -> Result<FileTimeStampRaw>;

    /// Queries on a path
    /// 
    /// # Arguments
    /// 
    /// * `query_id`: The [`QueryId`]
    /// * `in_buf`: Input data
    /// * `in_buf_size`: Input data size
    /// * `out_buf`: Output data
    /// * `out_buf_size`: Output data size
    fn query_entry(&mut self, path: String, query_id: QueryId, in_buf: *const u8, in_buf_size: usize, out_buf: *mut u8, out_buf_size: usize) -> Result<()>;
}

/// Represents a wrapper [`File`] implementation to translate IPC [`IFile`] objects to [`File`] objects
pub struct ProxyFile {
    file_obj: mem::Shared<dyn IFile>
}

impl ProxyFile {
    /// Creates a new [`ProxyFile`] from a [`IFile`] shared object
    /// 
    /// # Arguments
    /// 
    /// * `file_obj`: The IPC [`IFile`] object to wrap
    pub fn new(file_obj: mem::Shared<dyn IFile>) -> Self {
        Self {
            file_obj
        }
    }
}

impl File for ProxyFile {
    fn read(&mut self, offset: usize, out_buf: *mut u8, out_buf_size: usize, option: FileReadOption) -> Result<usize> {
        self.file_obj.get().read(option, offset, out_buf_size, ipc_sf::Buffer::from_mut_ptr(out_buf, out_buf_size))
    }

    fn write(&mut self, offset: usize, buf: *const u8, buf_size: usize, option: FileWriteOption) -> Result<()> {
        self.file_obj.get().write(option, offset, buf_size, ipc_sf::Buffer::from_ptr(buf, buf_size))
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

    fn operate_range(&mut self, operation_id: OperationId, offset: usize, size: usize) -> Result<FileQueryRangeInfo> {
        self.file_obj.get().operate_range(operation_id, offset, size)
    }

    fn operate_range_with_buffer(&mut self, operation_id: OperationId, offset: usize, size: usize, in_buf: *const u8, in_buf_size: usize, out_buf: *mut u8, out_buf_size: usize) -> Result<()> {
        self.file_obj.get().operate_range_with_buffer(operation_id, offset, size, ipc_sf::Buffer::from_ptr(in_buf, in_buf_size), ipc_sf::Buffer::from_mut_ptr(out_buf, out_buf_size))
    }
}

/// Represents a wrapper [`Directory`] implementation to translate IPC [`IDirectory`] objects to [`Directory`] objects
pub struct ProxyDirectory {
    dir_obj: mem::Shared<dyn IDirectory>
}

impl ProxyDirectory {
    /// Creates a new [`ProxyDirectory`] from a [`IDirectory`] shared object
    /// 
    /// # Arguments
    /// 
    /// * `dir_obj`: The IPC [`IDirectory`] object to wrap
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

/// Represents a wrapper [`FileSystem`] implementation to translate IPC [`IFileSystem`] objects to [`FileSystem`] objects
pub struct ProxyFileSystem {
    fs_obj: mem::Shared<dyn IFileSystem>
}

impl ProxyFileSystem {
    /// Creates a new [`ProxyFileSystem`] from a [`IFileSystem`] shared object
    /// 
    /// # Arguments
    /// 
    /// * `fs_obj`: The IPC [`IFileSystem`] object to wrap
    pub fn new(fs_obj: mem::Shared<dyn IFileSystem>) -> Self {
        Self {
            fs_obj
        }
    }
}

impl FileSystem for ProxyFileSystem {
    fn create_file(&mut self, path: String, attribute: FileAttribute, size: usize) -> Result<()> {
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

    fn get_free_space_size(&mut self, path: String) -> Result<usize> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().get_free_space_size(ipc_sf::Buffer::from_var(&sf_path))
    }

    fn get_total_space_size(&mut self, path: String) -> Result<usize> {
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

    fn query_entry(&mut self, path: String, query_id: QueryId, in_buf: *const u8, in_buf_size: usize, out_buf: *mut u8, out_buf_size: usize) -> Result<()> {
        let sf_path = fsp::Path::from_string(path);
        self.fs_obj.get().query_entry(ipc_sf::Buffer::from_var(&sf_path), query_id, ipc_sf::Buffer::from_ptr(in_buf, in_buf_size), ipc_sf::Buffer::from_mut_ptr(out_buf, out_buf_size))
    }
}

/// Represents an offset kind/relativeness
pub enum Whence {
    Start,
    Current,
    End
}

/// Represents a wrapper type to simplify file access
pub struct FileAccessor {
    file: mem::Shared<dyn File>,
    offset: usize
}

impl FileAccessor {
    /// Creates a new [`FileAccessor`] from a given [`File`] shared object
    /// 
    /// # Arguments
    /// 
    /// * `file`: The shared object
    pub fn new(file: mem::Shared<dyn File>) -> Self {
        Self {
            file,
            offset: 0
        }
    }

    /// Gets the underlying [`File`] shared object
    pub fn get_object(&self) -> mem::Shared<dyn File> {
        self.file.clone()
    }

    /// Gets the file size
    pub fn get_size(&mut self) -> Result<usize> {
        self.file.get().get_size()
    }

    /// Seeks in the file to a certain offset
    /// 
    /// # Arguments
    /// 
    /// * `offset`: The offset to seek to
    /// * `whence`: The offset relativeness
    pub fn seek(&mut self, offset: usize, whence: Whence) -> Result<()> {
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

    /// Reads data in the given buffer
    /// 
    /// 
    /// # Arguments
    /// 
    /// * `out_buf`: The output array address
    /// * `buf_count`: The output type count to read (not size in bytes but item count)
    pub fn read<T>(&mut self, out_buf: *mut T, buf_count: usize) -> Result<usize> {
        let read_size = self.file.get().read(self.offset, out_buf as *mut u8, buf_count * cmem::size_of::<T>(), FileReadOption::None())?;
        self.offset += read_size;
        Ok(read_size)
    }

    /// Reads data in the given array
    /// 
    /// # Arguments
    /// 
    /// * `out_arr`: The output array
    pub fn read_array<T>(&mut self, out_arr: &mut [T]) -> Result<usize> {
        self.read(out_arr.as_mut_ptr(), out_arr.len())
    }

    /// Reads a value
    pub fn read_val<T>(&mut self) -> Result<T> {
        let mut t = unsafe { cmem::zeroed::<T>() };
        self.read(&mut t, 1)?;
        Ok(t)
    }

    // TODO (writes): some sort of "flush" flag to not always flush after writing?

    /// Writes data from the given buffer
    /// 
    /// 
    /// # Arguments
    /// 
    /// * `buf`: The input array address
    /// * `buf_count`: The input type count to write (not size in bytes but item count)
    pub fn write<T>(&mut self, buf: *const T, buf_count: usize) -> Result<()> {
        let buf_size = buf_count * cmem::size_of::<T>();
        self.file.get().write(self.offset, buf as *const u8, buf_size, FileWriteOption::Flush())?;
        self.offset += buf_size;
        Ok(())
    }

    /// Writes data from the given array
    /// 
    /// # Arguments
    /// 
    /// * `arr`: The input array
    pub fn write_array<T>(&mut self, arr: &[T]) -> Result<()> {
        self.write(arr.as_ptr(), arr.len())
    }

    /// Writes a value
    /// 
    /// # Arguments
    /// 
    /// * `t`: The value to write
    pub fn write_val<T>(&mut self, t: &T) -> Result<()> {
        self.write(t as *const T, 1)
    }
}

/// Represents a wrapper type to simplify directory access
pub struct DirectoryAccessor {
    dir: mem::Shared<dyn Directory>
}

impl DirectoryAccessor {
    /// Creates a new [`DirectoryAccessor`] from a given [`Directory`] shared object
    /// 
    /// # Arguments
    /// 
    /// * `dir`: The shared object
    pub fn new(dir: mem::Shared<dyn Directory>) -> Self {
        Self { dir }
    }

    /// Gets the directory entry count
    pub fn get_entry_count(&mut self) -> Result<u64> {
        self.dir.get().get_entry_count()
    }

    /// Gets the underlying [`Directory`] shared object
    pub fn get_object(&self) -> mem::Shared<dyn Directory> {
        self.dir.clone()
    }

    /// Tries to read the next entry
    /// 
    /// Note that if the end is reached this will return `Ok(None)`, the result reflects other possible inner I/O errors
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

static mut G_FSPSRV_SESSION: sync::Locked<Option<mem::Shared<dyn IFileSystemProxy>>> = sync::Locked::new(false, None);

define_bit_enum! {
    /// Represents options for opening files
    FileOpenOption (u32) {
        None = 0,
        Create = bit!(0),
        Read = bit!(1),
        Write = bit!(2),
        Append = bit!(3)
    }
}

/// Initializes `fsp-srv` support with a given [`IFileSystemProxy`] shared object
/// 
/// # Arguments
/// 
/// * `session`: The shared object
pub fn initialize_fspsrv_session_with(session: mem::Shared<dyn IFileSystemProxy>) {
    unsafe {
        G_FSPSRV_SESSION.set(Some(session));
    }
}

/// Initializes `fsp-srv` support instantiating a [`FileSystemProxy`][`fsp::srv::FileSystemProxy`] shared object
#[inline]
pub fn initialize_fspsrv_session() -> Result<()> {
    initialize_fspsrv_session_with(service::new_service_object::<fsp::srv::FileSystemProxy>()?);
    Ok(())
}

/// Gets whether `fsp-srv` support was initialized
#[inline]
pub fn is_fspsrv_session_initialized() -> bool {
    unsafe {
        G_FSPSRV_SESSION.get().is_some()
    }
}

/// Finalizes `fsp-srv` support
#[inline]
pub fn finalize_fspsrv_session() {
    unsafe {
        G_FSPSRV_SESSION.set(None);
    }
}

/// Gets the global [`IFileSystemProxy`] shared object used for `fsp-srv` support
#[inline]
pub fn get_fspsrv_session() -> Result<&'static mem::Shared<dyn IFileSystemProxy>> {
    unsafe {
        G_FSPSRV_SESSION.get().as_ref().ok_or(super::rc::ResultNotInitialized::make())
    }
}

/// Mounts a [`FileSystem`]
/// 
/// Paths inside the filesystem will be accesible as `<name>:/<path>` with fns like [`open_file`], etc.
/// 
/// # Arguments
/// 
/// * `name`: The mount name
/// * `fs`: The [`FileSystem`] shared object
pub fn mount(name: &str, fs: mem::Shared<dyn FileSystem>) -> Result<()> {
    let root_name = PathSegment::from(format!("{}:", name), PathSegmentType::Root);
    unsafe {
        G_DEVICES.get().push(FileSystemDevice::from(root_name, fs));
    }

    Ok(())
}

//// Mounts an IPC [`IFileSystem`]
/// 
/// Essentially creates a [`ProxyFileSystem`] and [`mount`]s it
/// 
/// # Arguments
/// 
/// * `name`: The mount name
/// * `fs_obj`: The [`IFileSystem`] shared object
pub fn mount_fsp_filesystem(name: &str, fs_obj: mem::Shared<dyn IFileSystem>) -> Result<()> {
    let proxy_fs = mem::Shared::new(ProxyFileSystem::new(fs_obj));
    mount(name, proxy_fs)
}

/// Mounts the system's SD card using `fsp-srv` support
/// 
/// This will fail with [`ResultNotInitialized`][`super::rc::ResultNotInitialized`] if `fsp-srv` support isn't initialized
/// 
/// # Arguments
/// 
/// * `name`: The mount name
pub fn mount_sd_card(name: &str) -> Result<()> {
    let sd_fs_obj = get_fspsrv_session()?.get().open_sd_card_filesystem()?;
    mount_fsp_filesystem(name, sd_fs_obj)
}

/// Unmounts a mounted filesystem
/// 
/// Note that this does nothing if there is no mounted filesystem with the given name
/// 
/// # Arguments
/// 
/// * `name`: The mount name
pub fn unmount(name: &str) {
    let root_name = String::from(name);
    unsafe {
        G_DEVICES.get().retain(|dev| dev.root_name.name != root_name);
    }
}

/// Unmounts all filesystems
pub fn unmount_all() {
    unsafe {
        G_DEVICES.get().clear();
    }
}

/// Returns the [`FileSystem`] corresponding to a given path
/// 
/// If there is a filesystem mounted as `demo`, calling this with `"demo:/anything"` will return an instance to that mounted filesystem
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn get_path_filesystem(path: String) -> Result<mem::Shared<dyn FileSystem>> {
    let unpacked_path = unpack_path(path)?;
    let fs = find_device_by_name(unpacked_path.first().unwrap())?;
    
    Ok(fs)
}

/// Returns the [`FileSystem`] and the processed path corresponding to a given path
/// 
/// If there is a filesystem mounted as `demo`, calling this with `"demo:/anything"` will return an instance to that mounted filesystem and `"anything"` as the processed path
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn format_path(path: String) -> Result<(mem::Shared<dyn FileSystem>, String)> {
    let unpacked_path = unpack_path(path)?;
    let fs = find_device_by_name(unpacked_path.first().unwrap())?;
    let processed_path = pack_path(unpacked_path, false);
    
    Ok((fs, processed_path))
}

/// Creates a file
/// 
/// # Arguments
/// 
/// * `path`: The path to use
/// * `size`: The initial file size, default/IPC behaviour is to fill the file with zeros
/// * `attribute`: The file attribute, default/IPC behaviour uses this to allow creating "concatenation files" (allowing 32GB+ files in FAT32 filesystems)
pub fn create_file(path: String, size: usize, attribute: FileAttribute) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().create_file(processed_path, attribute, size)
}

/// Deletes a file
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn delete_file(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().delete_file(processed_path)
}

/// Creates a directory
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn create_directory(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().create_directory(processed_path)
}

/// Deletes a directory
/// 
/// Note that (in default/IPC behaviour) this won't succeed unless the directory is empty (see [`delete_directory_recursively`])
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn delete_directory(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().delete_directory(processed_path)
}

/// Deletes a directory and all its children files/directories
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn delete_directory_recursively(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().delete_directory_recursively(processed_path)
}

/// Deletes all the children files/directories inside a directory
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn clean_directory_recursively(path: String) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().clean_directory_recursively(processed_path)
}

/// Gets a path's [`DirectoryEntryType`]
/// 
/// This can be use to easily check if a file/directory exists, or whether they actually are a file or a directory
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn get_entry_type(path: String) -> Result<DirectoryEntryType> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_entry_type(processed_path)
}

/// Converts a [`FileOpenOption`] to a [`FileOpenMode`]
/// 
/// # Arguments
/// 
/// * `option`: Input option
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

/// Converts a [`FileOpenMode`] to a [`FileOpenOption`]
/// 
/// # Arguments
/// 
/// * `mode`: Input mode
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

/// Renames a file
/// 
/// # Arguments
/// 
/// * `old_path`: The old path to use
/// * `new_path`: The new path to use
pub fn rename_file(old_path: String, new_path: String) -> Result<()> {
    let (old_fs, processed_old_path) = format_path(old_path)?;
    let (new_fs, processed_new_path) = format_path(new_path)?;
    result_return_unless!(old_fs == new_fs, rc::ResultNotInSameFileSystem);

    old_fs.get().rename_file(processed_old_path, processed_new_path)
}

/// Renames a directory
/// 
/// # Arguments
/// 
/// * `old_path`: The old path to use
/// * `new_path`: The new path to use
pub fn rename_directory(old_path: String, new_path: String) -> Result<()> {
    let (old_fs, processed_old_path) = format_path(old_path)?;
    let (new_fs, processed_new_path) = format_path(new_path)?;
    result_return_unless!(old_fs == new_fs, rc::ResultNotInSameFileSystem);

    old_fs.get().rename_directory(processed_old_path, processed_new_path)
}

/// Renames a file/directory
/// 
/// Essentially is a wrapper for checking the entry type and calling [`rename_file`] or [`rename_directory`] according to that
/// 
/// Note that, to minimize overhead, this should only be used if the entry type isn't known beforehand
/// 
/// # Arguments
/// 
/// * `old_path`: The old path to use
/// * `new_path`: The new path to use
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

/// Opens a file as a [`FileAccessor`]
/// 
/// # Arguments
/// 
/// * `path`: The path to use
/// * `option`: The open option
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

    let offset : usize = match option.contains(FileOpenOption::Append()) {
        true => file.get().get_size().unwrap_or(0),
        false => 0
    };
    
    let mut file_acc = FileAccessor::new(file);
    file_acc.seek(offset, Whence::Start)?;
    Ok(file_acc)
}

/// Opens a directory as a [`DirectoryAccessor`]
/// 
/// # Arguments
/// 
/// * `path`: The path to use
/// * `mode`: The open mode
pub fn open_directory(path: String, mode: DirectoryOpenMode) -> Result<DirectoryAccessor> {
    let (fs, processed_path) = format_path(path)?;

    let dir = fs.get().open_directory(processed_path, mode)?;
    Ok(DirectoryAccessor::new(dir))
}

/// Commits on a filesystem
/// 
/// The only part of the path used is the filesystem mount name (to determine the filesystem to use)
/// 
/// # Argument
/// 
/// * `path`: The path to use
pub fn commit(path: String) -> Result<()> {
    let unpacked_path = unpack_path(path)?;
    let fs = find_device_by_name(unpacked_path.first().unwrap())?;
    fs.get().commit()
}

/// Gets the free space size at a given path
/// 
/// # Argument
/// 
/// * `path`: The path to use
pub fn get_free_space_size(path: String) -> Result<usize> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_free_space_size(processed_path)
}

/// Gets the total space size at a given path
/// 
/// # Argument
/// 
/// * `path`: The path to use
pub fn get_total_space_size(path: String) -> Result<usize> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_total_space_size(processed_path)
}

/// Gets the [`FileTimeStampRaw`] of a file
/// 
/// # Arguments
/// 
/// * `path`: The path to use
pub fn get_file_time_stamp_raw(path: String) -> Result<FileTimeStampRaw> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().get_file_time_stamp_raw(processed_path)
}

/// Queries on a path
/// 
/// # Arguments
/// 
/// * `query_id`: The [`QueryId`]
/// * `in_buf`: Input data
/// * `in_buf_size`: Input data size
/// * `out_buf`: Output data
/// * `out_buf_size`: Output data size
pub fn query_entry(path: String, query_id: QueryId, in_buf: *const u8, in_buf_size: usize, out_buf: *mut u8, out_buf_size: usize) -> Result<()> {
    let (fs, processed_path) = format_path(path)?;

    fs.get().query_entry(processed_path, query_id, in_buf, in_buf_size, out_buf, out_buf_size)
}

/// Sets the "concatenation file" attribute on a file
/// 
/// This essentially is a special case of [`query_entry`] to set an existing file as a "concatenation file"
/// 
/// # Arguments
/// 
/// * `path`: The path to use
#[inline]
pub fn set_concatenation_file_attribute(path: String) -> Result<()> {
    query_entry(path, QueryId::SetConcatenationFileAttribute, ptr::null(), 0, ptr::null_mut(), 0)
}

pub mod sf;

pub mod subdir;

pub mod pfs0;

pub mod nca;