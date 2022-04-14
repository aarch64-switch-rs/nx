use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;
use crate::mem;
use crate::ipc::sf::usb;

pub use crate::ipc::sf::usb::hs::*;

pub struct ClientEpSession {
    session: sf::Session
}

impl sf::IObject for ClientEpSession {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IClientEpSession for ClientEpSession {
    fn submit_out_request(&mut self, size: u32, unk: u32, buf: sf::InMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 0] (size, unk, buf) => (transferred_size: u32))
    }

    fn re_open(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => ())
    }

    fn submit_in_request(&mut self, size: u32, unk: u32, out_buf: sf::OutMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 1] (size, unk, out_buf) => (transferred_size: u32))
    }

    fn close(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => ())
    }

    fn reset(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] () => ())
    }

    fn get_completion_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 2] () => (event_handle: sf::CopyHandle))
    }

    fn close_deprecated(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] () => ())
    }

    fn populate_ring(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] () => ())
    }

    fn post_buffer_async(&mut self, size: u32, buf_addr: u64, unk: u64) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 4] (size, buf_addr, unk) => (xfer_id: u32))
    }

    fn get_xfer_report_deprecated(&mut self, count: u32, out_reports_buf: sf::OutMapAliasBuffer<XferReport>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 5] (count, out_reports_buf) => (read_count: u32))
    }

    fn get_xfer_report(&mut self, count: u32, out_reports_buf: sf::OutAutoSelectBuffer<XferReport>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 5] (count, out_reports_buf) => (read_count: u32))
    }

    fn batch_buffer_async_deprecated(&mut self, urb_count: u32, unk_1: u32, unk_2: u32, buf_addr: u64, unk_3: u64, urb_sizes_buf: sf::InMapAliasBuffer<u32>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 6] (urb_count, unk_1, unk_2, buf_addr, unk_3, urb_sizes_buf) => (xfer_id: u32))
    }

    fn batch_buffer_async(&mut self, urb_count: u32, unk_1: u32, unk_2: u32, buf_addr: u64, unk_3: u64, urb_sizes_buf: sf::InAutoSelectBuffer<u32>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 6] (urb_count, unk_1, unk_2, buf_addr, unk_3, urb_sizes_buf) => (xfer_id: u32))
    }

    fn create_smmu_space(&mut self, unk: [u8; 0x10]) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 7] (unk) => ())
    }

    fn share_report_ring(&mut self, unk: [u8; 0x4], unk_handle: sf::CopyHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 8] (unk, unk_handle) => ())
    }
}

impl service::IClientObject for ClientEpSession {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

pub struct ClientIfSession {
    session: sf::Session
}

impl sf::IObject for ClientIfSession {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IClientIfSession for ClientIfSession {
    fn get_state_change_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (event_handle: sf::CopyHandle))
    }

    fn set_interface(&mut self, unk: u8, profile_buf: sf::InMapAliasBuffer<InterfaceProfile>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (unk, profile_buf) => ())
    }

    fn get_interface(&mut self, out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] (out_profile_buf) => ())
    }

    fn get_alternate_interface(&mut self, unk: u8, out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (unk, out_profile_buf) => ())
    }

    fn get_current_frame_deprecated(&mut self) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 5] () => (cur_frame: u32))
    }

    fn get_current_frame(&mut self) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 4] () => (cur_frame: u32))
    }

    fn ctrl_xfer_async(&mut self, request_type: u8, request: u8, val: u16, idx: u16, length: u16, buf_addr: u64) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 5] (request_type, request, val, idx, length, buf_addr) => ())
    }

    fn submit_control_in_request(&mut self, request: u8, request_type: u8, val: u16, idx: u16, length: u16, timeout_ms: u32, out_buf: sf::OutMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 6] (request, request_type, val, idx, length, timeout_ms, out_buf) => (transferred_size: u32))
    }

    fn get_ctrl_xfer_completion_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 6] () => (event_handle: sf::CopyHandle))
    }

    fn submit_control_out_request(&mut self, request: u8, request_type: u8, val: u16, idx: u16, length: u16, timeout_ms: u32, buf: sf::InMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 7] (request, request_type, val, idx, length, timeout_ms, buf) => (transferred_size: u32))
    }

    fn get_ctrl_xfer_report(&mut self, out_report_buf: sf::OutMapAliasBuffer<XferReport>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 7] (out_report_buf) => ())
    }

    fn reset_device(&mut self, unk: u32) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 8] (unk) => ())
    }

    fn open_usb_ep_deprecated(&mut self, max_urb_count: u16, ep_type: u32, ep_number: u32, ep_direction: u32, max_xfer_size: u32) -> Result<(usb::EndPointDescriptor, mem::Shared<dyn IClientEpSession>)> {
        ipc_client_send_request_command!([self.session.object_info; 4] (max_urb_count, ep_type, ep_number, ep_direction, max_xfer_size) => (ep_desc: usb::EndPointDescriptor, ep_session: mem::Shared<ClientEpSession>))
    }

    fn open_usb_ep(&mut self, max_urb_count: u16, ep_type: u32, ep_number: u32, ep_direction: u32, max_xfer_size: u32) -> Result<(usb::EndPointDescriptor, mem::Shared<dyn IClientEpSession>)> {
        ipc_client_send_request_command!([self.session.object_info; 9] (max_urb_count, ep_type, ep_number, ep_direction, max_xfer_size) => (ep_desc: usb::EndPointDescriptor, ep_session: mem::Shared<ClientEpSession>))
    }
}

