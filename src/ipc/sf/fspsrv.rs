use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;

bit_enum! {
    FileOpenMode (u32) {
        None = 0,
        Read = bit!(0),
        Write = bit!(1),
        Append = bit!(2)
    }
}

bit_enum! {
    DirectoryOpenMode (u32) {
        ReadDirectories = bit!(0),
        ReadFiles = bit!(1),
        NoFileSizes = bit!(31)
    }
}

bit_enum! {
    FileAttribute (u32) {
        None = 0,
        ConcatenationFile = bit!(0)
    }
}

bit_enum! {
    FileReadOption (u32) {
        None = 0
    }
}

bit_enum! {
    FileWriteOption (u32) {
        None = 0,
        Flush = bit!(0)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum DirectoryEntryType {
    #[default]
    Directory = 0,
    File = 1
}

pub type Path = util::CString<0x301>;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DirectoryEntry {
    pub name: Path,
    pub attr: u8,
    pub pad: [u8; 2],
    pub entry_type: DirectoryEntryType,
    pub pad_2: [u8; 3],
    pub file_size: usize
}
const_assert!(core::mem::size_of::<DirectoryEntry>() == 0x310);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct FileTimeStampRaw {
    pub create: i64,
    pub modify: i64,
    pub access: i64,
    pub is_local_time: bool,
    pub pad: [u8; 7]
}
const_assert!(core::mem::size_of::<FileTimeStampRaw>() == 0x20);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum QueryId {
    SetConcatenationFileAttribute = 0,
    UpdateMac = 1,
    IsSignedSystemPartitionOnSdCardValid = 2,
    QueryUnpreparedFileInformation = 3
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct FileQueryRangeInfo {
    pub aes_ctr_key_type: u32,
    pub speed_emulation_type: u32,
    pub reserved_1: [u8; 0x20],
    pub reserved_2: [u8; 0x18]
}
const_assert!(core::mem::size_of::<FileQueryRangeInfo>() == 0x40);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum OperationId {
    FillZero = 0,
    DestroySignature = 1,
    Invalidate = 2,
    QueryRange = 3,
    QueryUnpreparedRange = 4,
    QueryLazyLoadCompletionRate = 5,
    SetLazyLoadPriority = 6,
    ReadLazyLoadFileForciblyForDebug = 10001
}

pub trait IFile {
    ipc_cmif_interface_define_command!(read: (option: FileReadOption, offset: usize, size: usize, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) => (read_size: usize));
    ipc_cmif_interface_define_command!(write: (option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer<u8>) => ());
    ipc_cmif_interface_define_command!(flush: () => ());
    ipc_cmif_interface_define_command!(set_size: (size: usize) => ());
    ipc_cmif_interface_define_command!(get_size: () => (size: usize));
    ipc_cmif_interface_define_command!(operate_range: (operation_id: OperationId, offset: usize, size: usize) => (info: FileQueryRangeInfo));
    ipc_cmif_interface_define_command!(operate_range_with_buffer: (operation_id: OperationId, offset: usize, size: usize, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) => ());
}

pub trait IDirectory {
    ipc_cmif_interface_define_command!(read: (out_entries: sf::OutMapAliasBuffer<DirectoryEntry>) => (read_count: u64));
    ipc_cmif_interface_define_command!(get_entry_count: () => (count: u64));
}

pub trait IFileSystem {
    ipc_cmif_interface_define_command!(create_file: (attribute: FileAttribute, size: usize, path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(delete_file: (path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(create_directory: (path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(delete_directory: (path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(delete_directory_recursively: (path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(rename_file: (old_path_buf: sf::InPointerBuffer<Path>, new_path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(rename_directory: (old_path_buf: sf::InPointerBuffer<Path>, new_path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(get_entry_type: (path_buf: sf::InPointerBuffer<Path>) => (entry_type: DirectoryEntryType));
    ipc_cmif_interface_define_command!(open_file: (mode: FileOpenMode, path_buf: sf::InPointerBuffer<Path>) => (file: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(open_directory: (mode: DirectoryOpenMode, path_buf: sf::InPointerBuffer<Path>) => (dir: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(commit: () => ());
    ipc_cmif_interface_define_command!(get_free_space_size: (path_buf: sf::InPointerBuffer<Path>) => (size: usize));
    ipc_cmif_interface_define_command!(get_total_space_size: (path_buf: sf::InPointerBuffer<Path>) => (size: usize));
    ipc_cmif_interface_define_command!(clean_directory_recursively: (path_buf: sf::InPointerBuffer<Path>) => ());
    ipc_cmif_interface_define_command!(get_file_time_stamp_raw: (path_buf: sf::InPointerBuffer<Path>) => (time_stamp: FileTimeStampRaw));
    ipc_cmif_interface_define_command!(query_entry: (path_buf: sf::InPointerBuffer<Path>, query_id: QueryId, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) => ());
}

pub trait IFileSystemProxy {
    ipc_cmif_interface_define_command!(set_current_process: (process_id: sf::ProcessId) => ());
    ipc_cmif_interface_define_command!(open_sd_card_filesystem: () => (sd_filesystem: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(output_access_log_to_sd_card: (log_buf: sf::InMapAliasBuffer<u8>) => ());
}