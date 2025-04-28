use crate::ipc::sf;
use crate::version;
use crate::{result::*, util};

use nx_derive::{Request, Response};

define_bit_enum! {
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

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DeviceFilter {
    pub flags: DeviceFilterFlags,
    pub vendor_id: u16,
    pub product_id: u16,
    pub device_min_bcd: u16,
    pub device_max_bcd: u16,
    pub device_class: super::ClassCode,
    pub device_subclass: u8,
    pub device_protocol: u8,
    pub interface_class: super::ClassCode,
    pub interface_subclass: u8,
    pub interface_protocol: u8,
}
const_assert!(core::mem::size_of::<DeviceFilter>() == 0x10);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum InterfaceAvailableEventId {
    Unk0 = 0,
    Unk1 = 1,
    Unk2 = 2,
}
#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C, packed)]
pub struct InterfaceProfile {
    pub id: u32,
    pub device_id: u32,
    pub unk: [u8; 0x4],
    pub interface_descriptor: super::InterfaceDescriptor,
    pub pad_1: [u8; 0x7],
    pub output_endpoint_descriptors: [super::EndPointDescriptor; 15],
    pub pad_2: [u8; 0x7],
    pub input_endpoint_descriptors: [super::EndPointDescriptor; 15],
    pub pad_3: [u8; 0x6],
    pub output_ss_endpoint_companion_descriptors: [super::SsEndPointCompanionDescriptor; 15],
    pub pad_4: [u8; 0x6],
    pub input_ss_endpoint_companion_descriptors: [super::SsEndPointCompanionDescriptor; 15],
    pub pad_5: [u8; 0x3],
}
const_assert!(core::mem::size_of::<InterfaceProfile>() == 0x1B8);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InterfaceInfo {
    pub unk_str: util::ArrayString<0x40>,
    pub bus_id: u32,
    pub device_id: u32,
    pub device_descriptor: super::DeviceDescriptor,
    pub config_descriptor: super::ConfigDescriptor,
    pub pad: [u8; 0x5],
    pub unk_maybe_timestamp: u64,
}
const_assert!(core::mem::size_of::<InterfaceInfo>() == 0x70);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct InterfaceQueryOutput {
    pub profile: InterfaceProfile,
    pub info: InterfaceInfo,
}
const_assert!(core::mem::size_of::<InterfaceQueryOutput>() == 0x228);

#[derive(Request, Response, Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct XferReport {
    pub xfer_id: u32,
    pub result: ResultCode,
    pub requested_size: u32,
    pub transferred_size: u32,
    pub unk: [u8; 8],
}
const_assert!(core::mem::size_of::<XferReport>() == 0x18);