impl service::IClientObject for ClientIfSession {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

pub struct ClientRootSession {
    session: sf::Session
}

impl sf::IObject for ClientRootSession {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl IClientRootSession for ClientRootSession {
    fn bind_client_process(&mut self, self_process_handle: sf::CopyHandle) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (self_process_handle) => ())
    }

    fn query_all_interfaces_deprecated(&mut self, filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 0] (filter, out_intfs) => (count: u32))
    }

    fn query_all_interfaces(&mut self, filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 1] (filter, out_intfs) => (count: u32))
    }

    fn query_available_interfaces_deprecated(&mut self, filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 1] (filter, out_intfs) => (count: u32))
    }

    fn query_available_interfaces(&mut self, filter: DeviceFilter, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 2] (filter, out_intfs) => (count: u32))
    }

    fn query_acquired_interfaces_deprecated(&mut self, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 2] (out_intfs) => (count: u32))
    }

    fn query_acquired_interfaces(&mut self, out_intfs: sf::OutMapAliasBuffer<InterfaceQueryOutput>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 3] (out_intfs) => (count: u32))
    }

    fn create_interface_available_event_deprecated(&mut self, event_id: InterfaceAvailableEventId, filter: DeviceFilter) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 3] (event_id, filter) => (event_handle: sf::CopyHandle))
    }

    fn create_interface_available_event(&mut self, event_id: InterfaceAvailableEventId, filter: DeviceFilter) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 4] (event_id, filter) => (event_handle: sf::CopyHandle))
    }

    fn destroy_interface_available_event_deprecated(&mut self, event_id: InterfaceAvailableEventId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (event_id) => ())
    }

    fn destroy_interface_available_event(&mut self, event_id: InterfaceAvailableEventId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 5] (event_id) => ())
    }

    fn get_interface_state_change_event_deprecated(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 5] () => (event_handle: sf::CopyHandle))
    }

    fn get_interface_state_change_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 6] () => (event_handle: sf::CopyHandle))
    }

    fn acquire_usb_if_deprecated(&mut self, id: u32, out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) -> Result<mem::Shared<dyn IClientIfSession>> {
        ipc_client_send_request_command!([self.session.object_info; 6] (id, out_profile_buf) => (if_session: mem::Shared<ClientIfSession>))
    }

    fn acquire_usb_if(&mut self, id: u32, out_info_buf: sf::OutMapAliasBuffer<InterfaceInfo>, out_profile_buf: sf::OutMapAliasBuffer<InterfaceProfile>) -> Result<mem::Shared<dyn IClientIfSession>> {
        ipc_client_send_request_command!([self.session.object_info; 7] (id, out_info_buf, out_profile_buf) => (if_session: mem::Shared<ClientIfSession>))
    }

    fn get_descriptor_string(&mut self, unk_1: u8, unk_2: bool, unk_maybe_id: u32, out_desc_buf: sf::OutMapAliasBuffer<u8>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 7] (unk_1, unk_2, unk_maybe_id, out_desc_buf) => (unk_maybe_desc_len: u32))
    }

    fn reset_device(&mut self, unk: u32) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 8] (unk) => ())
    }
}

impl service::IClientObject for ClientRootSession {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl service::IService for ClientRootSession {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("usb:hs")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}