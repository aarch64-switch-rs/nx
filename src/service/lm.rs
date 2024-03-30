use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::lm::*;

ipc_client_define_object_default!(Logger);

impl ILogger for Logger {
    fn log(&mut self, log_buf: sf::InAutoSelectBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (log_buf) => ())
    }

    fn set_destination(&mut self, log_destination: LogDestination) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (log_destination) => ())
    }
}

ipc_client_define_object_default!(LogService);

impl ILogService for LogService {
    fn open_logger(&mut self, process_id: sf::ProcessId) -> Result<mem::Shared<dyn ILogger>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id) => (logger: mem::Shared<Logger>))
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