use crate::result::*;
use crate::service;
use crate::mem;
use crate::mem::alloc;
use crate::svc;
use crate::ipc::sf;
use crate::service::nv;
use crate::service::nv::INvDrvServices;
use crate::service::vi;
use crate::service::vi::IApplicationRootService;
use crate::service::vi::ISystemRootService;
use crate::service::vi::IManagerRootService;
use crate::service::vi::IApplicationDisplayService;
use crate::service::dispdrv;
use crate::service::applet;

pub mod rc;

pub mod parcel;

pub mod binder;

pub mod ioctl;

pub mod surface;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum Layout {
    #[default]
    Invalid = 0,
    Pitch = 1,
    Tiled = 2,
    BlockLinear = 3
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum DisplayScanFormat {
    #[default]
    Progressive = 0,
    Interlaced = 1
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum Kind {
    #[default]
    Pitch = 0x0,
    Z16 = 0x1,
    Z16_2C = 0x2,
    Z16_MS2_2C = 0x3,
    Z16_MS4_2C = 0x4,
    Z16_MS8_2C = 0x5,
    Z16_MS16_2C = 0x6,
    Z16_2Z = 0x7,
    Z16_MS2_2Z = 0x8,
    Z16_MS4_2Z = 0x9,
    Z16_MS8_2Z = 0xa,
    Z16_MS16_2Z = 0xb,
    Z16_4CZ = 0xc,
    Z16_MS2_4CZ = 0xd,
    Z16_MS4_4CZ = 0xe,
    Z16_MS8_4CZ = 0xf,
    Z16_MS16_4CZ = 0x10,
    S8Z24 = 0x11,
    S8Z24_1Z = 0x12,
    S8Z24_MS2_1Z = 0x13,
    S8Z24_MS4_1Z = 0x14,
    S8Z24_MS8_1Z = 0x15,
    S8Z24_MS16_1Z = 0x16,
    S8Z24_2CZ = 0x17,
    S8Z24_MS2_2CZ = 0x18,
    S8Z24_MS4_2CZ = 0x19,
    S8Z24_MS8_2CZ = 0x1a,
    S8Z24_MS16_2CZ = 0x1b,
    S8Z24_2CS = 0x1C,
    S8Z24_MS2_2CS = 0x1d,
    S8Z24_MS4_2CS = 0x1e,
    S8Z24_MS8_2CS = 0x1f,
    S8Z24_MS16_2CS = 0x20,
    S8Z24_4CSZV = 0x21,
    S8Z24_MS2_4CSZV = 0x22,
    S8Z24_MS4_4CSZV = 0x23,
    S8Z24_MS8_4CSZV = 0x24,
    S8Z24_MS16_4CSZV = 0x25,
    V8Z24_MS4_VC12 = 0x26,
    V8Z24_MS4_VC4 = 0x27,
    V8Z24_MS8_VC8 = 0x28,
    V8Z24_MS8_VC24 = 0x29,
    S8 = 0x2a,
    S8_2S = 0x2b,
    V8Z24_MS4_VC12_1ZV = 0x2e,
    V8Z24_MS4_VC4_1ZV = 0x2f,
    V8Z24_MS8_VC8_1ZV = 0x30,
    V8Z24_MS8_VC24_1ZV = 0x31,
    V8Z24_MS4_VC12_2CS = 0x32,
    V8Z24_MS4_VC4_2CS = 0x33,
    V8Z24_MS8_VC8_2CS = 0x34,
    V8Z24_MS8_VC24_2CS = 0x35,
    V8Z24_MS4_VC12_2CZV = 0x3a,
    V8Z24_MS4_VC4_2CZV = 0x3b,
    V8Z24_MS8_VC8_2CZV = 0x3c,
    V8Z24_MS8_VC24_2CZV = 0x3d,
    V8Z24_MS4_VC12_2ZV = 0x3e,
    V8Z24_MS4_VC4_2ZV = 0x3f,
    V8Z24_MS8_VC8_2ZV = 0x40,
    V8Z24_MS8_VC24_2ZV = 0x41,
    V8Z24_MS4_VC12_4CSZV = 0x42,
    V8Z24_MS4_VC4_4CSZV = 0x43,
    V8Z24_MS8_VC8_4CSZV = 0x44,
    V8Z24_MS8_VC24_4CSZV = 0x45,
    Z24S8 = 0x46,
    Z24S8_1Z = 0x47,
    Z24S8_MS2_1Z = 0x48,
    Z24S8_MS4_1Z = 0x49,
    Z24S8_MS8_1Z = 0x4a,
    Z24S8_MS16_1Z = 0x4b,
    Z24S8_2CS = 0x4c,
    Z24S8_MS2_2CS = 0x4d,
    Z24S8_MS4_2CS = 0x4e,
    Z24S8_MS8_2CS = 0x4f,
    Z24S8_MS16_2CS = 0x50,
    Z24S8_2CZ = 0x51,
    Z24S8_MS2_2CZ = 0x52,
    Z24S8_MS4_2CZ = 0x53,
    Z24S8_MS8_2CZ = 0x54,
    Z24S8_MS16_2CZ = 0x55,
    Z24S8_4CSZV = 0x56,
    Z24S8_MS2_4CSZV = 0x57,
    Z24S8_MS4_4CSZV = 0x58,
    Z24S8_MS8_4CSZV = 0x59,
    Z24S8_MS16_4CSZV = 0x5a,
    Z24V8_MS4_VC12 = 0x5b,
    Z24V8_MS4_VC4 = 0x5C,
    Z24V8_MS8_VC8 = 0x5d,
    Z24V8_MS8_VC24 = 0x5e,
    Z24V8_MS4_VC12_1ZV = 0x63,
    Z24V8_MS4_VC4_1ZV = 0x64,
    Z24V8_MS8_VC8_1ZV = 0x65,
    Z24V8_MS8_VC24_1ZV = 0x66,
    Z24V8_MS4_VC12_2CS = 0x67,
    Z24V8_MS4_VC4_2CS = 0x68,
    Z24V8_MS8_VC8_2CS = 0x69,
    Z24V8_MS8_VC24_2CS = 0x6a,
    Z24V8_MS4_VC12_2CZV = 0x6f,
    Z24V8_MS4_VC4_2CZV = 0x70,
    Z24V8_MS8_VC8_2CZV = 0x71,
    Z24V8_MS8_VC24_2CZV = 0x72,
    Z24V8_MS4_VC12_2ZV = 0x73,
    Z24V8_MS4_VC4_2ZV = 0x74,
    Z24V8_MS8_VC8_2ZV = 0x75,
    Z24V8_MS8_VC24_2ZV = 0x76,
    Z24V8_MS4_VC12_4CSZV = 0x77,
    Z24V8_MS4_VC4_4CSZV = 0x78,
    Z24V8_MS8_VC8_4CSZV = 0x79,
    Z24V8_MS8_VC24_4CSZV = 0x7a,
    ZF32 = 0x7b,
    ZF32_1Z = 0x7C,
    ZF32_MS2_1Z = 0x7d,
    ZF32_MS4_1Z = 0x7e,
    ZF32_MS8_1Z = 0x7f,
    ZF32_MS16_1Z = 0x80,
    ZF32_2CS = 0x81,
    ZF32_MS2_2CS = 0x82,
    ZF32_MS4_2CS = 0x83,
    ZF32_MS8_2CS = 0x84,
    ZF32_MS16_2CS = 0x85,
    ZF32_2CZ = 0x86,
    ZF32_MS2_2CZ = 0x87,
    ZF32_MS4_2CZ = 0x88,
    ZF32_MS8_2CZ = 0x89,
    ZF32_MS16_2CZ = 0x8a,
    X8Z24_X16V8S8_MS4_VC12 = 0x8b,
    X8Z24_X16V8S8_MS4_VC4 = 0x8c,
    X8Z24_X16V8S8_MS8_VC8 = 0x8d,
    X8Z24_X16V8S8_MS8_VC24 = 0x8e,
    X8Z24_X16V8S8_MS4_VC12_1CS = 0x8f,
    X8Z24_X16V8S8_MS4_VC4_1CS = 0x90,
    X8Z24_X16V8S8_MS8_VC8_1CS = 0x91,
    X8Z24_X16V8S8_MS8_VC24_1CS = 0x92,
    X8Z24_X16V8S8_MS4_VC12_1ZV = 0x97,
    X8Z24_X16V8S8_MS4_VC4_1ZV = 0x98,
    X8Z24_X16V8S8_MS8_VC8_1ZV = 0x99,
    X8Z24_X16V8S8_MS8_VC24_1ZV = 0x9a,
    X8Z24_X16V8S8_MS4_VC12_1CZV = 0x9b,
    X8Z24_X16V8S8_MS4_VC4_1CZV = 0x9c,
    X8Z24_X16V8S8_MS8_VC8_1CZV = 0x9d,
    X8Z24_X16V8S8_MS8_VC24_1CZV = 0x9e,
    X8Z24_X16V8S8_MS4_VC12_2CS = 0x9f,
    X8Z24_X16V8S8_MS4_VC4_2CS = 0xa0,
    X8Z24_X16V8S8_MS8_VC8_2CS = 0xa1,
    X8Z24_X16V8S8_MS8_VC24_2CS = 0xa2,
    X8Z24_X16V8S8_MS4_VC12_2CSZV = 0xa3,
    X8Z24_X16V8S8_MS4_VC4_2CSZV = 0xa4,
    X8Z24_X16V8S8_MS8_VC8_2CSZV = 0xa5,
    X8Z24_X16V8S8_MS8_VC24_2CSZV = 0xa6,
    ZF32_X16V8S8_MS4_VC12 = 0xa7,
    ZF32_X16V8S8_MS4_VC4 = 0xa8,
    ZF32_X16V8S8_MS8_VC8 = 0xa9,
    ZF32_X16V8S8_MS8_VC24 = 0xaa,
    ZF32_X16V8S8_MS4_VC12_1CS = 0xab,
    ZF32_X16V8S8_MS4_VC4_1CS = 0xac,
    ZF32_X16V8S8_MS8_VC8_1CS = 0xad,
    ZF32_X16V8S8_MS8_VC24_1CS = 0xae,
    ZF32_X16V8S8_MS4_VC12_1ZV = 0xb3,
    ZF32_X16V8S8_MS4_VC4_1ZV = 0xb4,
    ZF32_X16V8S8_MS8_VC8_1ZV = 0xb5,
    ZF32_X16V8S8_MS8_VC24_1ZV = 0xb6,
    ZF32_X16V8S8_MS4_VC12_1CZV = 0xb7,
    ZF32_X16V8S8_MS4_VC4_1CZV = 0xb8,
    ZF32_X16V8S8_MS8_VC8_1CZV = 0xb9,
    ZF32_X16V8S8_MS8_VC24_1CZV = 0xba,
    ZF32_X16V8S8_MS4_VC12_2CS = 0xbb,
    ZF32_X16V8S8_MS4_VC4_2CS = 0xbc,
    ZF32_X16V8S8_MS8_VC8_2CS = 0xbd,
    ZF32_X16V8S8_MS8_VC24_2CS = 0xbe,
    ZF32_X16V8S8_MS4_VC12_2CSZV = 0xbf,
    ZF32_X16V8S8_MS4_VC4_2CSZV = 0xc0,
    ZF32_X16V8S8_MS8_VC8_2CSZV = 0xc1,
    ZF32_X16V8S8_MS8_VC24_2CSZV = 0xc2,
    ZF32_X24S8 = 0xc3,
    ZF32_X24S8_1CS = 0xc4,
    ZF32_X24S8_MS2_1CS = 0xc5,
    ZF32_X24S8_MS4_1CS = 0xc6,
    ZF32_X24S8_MS8_1CS = 0xc7,
    ZF32_X24S8_MS16_1CS = 0xc8,
    SmskedMessage = 0xca,
    SmhostMessage = 0xcb,
    C64_MS2_2CRA = 0xcd,
    ZF32_X24S8_2CSZV = 0xce,
    ZF32_X24S8_MS2_2CSZV = 0xcf,
    ZF32_X24S8_MS4_2CSZV = 0xd0,
    ZF32_X24S8_MS8_2CSZV = 0xd1,
    ZF32_X24S8_MS16_2CSZV = 0xd2,
    ZF32_X24S8_2CS = 0xd3,
    ZF32_X24S8_MS2_2CS = 0xd4,
    ZF32_X24S8_MS4_2CS = 0xd5,
    ZF32_X24S8_MS8_2CS = 0xd6,
    ZF32_X24S8_MS16_2CS = 0xd7,
    C32_2C = 0xd8,
    C32_2CBR = 0xd9,
    C32_2CBA = 0xda,
    C32_2CRA = 0xdb,
    C32_2BRA = 0xdc,
    C32_MS2_2C = 0xdd,
    C32_MS2_2CBR = 0xde,
    C32_MS2_2CRA = 0xcc,
    C32_MS4_2C = 0xdf,
    C32_MS4_2CBR = 0xe0,
    C32_MS4_2CBA = 0xe1,
    C32_MS4_2CRA = 0xe2,
    C32_MS4_2BRA = 0xe3,
    C32_MS8_MS16_2C = 0xe4,
    C32_MS8_MS16_2CRA = 0xe5,
    C64_2C = 0xe6,
    C64_2CBR = 0xe7,
    C64_2CBA = 0xe8,
    C64_2CRA = 0xe9,
    C64_2BRA = 0xea,
    C64_MS2_2C = 0xeb,
    C64_MS2_2CBR = 0xec,
    C64_MS4_2C = 0xed,
    C64_MS4_2CBR = 0xee,
    C64_MS4_2CBA = 0xef,
    C64_MS4_2CRA = 0xf0,
    C64_MS4_2BRA = 0xf1,
    C64_MS8_MS16_2C = 0xf2,
    C64_MS8_MS16_2CRA = 0xf3,
    C128_2C = 0xf4,
    C128_2CR = 0xf5,
    C128_MS2_2C = 0xf6,
    C128_MS2_2CR = 0xf7,
    C128_MS4_2C = 0xf8,
    C128_MS4_2CR = 0xf9,
    C128_MS8_MS16_2C = 0xfa,
    C128_MS8_MS16_2CR = 0xfb,
    X8C24 = 0xfc,
    PitchNoSwizzle = 0xfd,
    Generic_16BX2 = 0xfe,
    Invalid = 0xff
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u64)]
pub enum ColorFormat {
    #[default]
    Unspecified = 0,
    NonColor8 = 0x0009200408,
    NonColor16 = 0x0009200A10,
    NonColor24 = 0x0009201A18,
    NonColor32 = 0x0009201C20,
    X4C4 = 0x0009210508,
    A4L4 = 0x0100490508,
    A8L8 = 0x0100490E10,
    Float_A16L16 = 0x0100495D20,
    A1B5G5R5 = 0x0100531410,
    A4B4G4R4 = 0x0100531510,
    A5B5G5R1 = 0x0100531810,
    A2B10G10R10 = 0x0100532020,
    A8B8G8R8 = 0x0100532120,
    A16B16G16R16 = 0x0100532740,
    Float_A16B16G16R16 = 0x0100536740,
    A1R5G5B5 = 0x0100D11410,
    A4R4G4B4 = 0x0100D11510,
    A5R1G5B5 = 0x0100D11610,
    A2R10G10B10 = 0x0100D12020,
    A8R8G8B8 = 0x0100D12120,
    A1 = 0x0101240101,
    A2 = 0x0101240202,
    A4 = 0x0101240304,
    A8 = 0x0101240408,
    A16 = 0x0101240A10,
    A32 = 0x0101241C20,
    Float_A16 = 0x0101244A10,
    L4A4 = 0x0102000508,
    L8A8 = 0x0102000E10,
    B4G4R4A4 = 0x01060A1510,
    B5G5R1A5 = 0x01060A1710,
    B5G5R5A1 = 0x01060A1810,
    B8G8R8A8 = 0x01060A2120,
    B10G10R10A2 = 0x01060A2320,
    R1G5B5A5 = 0x0106881410,
    R4G4B4A4 = 0x0106881510,
    R5G5B5A1 = 0x0106881810,
    R8G8B8A8 = 0x0106882120,
    R10G10B10A2 = 0x0106882320,
    L1 = 0x010A000101,
    L2 = 0x010A000202,
    L4 = 0x010A000304,
    L8 = 0x010A000408,
    L16 = 0x010A000A10,
    L32 = 0x010A001C20,
    Float_L16 = 0x010A004A10,
    B5G6R5 = 0x010A0A1210,
    B6G5R5 = 0x010A0A1310,
    B5G5R5X1 = 0x010A0A1810,
    B8_G8_R8 = 0x010A0A1918,
    B8G8R8X8 = 0x010A0A2120,
    Float_B10G11R11 = 0x010A0A5E20,
    X1B5G5R5 = 0x010A531410,
    X8B8G8R8 = 0x010A532120,
    X16B16G16R16 = 0x010A532740,
    Float_X16B16G16R16 = 0x010A536740,
    R3G3B2 = 0x010A880608,
    R5G5B6 = 0x010A881110,
    R5G6B5 = 0x010A881210,
    R5G5B5X1 = 0x010A881810,
    R8_G8_B8 = 0x010A881918,
    R8G8B8X8 = 0x010A882120,
    X1R5G5B5 = 0x010AD11410,
    X8R8G8B8 = 0x010AD12120,
    RG8 = 0x010B080E10,
    R16G16 = 0x010B081D20,
    Float_R16G16 = 0x010B085D20,
    R8 = 0x010B200408,
    R16 = 0x010B200A10,
    Float_R16 = 0x010B204A10,
    A2B10G10R10_sRGB = 0x0200532020,
    A8B8G8R8_sRGB = 0x0200532120,
    A16B16G16R16_sRGB = 0x0200532740,
    A2R10G10B10_sRGB = 0x0200D12020,
    B10G10R10A2_sRGB = 0x02060A2320,
    R10G10B10A2_sRGB = 0x0206882320,
    X8B8G8R8_sRGB = 0x020A532120,
    X16B16G16R16_sRGB = 0x020A532740,
    A2B10G10R10_709 = 0x0300532020,
    A8B8G8R8_709 = 0x0300532120,
    A16B16G16R16_709 = 0x0300532740,
    A2R10G10B10_709 = 0x0300D12020,
    B10G10R10A2_709 = 0x03060A2320,
    R10G10B10A2_709 = 0x0306882320,
    X8B8G8R8_709 = 0x030A532120,
    X16B16G16R16_709 = 0x030A532740,
    A2B10G10R10_709_Linear = 0x0400532020,
    A8B8G8R8_709_Linear = 0x0400532120,
    A16B16G16R16_709_Linear = 0x0400532740,
    A2R10G10B10_709_Linear = 0x0400D12020,
    B10G10R10A2_709_Linear = 0x04060A2320,
    R10G10B10A2_709_Linear = 0x0406882320,
    X8B8G8R8_709_Linear = 0x040A532120,
    X16B16G16R16_709_Linear = 0x040A532740,
    Float_A16B16G16R16_scRGB_Linear = 0x0500536740,
    A2B10G10R10_2020 = 0x0600532020,
    A8B8G8R8_2020 = 0x0600532120,
    A16B16G16R16_2020 = 0x0600532740,
    A2R10G10B10_2020 = 0x0600D12020,
    B10G10R10A2_2020 = 0x06060A2320,
    R10G10B10A2_2020 = 0x0606882320,
    X8B8G8R8_2020 = 0x060A532120,
    X16B16G16R16_2020 = 0x060A532740,
    A2B10G10R10_2020_Linear = 0x0700532020,
    A8B8G8R8_2020_Linear = 0x0700532120,
    A16B16G16R16_2020_Linear = 0x0700532740,
    Float_A16B16G16R16_2020_Linear = 0x0700536740,
    A2R10G10B10_2020_Linear = 0x0700D12020,
    B10G10R10A2_2020_Linear = 0x07060A2320,
    R10G10B10A2_2020_Linear = 0x0706882320,
    X8B8G8R8_2020_Linear = 0x070A532120,
    X16B16G16R16_2020_Linear = 0x070A532740,
    Float_A16B16G16R16_2020_PQ = 0x0800536740,
    A4I4 = 0x0901210508,
    A8I8 = 0x0901210E10,
    I4A4 = 0x0903200508,
    I8A8 = 0x0903200E10,
    I1 = 0x0909200101,
    I2 = 0x0909200202,
    I4 = 0x0909200304,
    I8 = 0x0909200408,
    A8Y8U8V8 = 0x0A00D12120,
    A16Y16U16V16 = 0x0A00D12740,
    Y8U8V8A8 = 0x0A06882120,
    V8_U8 = 0x0A080C0710,
    V8U8 = 0x0A080C0E10,
    V10U10 = 0x0A08142220,
    V12U12 = 0x0A08142420,
    V8 = 0x0A08240408,
    V10 = 0x0A08240F10,
    V12 = 0x0A08241010,
    U8_V8 = 0x0A08440710,
    U8V8 = 0x0A08440E10,
    U10V10 = 0x0A08842220,
    U12V12 = 0x0A08842420,
    U8 = 0x0A09040408,
    U10 = 0x0A09040F10,
    U12 = 0x0A09041010,
    Y8 = 0x0A09200408,
    Y10 = 0x0A09200F10,
    Y12 = 0x0A09201010,
    YVYU = 0x0A0A500810,
    VYUY = 0x0A0A500910,
    YUYV = 0x0A0A880810,
    UYVY = 0x0A0A880910,
    Y8_U8_V8 = 0x0A0A881918,
    V8_U8_RR = 0x0B080C0710,
    V8U8_RR = 0x0B080C0E10,
    V8_RR = 0x0B08240408,
    U8_V8_RR = 0x0B08440710,
    U8V8_RR = 0x0B08440E10,
    U8_RR = 0x0B09040408,
    Y8_RR = 0x0B09200408,
    V8_U8_ER = 0x0C080C0710,
    V8U8_ER = 0x0C080C0E10,
    V8_ER = 0x0C08240408,
    U8_V8_ER = 0x0C08440710,
    U8V8_ER = 0x0C08440E10,
    U8_ER = 0x0C09040408,
    Y8_ER = 0x0C09200408,
    V8_U8_709 = 0x0D080C0710,
    V8U8_709 = 0x0D080C0E10,
    V10U10_709 = 0x0D08142220,
    V12U12_709 = 0x0D08142420,
    V8_709 = 0x0D08240408,
    V10_709 = 0x0D08240F10,
    V12_709 = 0x0D08241010,
    U8_V8_709 = 0x0D08440710,
    U8V8_709 = 0x0D08440E10,
    U10V10_709 = 0x0D08842220,
    U12V12_709 = 0x0D08842420,
    U8_709 = 0x0D09040408,
    U10_709 = 0x0D09040F10,
    U12_709 = 0x0D09041010,
    Y8_709 = 0x0D09200408,
    Y10_709 = 0x0D09200F10,
    Y12_709 = 0x0D09201010,
    V8_U8_709_ER = 0x0E080C0710,
    V8U8_709_ER = 0x0E080C0E10,
    V10U10_709_ER = 0x0E08142220,
    V12U12_709_ER = 0x0E08142420,
    V8_709_ER = 0x0E08240408,
    V10_709_ER = 0x0E08240F10,
    V12_709_ER = 0x0E08241010,
    U8_V8_709_ER = 0x0E08440710,
    U8V8_709_ER = 0x0E08440E10,
    U10V10_709_ER = 0x0E08842220,
    U12V12_709_ER = 0x0E08842420,
    U8_709_ER = 0x0E09040408,
    U10_709_ER = 0x0E09040F10,
    U12_709_ER = 0x0E09041010,
    Y8_709_ER = 0x0E09200408,
    Y10_709_ER = 0x0E09200F10,
    Y12_709_ER = 0x0E09201010,
    V10U10_2020 = 0x0F08142220,
    V12U12_2020 = 0x0F08142420,
    V10_2020 = 0x0F08240F10,
    V12_2020 = 0x0F08241010,
    U10V10_2020 = 0x0F08842220,
    U12V12_2020 = 0x0F08842420,
    U10_2020 = 0x0F09040F10,
    U12_2020 = 0x0F09041010,
    Y10_2020 = 0x0F09200F10,
    Y12_2020 = 0x0F09201010,
    Bayer8RGGB = 0x1009200408,
    Bayer16RGGB = 0x1009200A10,
    BayerS16RGGB = 0x1009208A10,
    X2Bayer14RGGB = 0x1009210B10,
    X4Bayer12RGGB = 0x1009210C10,
    X6Bayer10RGGB = 0x1009210D10,
    Bayer8BGGR = 0x1109200408,
    Bayer16BGGR = 0x1109200A10,
    BayerS16BGGR = 0x1109208A10,
    X2Bayer14BGGR = 0x1109210B10,
    X4Bayer12BGGR = 0x1109210C10,
    X6Bayer10BGGR = 0x1109210D10,
    Bayer8GRBG = 0x1209200408,
    Bayer16GRBG = 0x1209200A10,
    BayerS16GRBG = 0x1209208A10,
    X2Bayer14GRBG = 0x1209210B10,
    X4Bayer12GRBG = 0x1209210C10,
    X6Bayer10GRBG = 0x1209210D10,
    Bayer8GBRG = 0x1309200408,
    Bayer16GBRG = 0x1309200A10,
    BayerS16GBRG = 0x1309208A10,
    X2Bayer14GBRG = 0x1309210B10,
    X4Bayer12GBRG = 0x1309210C10,
    X6Bayer10GBRG = 0x1309210D10,
    XYZ = 0x140A886640
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum PixelFormat {
    #[default]
    Invalid = 0,
    RGBA_8888 = 1,
    RGBX_8888 = 2,
    RGB_888 = 3,
    RGB_565 = 4,
    BGRA_8888 = 5,
    RGBA_5551 = 6,
    RGBA_4444 = 7,
    YCRB_420_SP = 17,
    Raw16 = 32,
    Blob = 33,
    ImplementationDefined = 34,
    YCBCR_420_888 = 35,
    Y8 = 0x20203859,
    Y16 = 0x20363159,
    YV12 = 0x32315659
}

define_bit_enum! {
    GraphicsAllocatorUsage (u32) {
        SoftwareReadNever = 0,
        SoftwareReadRarely = 0x2,
        SoftwareReadOften = 0x3,
        SoftwareReadMask = 0xF,

        SoftwareWriteNever = 0,
        SoftwareWriteRarely = 0x20,
        SoftwareWriteOften = 0x30,
        SoftwareWriteMask = 0xF0,

        HardwareTexture = 0x100,
        HardwareRender = 0x200,
        Hardware2d = 0x400,
        HardwareComposer = 0x800,
        HardwareFramebuffer = 0x1000,
        HardwareExternalDisplay = 0x2000,
        HardwareProtected = 0x4000,
        HardwareCursor = 0x8000,
        HardwareVideoEncoder = 0x10000,
        HardwareCameraWrite = 0x20000,
        HardwareCameraRead = 0x40000,
        HardwareCameraZSL = 0x60000,
        HardwareCameraMask = 0x60000,
        HardwareMask = 0x71F00,
        RenderScript = 0x100000   
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(i32)]
pub enum ConnectionApi {
    #[default]
    Invalid = 0,
    EGL = 1,
    Cpu = 2,
    Media = 3,
    Camera = 4
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum DisconnectMode {
    #[default]
    Api,
    AllLocal
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct QueueBufferOutput {
    pub width: u32,
    pub height: u32,
    pub transform_hint: u32,
    pub pending_buffer_count: u32
}

impl QueueBufferOutput {
    pub const fn new() -> Self {
        Self { width: 0, height: 0, transform_hint: 0, pending_buffer_count: 0 }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Plane {
    pub width: u32,
    pub height: u32,
    pub color_format: ColorFormat,
    pub layout: Layout,
    pub pitch: u32,
    pub map_handle: u32,
    pub offset: u32,
    pub kind: Kind,
    pub block_height_log2: u32,
    pub display_scan_format: DisplayScanFormat,
    pub second_field_offset: u32,
    pub flags: u64,
    pub size: usize,
    pub unk: [u32; 6]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct GraphicBufferHeader {
    pub magic: u32,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub pixel_format: PixelFormat,
    pub gfx_alloc_usage: GraphicsAllocatorUsage,
    pub pid: u32,
    pub refcount: u32,
    pub fd_count: u32,
    pub buffer_size: u32
}

pub const GRAPHIC_BUFFER_HEADER_MAGIC: u32 = u32::from_be_bytes(*b"GBFR");

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
#[repr(packed)]
pub struct GraphicBuffer {
    pub header: GraphicBufferHeader,
    pub null: u32,
    pub map_id: u32,
    pub zero: u32,
    pub magic: u32,
    pub pid: u32,
    pub buffer_type: u32,
    pub gfx_alloc_usage: GraphicsAllocatorUsage,
    pub pixel_format: PixelFormat,
    pub external_pixel_format: PixelFormat,
    pub stride: u32,
    pub full_size: u32,
    pub plane_count: u32,
    pub zero_2: u32,
    pub planes: [Plane; 3],
    pub unused: u64
}

pub const GRAPHIC_BUFFER_MAGIC: u32 = 0xDAFFCAFF;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Fence {
    id: u32,
    value: u32
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct MultiFence {
    fence_count: u32,
    fences: [Fence; 4]
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum Transform {
    #[default]
    Invalid = 0,
    FlipH = 1,
    FlipV = 2,
    Rotate90 = 4,
    Rotate180 = 3,
    Rotate270 = 7
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
#[repr(packed)]
pub struct QueueBufferInput {
    timestamp: i64,
    is_auto_timestamp: i32,
    crop: Rect,
    scaling_mode: i32,
    transform: Transform,
    sticky_transform: u32,
    unk: u32,
    swap_interval: u32,
    fences: MultiFence
}

pub const BLOCK_HEIGHT_LOG2: u32 = 4;
pub const BLOCK_HEIGHT: u32 = 8 * (1 << BLOCK_HEIGHT_LOG2);

pub const fn calculate_bpp(color_fmt: ColorFormat) -> u32 {
    (((color_fmt as u64) >> 3) & 0x1F) as u32
}

pub const fn align_width(bpp: u32, width: u32) -> u32 {
    ((width * bpp + 63) & !63) / bpp
}

pub const fn align_height(height: u32) -> u32 {
    (height + BLOCK_HEIGHT - 1) & !(BLOCK_HEIGHT - 1)
}

const NVHOST_PATH: &str = nul!("/dev/nvhost-as-gpu");
const NVMAP_PATH: &str = nul!("/dev/nvmap");
const NVHOSTCTRL_PATH: &str = nul!("/dev/nvhost-ctrl");

const SIZE_FACTOR: f32 = 1.5; // 1920x1080 / 1280x720

pub const SCREEN_WIDTH: u32 = 1280;
pub const SCREEN_HEIGHT: u32 = 720;

pub enum LayerZ {
    Max,
    Min,
    Value(i64)
}

pub enum NvDrvServiceKind {
    Application,
    Applet,
    System
}

pub enum ViServiceKind {
    Application,
    System,
    Manager
}

#[allow(unreachable_patterns)]
pub fn convert_nv_error_code(err: nv::ErrorCode) -> Result<()> {
    match err {
        nv::ErrorCode::Success => Ok(()),
        nv::ErrorCode::NotImplemented => rc::ResultNvErrorCodeNotImplemented::make_err(),
        nv::ErrorCode::NotSupported => rc::ResultNvErrorCodeNotSupported::make_err(),
        nv::ErrorCode::NotInitialized => rc::ResultNvErrorCodeNotInitialized::make_err(),
        nv::ErrorCode::InvalidParameter => rc::ResultNvErrorCodeInvalidParameter::make_err(),
        nv::ErrorCode::TimeOut => rc::ResultNvErrorCodeTimeOut::make_err(),
        nv::ErrorCode::InsufficientMemory => rc::ResultNvErrorCodeInsufficientMemory::make_err(),
        nv::ErrorCode::ReadOnlyAttribute => rc::ResultNvErrorCodeReadOnlyAttribute::make_err(),
        nv::ErrorCode::InvalidState => rc::ResultNvErrorCodeInvalidState::make_err(),
        nv::ErrorCode::InvalidAddress => rc::ResultNvErrorCodeInvalidAddress::make_err(),
        nv::ErrorCode::InvalidSize => rc::ResultNvErrorCodeInvalidSize::make_err(),
        nv::ErrorCode::InvalidValue => rc::ResultNvErrorCodeInvalidValue::make_err(),
        nv::ErrorCode::AlreadyAllocated => rc::ResultNvErrorCodeAlreadyAllocated::make_err(),
        nv::ErrorCode::Busy => rc::ResultNvErrorCodeBusy::make_err(),
        nv::ErrorCode::ResourceError => rc::ResultNvErrorCodeResourceError::make_err(),
        nv::ErrorCode::CountMismatch => rc::ResultNvErrorCodeCountMismatch::make_err(),
        nv::ErrorCode::SharedMemoryTooSmall => rc::ResultNvErrorCodeSharedMemoryTooSmall::make_err(),
        nv::ErrorCode::FileOperationFailed => rc::ResultNvErrorCodeFileOperationFailed::make_err(),
        nv::ErrorCode::IoctlFailed => rc::ResultNvErrorCodeIoctlFailed::make_err(),
        _ => rc::ResultNvErrorCodeInvalid::make_err(),
    }
}

pub struct Context {
    vi_service: mem::Shared<dyn sf::IObject>,
    nvdrv_service: mem::Shared<dyn INvDrvServices>,
    application_display_service: mem::Shared<dyn IApplicationDisplayService>,
    hos_binder_driver: mem::Shared<dyn dispdrv::IHOSBinderDriver>,
    transfer_mem: alloc::Buffer<u8>,
    transfer_mem_handle: svc::Handle,
    nvhost_fd: u32,
    nvmap_fd: u32,
    nvhostctrl_fd: u32,
}

impl Context {
    pub fn new(nv_kind: NvDrvServiceKind, vi_kind: ViServiceKind, transfer_mem_size: usize) -> Result<Self> {
        // Note: need to store a reference of the vi-service since it works as a domain, thus closing the original handle leaves all opened interfaces unusable
        // Storing it as a IObject shared-ptr since different vi services have different base interfaces...
        let (vi_srv, application_display_srv) = match vi_kind {
            ViServiceKind::Manager => {
                let vi_srv = service::new_service_object::<vi::ManagerRootService>()?;
                let app_disp_srv: mem::Shared<dyn IApplicationDisplayService> = vi_srv.get().get_display_service(vi::DisplayServiceMode::Privileged)?;

                (vi_srv as mem::Shared<dyn sf::IObject>, app_disp_srv)
            },
            ViServiceKind::System => {
                let vi_srv = service::new_service_object::<vi::SystemRootService>()?;
                let app_disp_srv: mem::Shared<dyn IApplicationDisplayService> = vi_srv.get().get_display_service(vi::DisplayServiceMode::Privileged)?;

                (vi_srv as mem::Shared<dyn sf::IObject>, app_disp_srv)
            },
            ViServiceKind::Application => {
                let vi_srv = service::new_service_object::<vi::ApplicationRootService>()?;
                let app_disp_srv: mem::Shared<dyn IApplicationDisplayService> = vi_srv.get().get_display_service(vi::DisplayServiceMode::User)?;

                (vi_srv as mem::Shared<dyn sf::IObject>, app_disp_srv)
            }
        };

        let nvdrv_srv = match nv_kind {
            NvDrvServiceKind::Application => {
                service::new_service_object::<nv::ApplicationNvDrvService>()? as mem::Shared<dyn INvDrvServices>
            },
            NvDrvServiceKind::Applet => {
                service::new_service_object::<nv::AppletNvDrvService>()? as mem::Shared<dyn INvDrvServices>
            },
            NvDrvServiceKind::System => {
                service::new_service_object::<nv::SystemNvDrvService>()? as mem::Shared<dyn INvDrvServices>
            }
        };

        Self::from(vi_srv, application_display_srv, nvdrv_srv, transfer_mem_size)
    }

    pub fn from(vi_srv: mem::Shared<dyn sf::IObject>, application_display_srv: mem::Shared<dyn IApplicationDisplayService>, nvdrv_srv: mem::Shared<dyn INvDrvServices>, transfer_mem_size: usize) -> Result<Self> {
        let transfer_mem = alloc::Buffer::new(alloc::PAGE_ALIGNMENT, transfer_mem_size)?;
        let transfer_mem_handle = svc::create_transfer_memory(transfer_mem.ptr, transfer_mem_size, svc::MemoryPermission::None())?;
        nvdrv_srv.get().initialize(transfer_mem_size as u32, sf::Handle::from(svc::CURRENT_PROCESS_PSEUDO_HANDLE), sf::Handle::from(transfer_mem_handle))?;

        let (nvhost_fd, nvhost_err) = nvdrv_srv.get().open(sf::Buffer::from_array(NVHOST_PATH.as_bytes()))?;
        convert_nv_error_code(nvhost_err)?;
        let (nvmap_fd, nvmap_err) = nvdrv_srv.get().open(sf::Buffer::from_array(NVMAP_PATH.as_bytes()))?;
        convert_nv_error_code(nvmap_err)?;
        let (nvhostctrl_fd, nvhostctrl_err) = nvdrv_srv.get().open(sf::Buffer::from_array(NVHOSTCTRL_PATH.as_bytes()))?;
        convert_nv_error_code(nvhostctrl_err)?;
        
        let hos_binder_drv = application_display_srv.get().get_relay_service()?;
        Ok(Self { vi_service: vi_srv, nvdrv_service: nvdrv_srv, application_display_service: application_display_srv, hos_binder_driver: hos_binder_drv, transfer_mem, transfer_mem_handle, nvhost_fd, nvmap_fd, nvhostctrl_fd })
    }

    pub fn get_nvdrv_service(&self) -> mem::Shared<dyn INvDrvServices> {
        self.nvdrv_service.clone()
    }

    pub fn get_application_display_service(&self) -> mem::Shared<dyn IApplicationDisplayService> {
        self.application_display_service.clone()
    }

    pub fn get_hos_binder_driver(&self) -> mem::Shared<dyn dispdrv::IHOSBinderDriver> {
        self.hos_binder_driver.clone()
    }

    fn stray_layer_destroy(layer_id: vi::LayerId, application_display_service: mem::Shared<dyn IApplicationDisplayService>) -> Result<()> {
        application_display_service.get().destroy_stray_layer(layer_id)
    }

    fn managed_layer_destroy(layer_id: vi::LayerId, application_display_service: mem::Shared<dyn IApplicationDisplayService>) -> Result<()> {
        let manager_display_service = application_display_service.get().get_manager_display_service()?;
        manager_display_service.get().destroy_managed_layer(layer_id)
    }

    fn create_surface_impl(&mut self, buffer_count: u32, display_id: vi::DisplayId, layer_id: vi::LayerId, width: u32, height: u32, color_fmt: ColorFormat, pixel_fmt: PixelFormat, layout: Layout, layer_destroy_fn: surface::LayerDestroyFn, native_window: parcel::ParcelPayload) -> Result<surface::Surface> {
        let mut parcel = parcel::Parcel::new();
        parcel.load_from(native_window);
        
        let data: parcel::ParcelData = parcel.read()?;
        surface::Surface::new(data.handle, self.nvdrv_service.clone(), self.application_display_service.clone(), self.nvhost_fd, self.nvmap_fd, self.nvhostctrl_fd, self.hos_binder_driver.clone(), buffer_count, display_id, layer_id, width, height, color_fmt, pixel_fmt, layout, layer_destroy_fn)
    }

    pub fn create_stray_layer_surface(&mut self, display_name: &str, buffer_count: u32, color_fmt: ColorFormat, pixel_fmt: PixelFormat, layout: Layout) -> Result<surface::Surface> {
        let display_id = self.application_display_service.get().open_display(vi::DisplayName::from_str(display_name))?;
        let native_window = parcel::ParcelPayload::new();
        let (layer_id, _) = self.application_display_service.get().create_stray_layer(vi::LayerFlags::Default(), display_id, sf::Buffer::from_other_var(&native_window))?;

        self.create_surface_impl(buffer_count, display_id, layer_id, 1280, 720, color_fmt, pixel_fmt, layout, Self::stray_layer_destroy, native_window)
    }

    fn set_layer_z_impl(display_id: vi::DisplayId, layer_id: vi::LayerId, z: LayerZ, system_display_service: mem::Shared<dyn vi::ISystemDisplayService>) -> Result<()> {
        let z_value = match z {
            LayerZ::Max => system_display_service.get().get_z_order_count_max(display_id)?,
            LayerZ::Min => system_display_service.get().get_z_order_count_min(display_id)?,
            LayerZ::Value(z_val) => z_val
        };
        system_display_service.get().set_layer_z(layer_id, z_value)
    }

    fn set_layer_size_impl(layer_id: vi::LayerId, width: u32, height: u32, system_display_service: mem::Shared<dyn vi::ISystemDisplayService>) -> Result<()> {
        system_display_service.get().set_layer_size(layer_id, (width as f32 * SIZE_FACTOR) as u64, (height as f32 * SIZE_FACTOR) as u64)
    }

    fn set_layer_position_impl(layer_id: vi::LayerId, x: f32, y: f32, system_display_service: mem::Shared<dyn vi::ISystemDisplayService>) -> Result<()> {
        system_display_service.get().set_layer_position(x * SIZE_FACTOR, y * SIZE_FACTOR, layer_id)
    }

    pub fn create_managed_layer_surface(&mut self, display_name: &str, aruid: applet::AppletResourceUserId, layer_flags: vi::LayerFlags, x: f32, y: f32, width: u32, height: u32, z: LayerZ, buffer_count: u32, color_fmt: ColorFormat, pixel_fmt: PixelFormat, layout: Layout) -> Result<surface::Surface> {
        let display_name_v = vi::DisplayName::from_str(display_name);
        let display_id = self.application_display_service.get().open_display(display_name_v)?;
        let system_display_service = self.application_display_service.get().get_system_display_service()?;
        let manager_display_service = self.application_display_service.get().get_manager_display_service()?;
        let native_window = parcel::ParcelPayload::new();

        let layer_id = manager_display_service.get().create_managed_layer(layer_flags, display_id, aruid)?;
        self.application_display_service.get().open_layer(display_name_v, layer_id, sf::ProcessId::from(aruid), sf::Buffer::from_other_var(&native_window))?;
        Self::set_layer_position_impl(layer_id, x, y, system_display_service.clone())?;
        Self::set_layer_size_impl(layer_id, width, height, system_display_service.clone())?;
        Self::set_layer_z_impl(display_id, layer_id, z, system_display_service)?;

        self.create_surface_impl(buffer_count, display_id, layer_id, width, height, color_fmt, pixel_fmt, layout, Self::managed_layer_destroy, native_window)
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let _ = self.vi_service; // Avoid "dead code" warnings
        let _ = self.nvdrv_service.get().close(self.nvhost_fd);
        let _ = self.nvdrv_service.get().close(self.nvmap_fd);
        let _ = self.nvdrv_service.get().close(self.nvhostctrl_fd);

        self.transfer_mem.release();
        let _ = svc::close_handle(self.transfer_mem_handle);
    }
}