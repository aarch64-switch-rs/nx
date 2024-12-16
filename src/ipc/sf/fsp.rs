use crate::result::*;
use crate::ipc::sf;
use crate::util;
use crate::version;

pub mod rc;

use nx_derive::{Request, Response};

define_bit_enum! {
    FileOpenMode (u32) {
        None = 0,
        Read = bit!(0),
        Write = bit!(1),
        Append = bit!(2)
    }
}

define_bit_enum! {
    DirectoryOpenMode (u32) {
        ReadDirectories = bit!(0),
        ReadFiles = bit!(1),
        NoFileSizes = bit!(31)
    }
}

define_bit_enum! {
    FileAttribute (u32) {
        None = 0,
        ConcatenationFile = bit!(0)
    }
}

define_bit_enum! {
    FileReadOption (u32) {
        None = 0
    }
}

define_bit_enum! {
    FileWriteOption (u32) {
        None = 0,
        Flush = bit!(0)
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum DirectoryEntryType {
    #[default]
    Directory = 0,
    File = 1
}

pub type Path = util::ArrayString<0x301>;

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
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

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct FileTimeStampRaw {
    pub create: i64,
    pub modify: i64,
    pub access: i64,
    pub is_local_time: bool,
    pub pad: [u8; 7]
}
const_assert!(core::mem::size_of::<FileTimeStampRaw>() == 0x20);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum QueryId {
    SetConcatenationFileAttribute = 0,
    UpdateMac = 1,
    IsSignedSystemPartitionOnSdCardValid = 2,
    QueryUnpreparedFileInformation = 3
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct FileQueryRangeInfo {
    pub aes_ctr_key_type: u32,
    pub speed_emulation_type: u32,
    pub reserved_1: [u8; 0x20],
    pub reserved_2: [u8; 0x18]
}
const_assert!(core::mem::size_of::<FileQueryRangeInfo>() == 0x40);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
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

ipc_sf_define_default_interface_client!(File);
ipc_sf_define_interface_trait! {
	trait File {
        read [0, version::VersionInterval::all(), mut]: (option: FileReadOption, offset: usize, size: usize, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) =>  (read_size: usize) (read_size: usize);
        write [1, version::VersionInterval::all(), mut]: (option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer<u8>) =>  () ();
        flush [2, version::VersionInterval::all(), mut]: () => () ();
        set_size [3, version::VersionInterval::all(), mut]: (size: usize) =>  () ();
        get_size [4, version::VersionInterval::all(), mut]: () => (size: usize) (size: usize);
        operate_range [5, version::VersionInterval::from(version::Version::new(4,0,0)), mut]: (operation_id: OperationId, offset: usize, size: usize) =>  (info: FileQueryRangeInfo) (info: FileQueryRangeInfo);
        operate_range_with_buffer [6, version::VersionInterval::from(version::Version::new(12,0,0)), mut]: (operation_id: OperationId, offset: usize, size: usize, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) =>  () ();
    }
}

ipc_sf_define_default_interface_client!(Directory);
ipc_sf_define_interface_trait! {
	trait Directory {
        read [0, version::VersionInterval::all()]: (out_entries: sf::OutMapAliasBuffer<DirectoryEntry>) =>  (read_count: u64) (read_count: u64);
        get_entry_count [1, version::VersionInterval::all()]: () => (count: u64) (count: u64);
    }
}

ipc_sf_define_default_interface_client!(FileSystem);
ipc_sf_define_interface_trait! {
	trait FileSystem {
        create_file [0, version::VersionInterval::all()]: (attribute: FileAttribute, size: usize, path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        delete_file [1, version::VersionInterval::all()]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        create_directory [2, version::VersionInterval::all()]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        delete_directory [3, version::VersionInterval::all()]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        delete_directory_recursively [4, version::VersionInterval::all()]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        rename_file [5, version::VersionInterval::all()]: (old_path_buf: sf::InFixedPointerBuffer<Path>, new_path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        rename_directory [6, version::VersionInterval::all()]: (old_path_buf: sf::InFixedPointerBuffer<Path>, new_path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        get_entry_type [7, version::VersionInterval::all()]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  (entry_type: DirectoryEntryType) (entry_type: DirectoryEntryType);
        open_file [8, version::VersionInterval::all()]: (mode: FileOpenMode, path_buf: sf::InFixedPointerBuffer<Path>) =>  (file: File) (file: session_type!(File));
        open_directory [9, version::VersionInterval::all()]: (mode: DirectoryOpenMode, path_buf: sf::InFixedPointerBuffer<Path>) =>  (dir: Directory) (dir: session_type!(Directory));
        commit [10, version::VersionInterval::all()]: () => () ();
        get_free_space_size [11, version::VersionInterval::all()]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  (size: usize) (size: usize);
        get_total_space_size [12, version::VersionInterval::all()]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  (size: usize) (size: usize);
        clean_directory_recursively [13, version::VersionInterval::from(version::Version::new(3,0,0))]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  () ();
        get_file_time_stamp_raw [14, version::VersionInterval::from(version::Version::new(3,0,0))]: (path_buf: sf::InFixedPointerBuffer<Path>) =>  (time_stamp: FileTimeStampRaw) (time_stamp: FileTimeStampRaw);
        query_entry [15, version::VersionInterval::from(version::Version::new(4,0,0))]: (path_buf: sf::InFixedPointerBuffer<Path>, query_id: QueryId, in_buf: sf::InNonSecureMapAliasBuffer<u8>, out_buf: sf::OutNonSecureMapAliasBuffer<u8>) =>  () ();
    }
}

ipc_sf_define_default_interface_client!(FileSystemProxy);
ipc_sf_define_interface_trait! {
	trait FileSystemProxy {
        set_current_process [1, version::VersionInterval::all()]: (process_id: sf::ProcessId) =>  () ();
        open_sd_card_filesystem [18, version::VersionInterval::all()]: () => (sd_filesystem: FileSystem) (sd_filesystem: session_type!(FileSystem));
        output_access_log_to_sd_card [1006, version::VersionInterval::all()]: (log_buf: sf::InMapAliasBuffer<u8>) =>  () ();
    }
}

pub mod srv;