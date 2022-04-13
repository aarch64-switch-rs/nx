use crate::{result::*, util};
use crate::ipc::sf;
use crate::version;

bit_enum! {
    DeviceFilterFlags (u16) {
        IdVendor = bit!(0),
        IdProduct = bit!(1),
        DeviceMin = bit!(2),
        DeviceMax = bit!(3),
        DeviceClass = bit!(4),
        DeviceSubClass = bit!(5),
        DeviceProtocol = bit!(6),
        InterfaceClass = bit!(7),
        InterfaceSubClass = bit!(8),
        InterfaceProtocol = bit!(9)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DeviceFilter {
    flags: DeviceFilterFlags,
    vendor_id: u16,
    product_id: u16,
    device_min_bcd: u16,
    device_max_bcd: u16,
    device_class: u8,
    device_subclass: u8,
    device_protocol: u8,
    interface_class: u8,
    interface_subclass: u8,
    interface_protocol: u8
}
const_assert!(core::mem::size_of::<DeviceFilter>() == 0x10);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum InterfaceAvailableEventId {
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct InterfaceProfile {
    id: u32,
    device_id: u32,
    unk: [u8; 0x4],
    interface_descriptor: super::InterfaceDescriptor,
    pad_1: [u8; 0x7],
    output_endpoint_descriptors: [super::EndPointDescriptor; 15],
    pad_2: [u8; 0x7],
    input_endpoint_descriptors: [super::EndPointDescriptor; 15],
    pad_3: [u8; 0x6],
    output_ss_endpoint_companion_descriptors: [super::SsEndPointCompanionDescriptor; 15],
    pad_4: [u8; 0x6],
    input_ss_endpoint_companion_descriptors: [super::SsEndPointCompanionDescriptor; 15],
    pad_5: [u8; 0x3]
}
const_assert!(core::mem::size_of::<InterfaceProfile>() == 0x1B8);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InterfaceQueryOutput {
    profile: InterfaceProfile,
    unk_str: util::CString<0x40>,
    bus_id: u32,
    device_id: u32,
    device_descriptor: super::DeviceDescriptor,
    config_descriptor: super::ConfigDescriptor,
    pad: [u8; 0x5],
    unk_maybe_timestamp: u64
}
const_assert!(core::mem::size_of::<InterfaceQueryOutput>() == 0x228);

ipc_sf_define_interface_trait! {
    trait IClientRootSession {
        bind_client_process [0, version::VersionInterval::from(version::Version::new(2,0,0))]: (self_process_handle: sf::CopyHandle) => ();
        query_all_interfaces_deprecated [0, version::VersionInterval::to(version::Version::new(1,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) => (count: u32);
        query_all_interfaces [1, version::VersionInterval::from(version::Version::new(2,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) => (count: u32);
        query_available_interfaces_deprecated [1, version::VersionInterval::to(version::Version::new(1,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) => (count: u32);
        query_available_interfaces [2, version::VersionInterval::from(version::Version::new(2,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) => (count: u32);
        query_acquired_interfaces_deprecated [2, version::VersionInterval::to(version::Version::new(1,0,0))]: (out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) => (count: u32);
        query_acquired_interfaces [3, version::VersionInterval::from(version::Version::new(2,0,0))]: (out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) => (count: u32);
        create_interface_available_event_deprecated [3, version::VersionInterval::to(version::Version::new(1,0,0))]: (event_id: InterfaceAvailableEventId, filter: DeviceFilter) => (event_handle: sf::CopyHandle);
        create_interface_available_event [4, version::VersionInterval::from(version::Version::new(2,0,0))]: (event_id: InterfaceAvailableEventId, filter: DeviceFilter) => (event_handle: sf::CopyHandle);
        destroy_interface_available_event_deprecated [4, version::VersionInterval::to(version::Version::new(1,0,0))]: (event_id: InterfaceAvailableEventId) => ();
        destroy_interface_available_event [5, version::VersionInterval::from(version::Version::new(2,0,0))]: (event_id: InterfaceAvailableEventId) => ();
        get_interface_state_change_event_deprecated [5, version::VersionInterval::to(version::Version::new(1,0,0))]: () => (event_handle: sf::CopyHandle);
        get_interface_state_change_event [6, version::VersionInterval::from(version::Version::new(2,0,0))]: () => (event_handle: sf::CopyHandle);
    }
}