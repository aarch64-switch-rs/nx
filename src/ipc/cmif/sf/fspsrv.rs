use crate::result::*;
use crate::ipc::cmif::sf;
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

pub trait IFile {
    nipc_cmif_interface_define_command!(read: (option: FileReadOption, offset: usize, size: usize, buf: sf::OutNonSecureMapAliasBuffer) => (read_size: usize));
    nipc_cmif_interface_define_command!(write: (option: FileWriteOption, offset: usize, size: usize, buf: sf::InNonSecureMapAliasBuffer) => ());
    nipc_cmif_interface_define_command!(get_size: () => (size: usize));
}

pub trait IFileSystem {
    nipc_cmif_interface_define_command!(create_file: (attribute: FileAttribute, size: usize, path_buf: sf::InPointerBuffer) => ());
    nipc_cmif_interface_define_command!(delete_file: (path_buf: sf::InPointerBuffer) => ());
    nipc_cmif_interface_define_command!(create_directory: (path_buf: sf::InPointerBuffer) => ());
    nipc_cmif_interface_define_command!(delete_directory: (path_buf: sf::InPointerBuffer) => ());
    nipc_cmif_interface_define_command!(delete_directory_recursively: (path_buf: sf::InPointerBuffer) => ());
    nipc_cmif_interface_define_command!(get_entry_type: (path_buf: sf::InPointerBuffer) => (entry_type: DirectoryEntryType));
    nipc_cmif_interface_define_command!(open_file: (mode: FileOpenMode, path_buf: sf::InPointerBuffer) => (file: mem::Shared<dyn sf::IObject>));
}

pub trait IFileSystemProxy {
    nipc_cmif_interface_define_command!(set_current_process: (process_id: sf::ProcessId) => ());
    nipc_cmif_interface_define_command!(open_sd_card_filesystem: () => (sd_filesystem: mem::Shared<dyn sf::IObject>));
    nipc_cmif_interface_define_command!(output_access_log_to_sd_card: (access_log: sf::InMapAliasBuffer) => ());
}