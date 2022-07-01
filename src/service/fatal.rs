use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;

pub use crate::ipc::sf::fatal::*;

ipc_client_define_object_default!(Service);

impl IService for Service {
    fn throw_fatal_with_policy(&mut self, rc: ResultCode, policy: FatalPolicy, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (rc, policy, process_id) => ())
    }
}

impl service::IService for Service {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("fatal:u")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}