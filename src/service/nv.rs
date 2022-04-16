use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::ipc::client;
use crate::service;

pub use crate::ipc::sf::nv::*;

// NvDrvService is the base trait for all the different services, since the only difference is their service names
pub trait NvDrvService: client::IClientObject {}

impl<S: NvDrvService> INvDrvServices for S {
    fn open(&mut self, path: sf::InMapAliasBuffer<u8>) -> Result<(Fd, ErrorCode)> {
        ipc_client_send_request_command!([self.get_info(); 0] (path) => (fd: Fd, error_code: ErrorCode))
    }

    fn ioctl(&mut self, fd: Fd, id: IoctlId, in_buf: sf::InAutoSelectBuffer<u8>, out_buf: sf::OutAutoSelectBuffer<u8>) -> Result<ErrorCode> {
        ipc_client_send_request_command!([self.get_info(); 1] (fd, id, in_buf, out_buf) => (error_code: ErrorCode))
    }

    fn close(&mut self, fd: Fd) -> Result<ErrorCode> {
        ipc_client_send_request_command!([self.get_info(); 2] (fd) => (error_code: ErrorCode))
    }

    fn initialize(&mut self, transfer_mem_size: u32, self_process_handle: sf::CopyHandle, transfer_mem_handle: sf::CopyHandle) -> Result<ErrorCode> {
        ipc_client_send_request_command!([self.get_info(); 3] (transfer_mem_size, self_process_handle, transfer_mem_handle) => (error_code: ErrorCode))
    }
}

pub struct ApplicationNvDrvService {
    session: sf::Session
}

impl sf::IObject for ApplicationNvDrvService {
    ipc_sf_object_impl_default_command_metadata!();
}

impl NvDrvService for ApplicationNvDrvService {}

impl client::IClientObject for ApplicationNvDrvService {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for ApplicationNvDrvService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nvdrv")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct AppletNvDrvService {
    session: sf::Session
}

impl sf::IObject for AppletNvDrvService {
    ipc_sf_object_impl_default_command_metadata!();
}

impl NvDrvService for AppletNvDrvService {}

impl client::IClientObject for AppletNvDrvService {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for AppletNvDrvService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nvdrv:a")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct SystemNvDrvService {
    session: sf::Session
}

impl sf::IObject for SystemNvDrvService {
    ipc_sf_object_impl_default_command_metadata!();
}

impl NvDrvService for SystemNvDrvService {}

impl client::IClientObject for SystemNvDrvService {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }

    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }
}

impl service::IService for SystemNvDrvService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("nvdrv:s")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}