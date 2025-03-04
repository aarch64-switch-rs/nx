use crate::ipc::sf;
use crate::result::*;
use crate::util::ArrayString;
use crate::version;
use core::fmt::{Debug, Display, Formatter, Result as FmtResult};

use nx_derive::{Request, Response};

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(C)]
pub struct ProgramId(pub u64);

impl Display for ProgramId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:#018X}", self.0)
    }
}

impl Debug for ProgramId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:#018X}", self.0)
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(C)]
pub struct ApplicationId(pub u64);

impl Display for ApplicationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:#018X}", self.0)
    }
}

impl Debug for ApplicationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{:#018X}", self.0)
    }
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum StorageId {
    #[default]
    None = 0,
    Host = 1,
    GameCard = 2,
    BuiltInSystem = 3,
    BuiltInUser = 4,
    SdCard = 5,
    Any = 6,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum ContentMetaType {
    #[default]
    Unknown = 0x0,
    SystemProgram = 0x1,
    SystemData = 0x2,
    SystemUpdate = 0x3,
    BootImagePackage = 0x4,
    BootImagePackageSafe = 0x5,
    Application = 0x80,
    Patch = 0x81,
    AddOnContent = 0x82,
    Delta = 0x83,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum ContentType {
    #[default]
    Meta = 0,
    Program = 1,
    Data = 2,
    Control = 3,
    HtmlDocument = 4,
    LegalInformation = 5,
    DeltaFragment = 6,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum ContentInstallType {
    #[default]
    Full = 0x0,
    FragmentOnly = 0x1,
    Unknown = 0x7,
}

pub type ContentPath = ArrayString<0x301>;

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ContentMetaKey {
    pub program_id: ProgramId,
    pub version: u32,
    pub meta_type: ContentMetaType,
    pub install_type: ContentInstallType,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ApplicationContentMetaKey {
    pub key: ContentMetaKey,
    pub app_id: ApplicationId,
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ContentId {
    pub id: [u8; 0x10],
}

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct LegacyContentInfo {
    pub hash: [u8; 0x20],
    pub id: ContentId,
    pub size: [u8; 0x6],
    pub cnt_type: ContentType,
    pub id_offset: u8,
}
const_assert!(core::mem::size_of::<LegacyContentInfo>() == 0x38);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ContentInfo {
    pub hash: [u8; 0x20],
    pub id: ContentId,
    pub size: [u8; 0x5],
    pub attrs: u8,
    pub cnt_type: ContentType,
    pub id_offset: u8,
}
const_assert!(core::mem::size_of::<ContentInfo>() == 0x38);

#[derive(Request, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ContentMetaInfo {
    pub program_id: ProgramId,
    pub version: u32,
    pub meta_type: ContentMetaType,
    pub attrs: u8,
    pub pad: [u8; 2],
}

ipc_sf_define_default_interface_client!(ContentMetaDatabase);
ipc_sf_define_interface_trait! {
    trait ContentMetaDatabase {
        set [0, version::VersionInterval::all()]: (meta_key: ContentMetaKey, in_rec_buf: sf::InMapAliasBuffer<u8>) =>  () ();
        get [1, version::VersionInterval::all()]: (meta_key: ContentMetaKey, out_rec_buf: sf::OutMapAliasBuffer<u8>) =>  (size: usize) (size: usize);
        remove [2, version::VersionInterval::all()]: (meta_key: ContentMetaKey) =>  () ();
        get_content_id_by_type [3, version::VersionInterval::all()]: (meta_key: ContentMetaKey, cnt_type: ContentType) =>  (id: ContentId) (id: ContentId);
        list_content_info [4, version::VersionInterval::all()]: (out_rec_buf: sf::OutMapAliasBuffer<ContentInfo>, meta_key: ContentMetaKey, offset: u32) =>  (count: u32) (count: u32);
        list [5, version::VersionInterval::all()]: (out_meta_keys: sf::OutMapAliasBuffer<ContentMetaKey>, meta_type: ContentMetaType, program_id: ProgramId, program_id_min: ProgramId, program_id_max: ProgramId, install_type: ContentInstallType) =>  (total: u32, written: u32) (total: u32, written: u32);
        get_latest_content_meta_key [6, version::VersionInterval::all()]: (program_id: ProgramId) =>  (meta_key: ContentMetaKey) (meta_key: ContentMetaKey);
        list_application [7, version::VersionInterval::all()]: (out_app_meta_keys: sf::OutMapAliasBuffer<ApplicationContentMetaKey>, meta_type: ContentMetaType) =>  (total: u32, written: u32) (total: u32, written: u32);
        has [8, version::VersionInterval::all()]: (meta_key: ContentMetaKey) =>  (has: bool) (has: bool);
        has_all [9, version::VersionInterval::all()]: (meta_keys_buf: sf::InMapAliasBuffer<ContentMetaKey>) =>  (has: bool) (has: bool);
        get_size [10, version::VersionInterval::all()]: (meta_key: ContentMetaKey) =>  (size: usize) (size: usize);
        get_required_system_version [11, version::VersionInterval::all()]: (meta_key: ContentMetaKey) =>  (version: u32) (version: u32);
        get_patch_content_meta_id [12, version::VersionInterval::all()]: (meta_key: ContentMetaKey) =>  (patch_id: ProgramId) (patch_id: ProgramId);
        disable_forcibly [13, version::VersionInterval::all()]: () => () ();
        lookup_orphan_content [14, version::VersionInterval::all()]: (content_ids_buf: sf::InMapAliasBuffer<ContentId>, out_orphaned_buf: sf::OutMapAliasBuffer<bool>) =>  () ();
        commit [15, version::VersionInterval::all()]: () => () ();
        has_content [16, version::VersionInterval::all()]: (meta_key: ContentMetaKey, id: ContentId) =>  (has: bool) (has: bool);
        list_content_meta_info [17, version::VersionInterval::all()]: (out_meta_infos: sf::OutMapAliasBuffer<ContentMetaInfo>, meta_key: ContentMetaKey, offset: u32) =>  (written: u32) (written: u32);
        get_attributes [18, version::VersionInterval::all()]: (meta_key: ContentMetaKey) =>  (attrs: u8) (attrs: u8);
        get_required_application_version [19, version::VersionInterval::from(version::Version::new(2, 0, 0))]: (meta_key: ContentMetaKey) =>  (version: u32) (version: u32);
        get_content_id_by_type_and_offset [20, version::VersionInterval::from(version::Version::new(5, 0, 0))]: (meta_key: ContentMetaKey, cnt_type: ContentType, id_offset: u8) =>  (id: ContentId) (id: ContentId);
        get_count [21, version::VersionInterval::from(version::Version::new(10, 0, 0))]: () => (count: u32) (count: u32);
        get_owner_application_id [22, version::VersionInterval::from(version::Version::new(10, 0, 0))]: (meta_key: ContentMetaKey) =>  (app_id: ApplicationId) (app_id: ApplicationId);
    }
}

ipc_sf_define_default_interface_client!(ContentManager);
ipc_sf_define_interface_trait! {
    trait ContentManager {
        open_content_meta_database [0, version::VersionInterval::all()]: (storage_id: StorageId) =>  (meta_db: ContentMetaDatabase) (meta_db: session_type!(ContentMetaDatabase));
    }
}
