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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum DirectoryEntryType {
    Directory = 0,
    File = 1
}

pub type Path = util::CString<0x301>;

#[derive(Copy, Clone)]
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

pub trait IFile {
    ipc_cmif_interface_define_command!(read: (option: FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer) => (read_size: usize));
    ipc_cmif_interface_define_command!(write: (option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer) => ());
    ipc_cmif_interface_define_command!(get_size: () => (size: usize));
}

pub trait IDirectory {
    ipc_cmif_interface_define_command!(read: (out_entries: sf::OutMapAliasBuffer) => (read_count: u64));
    ipc_cmif_interface_define_command!(get_entry_count: () => (count: u64));
}

pub trait IFileSystem {
    ipc_cmif_interface_define_command!(create_file: (attribute: FileAttribute, size: usize, path_buf: sf::InPointerBuffer) => ());
    ipc_cmif_interface_define_command!(delete_file: (path_buf: sf::InPointerBuffer) => ());
    ipc_cmif_interface_define_command!(create_directory: (path_buf: sf::InPointerBuffer) => ());
    ipc_cmif_interface_define_command!(delete_directory: (path_buf: sf::InPointerBuffer) => ());
    ipc_cmif_interface_define_command!(delete_directory_recursively: (path_buf: sf::InPointerBuffer) => ());
    ipc_cmif_interface_define_command!(get_entry_type: (path_buf: sf::InPointerBuffer) => (entry_type: DirectoryEntryType));
    ipc_cmif_interface_define_command!(open_file: (mode: FileOpenMode, path_buf: sf::InPointerBuffer) => (file: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(open_directory: (mode: DirectoryOpenMode, path_buf: sf::InPointerBuffer) => (dir: mem::Shared<dyn sf::IObject>));
}

pub trait IFileSystemProxy {
    ipc_cmif_interface_define_command!(set_current_process: (process_id: sf::ProcessId) => ());
    ipc_cmif_interface_define_command!(open_sd_card_filesystem: () => (sd_filesystem: mem::Shared<dyn sf::IObject>));
    ipc_cmif_interface_define_command!(output_access_log_to_sd_card: (access_log: sf::InMapAliasBuffer) => ());
}