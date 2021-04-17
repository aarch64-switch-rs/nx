use crate::result::*;
use crate::ipc::cmif::sf;
use crate::util;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct FirmwareVersion {
    pub major: u8,
    pub minor: u8,
    pub micro: u8,
    pub pad_1: u8,
    pub revision_major: u8,
    pub revision_minor: u8,
    pub pad_2: u8,
    pub pad_3: u8,
    pub platform: util::CString<0x20>,
    pub version_hash: util::CString<0x40>,
    pub display_version: util::CString<0x18>,
    pub display_title: util::CString<0x80>
}

pub trait ISystemSettingsServer {
    ipc_cmif_interface_define_command!(get_firmware_version: (out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) => ());
}