ipc_sf_define_default_client_for_interface!(ClientEpSession);
ipc_sf_define_interface_trait! {
    trait ClientEpSession {
        submit_out_request [0, version::VersionInterval::to(version::Version::new(1,0,0))]: (size: u32, unk: u32, buf: sf::InMapAliasBuffer<u8>) =>  (transferred_size: u32) (transferred_size: u32);
        re_open [0, version::VersionInterval::from(version::Version::new(2,0,0))]: () => () ();
        submit_in_request [1, version::VersionInterval::to(version::Version::new(1,0,0))]: (size: u32, unk: u32, out_buf: sf::OutMapAliasBuffer<u8>) =>  (transferred_size: u32) (transferred_size: u32);
        close [1, version::VersionInterval::from(version::Version::new(2,0,0))]: () => () ();
        reset [2, version::VersionInterval::to(version::Version::new(1,0,0))]: () => () ();
        get_completion_event [2, version::VersionInterval::from(version::Version::new(2,0,0))]: () => (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        close_deprecated [3, version::VersionInterval::to(version::Version::new(1,0,0))]: () => () ();
        populate_ring [3, version::VersionInterval::from(version::Version::new(2,0,0))]: () => () ();
        post_buffer_async [4, version::VersionInterval::from(version::Version::new(2,0,0))]: (size: u32, buf_addr: u64, unk: u64) =>  (xfer_id: u32) (xfer_id: u32);
        get_xfer_report_deprecated [5, version::VersionInterval::from_to(version::Version::new(2,0,0), version::Version::new(2,3,0))]: (count: u32, out_reports_buf: sf::OutMapAliasBuffer<XferReport>) =>  (got_count: u32) (got_count: u32);
        get_xfer_report [5, version::VersionInterval::from(version::Version::new(3,0,0))]: (count: u32, out_reports_buf: sf::OutAutoSelectBuffer<XferReport>) =>  (got_count: u32) (got_count: u32);
        batch_buffer_async_deprecated [6, version::VersionInterval::from_to(version::Version::new(2,0,0), version::Version::new(2,3,0))]: (urb_count: u32, unk_1: u32, unk_2: u32, buf_addr: u64, unk_3: u64, urb_sizes_buf: sf::InMapAliasBuffer<u32>) =>  (xfer_id: u32) (xfer_id: u32);
        batch_buffer_async [6, version::VersionInterval::from(version::Version::new(3,0,0))]: (urb_count: u32, unk_1: u32, unk_2: u32, buf_addr: u64, unk_3: u64, urb_sizes_buf: sf::InAutoSelectBuffer<u32>) =>  (xfer_id: u32) (xfer_id: u32);
        create_smmu_space [7, version::VersionInterval::from(version::Version::new(4,0,0))]: (unk: [u8; 0x10]) =>  () ();
        share_report_ring [8, version::VersionInterval::from(version::Version::new(4,0,0))]: (unk: [u8; 0x4], unk_handle: sf::CopyHandle) =>  () ();
    }
}

ipc_sf_define_default_client_for_interface!(ClientIfSession);
ipc_sf_define_interface_trait! {
    trait ClientIfSession {
        get_state_change_event [0, version::VersionInterval::all()]: () => (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        set_interface [1, version::VersionInterval::all()]: (unk: u8, profile_buf: sf::InMapAliasBuffer<InterfaceProfile>) =>  () ();
        get_interface [2, version::VersionInterval::all()]: (out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) =>  () ();
        get_alternate_interface [3, version::VersionInterval::all()]: (unk: u8, out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) =>  () ();
        get_current_frame_deprecated [5, version::VersionInterval::to(version::Version::new(1,0,0))]: () => (cur_frame: u32) (cur_frame: u32);
        get_current_frame [4, version::VersionInterval::from(version::Version::new(2,0,0))]: () => (cur_frame: u32) (cur_frame: u32);
        ctrl_xfer_async [5, version::VersionInterval::from(version::Version::new(2,0,0))]: (request_type: u8, request: u8, val: u16, idx: u16, length: u16, buf_addr: u64) =>  () ();
        submit_control_in_request [6, version::VersionInterval::to(version::Version::new(1,0,0))]: (request: u8, request_type: u8, val: u16, idx: u16, length: u16, timeout_ms: u32, out_buf: sf::OutMapAliasBuffer<u8>) =>  (transferred_size: u32) (transferred_size: u32);
        get_ctrl_xfer_completion_event [6, version::VersionInterval::from(version::Version::new(2,0,0))]: () => (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        submit_control_out_request [7, version::VersionInterval::to(version::Version::new(1,0,0))]: (request: u8, request_type: u8, val: u16, idx: u16, length: u16, timeout_ms: u32, buf: sf::InMapAliasBuffer<u8>) =>  (transferred_size: u32) (transferred_size: u32);
        get_ctrl_xfer_report [7, version::VersionInterval::from(version::Version::new(2,0,0))]: (out_report_buf: sf::OutMapAliasBuffer<XferReport>) =>  () ();
        reset_device [8, version::VersionInterval::all()]: (unk: u32) =>  () ();
        open_usb_ep_deprecated [4, version::VersionInterval::to(version::Version::new(1,0,0))]: (max_urb_count: u16, ep_type: u32, ep_number: u32, ep_direction: u32, max_xfer_size: u32) =>  (ep_desc: super::EndPointDescriptor, ep_session: ClientEpSession) (ep_desc: super::EndPointDescriptor, ep_session: ClientEpSession);
        open_usb_ep [9, version::VersionInterval::from(version::Version::new(2,0,0))]: (max_urb_count: u16, ep_type: u32, ep_number: u32, ep_direction: u32, max_xfer_size: u32) =>  (ep_desc: super::EndPointDescriptor, ep_session: ClientEpSession) (ep_desc: super::EndPointDescriptor, ep_session: ClientEpSession);
    }
}

ipc_sf_define_default_client_for_interface!(ClientRootSession);
ipc_sf_define_interface_trait! {
    trait ClientRootSession {
        bind_client_process [0, version::VersionInterval::from(version::Version::new(2,0,0))]: (self_process_handle: sf::CopyHandle) =>  () ();
        query_all_interfaces_deprecated [0, version::VersionInterval::to(version::Version::new(1,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) =>  (count: u32) (count: u32);
        query_all_interfaces [1, version::VersionInterval::from(version::Version::new(2,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) =>  (count: u32) (count: u32);
        query_available_interfaces_deprecated [1, version::VersionInterval::to(version::Version::new(1,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) =>  (count: u32) (count: u32);
        query_available_interfaces [2, version::VersionInterval::from(version::Version::new(2,0,0))]: (filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) =>  (count: u32) (count: u32);
        query_acquired_interfaces_deprecated [2, version::VersionInterval::to(version::Version::new(1,0,0))]: (out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) =>  (count: u32) (count: u32);
        query_acquired_interfaces [3, version::VersionInterval::from(version::Version::new(2,0,0))]: (out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) =>  (count: u32) (count: u32);
        create_interface_available_event_deprecated [3, version::VersionInterval::to(version::Version::new(1,0,0))]: (event_id: InterfaceAvailableEventId, filter: DeviceFilter) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        create_interface_available_event [4, version::VersionInterval::from(version::Version::new(2,0,0))]: (event_id: InterfaceAvailableEventId, filter: DeviceFilter) =>  (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        destroy_interface_available_event_deprecated [4, version::VersionInterval::to(version::Version::new(1,0,0))]: (event_id: InterfaceAvailableEventId) =>  () ();
        destroy_interface_available_event [5, version::VersionInterval::from(version::Version::new(2,0,0))]: (event_id: InterfaceAvailableEventId) =>  () ();
        get_interface_state_change_event_deprecated [5, version::VersionInterval::to(version::Version::new(1,0,0))]: () => (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        get_interface_state_change_event [6, version::VersionInterval::from(version::Version::new(2,0,0))]: () => (event_handle: sf::CopyHandle) (event_handle: sf::CopyHandle);
        acquire_usb_if_deprecated [6, version::VersionInterval::to(version::Version::new(1,0,0))]: (id: u32, out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) =>  (if_session: ClientIfSession) (if_session: ClientIfSession);
        acquire_usb_if [7, version::VersionInterval::from(version::Version::new(2,0,0))]: (id: u32, out_info_buf: sf::OutMapAliasBuffer<InterfaceInfo>, out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) =>  (if_session: ClientIfSession) (if_session: ClientIfSession);
        get_descriptor_string [7, version::VersionInterval::to(version::Version::new(1,0,0))]: (unk_1: u8, unk_2: bool, unk_maybe_id: u32, out_desc_buf: sf::OutMapAliasBuffer<u8>) =>  (unk_maybe_desc_len: u32) (unk_maybe_desc_len: u32);
        reset_device [8, version::VersionInterval::to(version::Version::new(1,0,0))]: (unk: u32) =>  () ();
    }
}
