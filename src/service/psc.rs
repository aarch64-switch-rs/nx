use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::psc::*;

ipc_client_define_object_default!(PmModule);

impl IPmModule for PmModule {
    fn initialize(&mut self, id: ModuleId, dependencies: sf::InMapAliasBuffer<ModuleId>) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 0] (id, dependencies) => (event_handle: sf::CopyHandle))
    }

    fn get_request(&mut self) -> Result<(State, u32)> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => (state: State, flags: u32))
    }

    fn acknowledge(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] () => ())
    }

    fn finalize(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] () => ())
    }

    fn acknowledge_ex(&mut self, state: State) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (state) => ())
    }
}

ipc_client_define_object_default!(PmService);

impl IPmService for PmService {
    fn get_pm_module(&mut self) -> Result<mem::Shared<dyn IPmModule>> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (pm_module: mem::Shared<PmModule>))
    }
}

impl service::IService for PmService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("psc:m")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}