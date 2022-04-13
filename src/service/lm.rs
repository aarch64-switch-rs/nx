use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::lm::*;

pub struct Logger {
    session: sf::Session
}

impl sf::IObject for Logger {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl ILogger for Logger {
    fn log(&mut self, log_buf: sf::InAutoSelectBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (log_buf) => ())
    }

    fn set_destination(&mut self, log_destination: LogDestination) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (log_destination) => ())
    }
}

impl service::IClientObject for Logger {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

pub struct LogService {
    session: sf::Session
}

impl sf::IObject for LogService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    ipc_sf_object_impl_default_command_metadata!();
}

impl ILogService for LogService {
    fn open_logger(&mut self, process_id: sf::ProcessId) -> Result<mem::Shared<dyn ILogger>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id) => (logger: mem::Shared<Logger>))
    }
}

impl service::IClientObject for LogService {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl service::IService for LogService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("lm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}