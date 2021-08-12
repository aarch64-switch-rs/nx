use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum Age {
    Young,
    Normal,
    Old,
    #[default]
    All
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum Gender {
    Male,
    Female,
    #[default]
    All
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum FaceColor {
    Black,
    White,
    Asian,
    #[default]
    All
}

bit_enum! {
    SourceFlag (u32) {
        Database = bit!(0),
        Default = bit!(1)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum SpecialKeyCode {
    #[default]
    Normal = 0,
    Special = 0xA523B78F
}

pub type CreateId = util::Uuid;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct CharInfo {
    pub id: CreateId,
    pub name: util::CString16<11>,
    pub unk_1: u8,
    pub mii_color: u8,
    pub mii_sex: u8,
    pub mii_height: u8,
    pub mii_width: u8,
    pub unk_2: [u8; 2],
    pub mii_face_shape: u8,
    pub mii_face_color: u8,
    pub mii_wrinkles_style: u8,
    pub mii_makeup_style: u8,
    pub mii_hair_style: u8,
    pub mii_hair_color: u8,
    pub mii_has_hair_flipped: u8,
    pub mii_eye_style: u8,
    pub mii_eye_color: u8,
    pub mii_eye_size: u8,
    pub mii_eye_thickness: u8,
    pub mii_eye_angle: u8,
    pub mii_eye_pos_x: u8,
    pub mii_eye_pos_y: u8,
    pub mii_eyebrow_style: u8,
    pub mii_eyebrow_color: u8,
    pub mii_eyebrow_size: u8,
    pub mii_eyebrow_thickness: u8,
    pub mii_eyebrow_angle: u8,
    pub mii_eyebrow_pos_x: u8,
    pub mii_eyebrow_pos_y: u8,
    pub mii_nose_style: u8,
    pub mii_nose_size: u8,
    pub mii_nose_pos: u8,
    pub mii_mouth_style: u8,
    pub mii_mouth_color: u8,
    pub mii_mouth_size: u8,
    pub mii_mouth_thickness: u8,
    pub mii_mouth_pos: u8,
    pub mii_facial_hair_color: u8,
    pub mii_beard_style: u8,
    pub mii_mustache_style: u8,
    pub mii_mustache_size: u8,
    pub mii_mustache_pos: u8,
    pub mii_glasses_style: u8,
    pub mii_glasses_color: u8,
    pub mii_glasses_size: u8,
    pub mii_glasses_pos: u8,
    pub mii_has_mole: u8,
    pub mii_mole_size: u8,
    pub mii_mole_pos_x: u8,
    pub mii_mole_pos_y: u8,
    pub unk_3: u8
}

pub trait IDatabaseService {
    ipc_cmif_interface_define_command!(is_updated: (flag: SourceFlag) => (updated: bool));
    ipc_cmif_interface_define_command!(is_full: () => (full: bool));
    ipc_cmif_interface_define_command!(get_count: (flag: SourceFlag) => (count: u32));
    ipc_cmif_interface_define_command!(get_1: (flag: SourceFlag, out_char_infos: sf::OutMapAliasBuffer) => (count: u32));
    ipc_cmif_interface_define_command!(build_random: (age: Age, gender: Gender, face_color: FaceColor) => (char_info: CharInfo));
}

pub trait IStaticService {
    ipc_cmif_interface_define_command!(get_database_service: (key_code: SpecialKeyCode) => (database_service: mem::Shared<dyn sf::IObject>));
}