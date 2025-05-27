use crate::ipc::sf;
use crate::ipc::sf::mii;
use crate::util;
use crate::version;

use nx_derive::{Request, Response};

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
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
    pub platform: util::ArrayString<0x20>,
    pub version_hash: util::ArrayString<0x40>,
    pub display_version: util::ArrayString<0x18>,
    pub display_title: util::ArrayString<0x80>,
}
const_assert!(core::mem::size_of::<FirmwareVersion>() == 0x100);

ipc_sf_define_interface_trait! {
    trait SystemSettings {
        get_firmware_version [3, version::VersionInterval::all()]: (out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) => ();
        get_firmware_version_2 [4, version::VersionInterval::from(version::Version::new(3,0,0))]: (out_version: sf::OutFixedPointerBuffer<FirmwareVersion>) => ();
        get_mii_author_id [90, version::VersionInterval::all()]: () => (id: mii::CreateId);
    }
}
