use crate::result::*;
use crate::ipc::cmif::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::cmif::sf::psc::*;

pub struct PmModule {
    session: sf::Session
}

impl sf::IObject for PmModule {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(initialize: 0),
            ipc_cmif_interface_make_command_meta!(get_request: 1),
            ipc_cmif_interface_make_command_meta!(acknowledge: 2),
            ipc_cmif_interface_make_command_meta!(finalize: 3),
            ipc_cmif_interface_make_command_meta!(acknowledge_ex: 4)
        ]
    }
}

impl service::cmif::IClientObject for PmModule {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IPmModule for PmModule {
    fn initialize(&mut self, id: ModuleId, dependencies: sf::InMapAliasBuffer) -> Result<sf::CopyHandle> {
        ipc_cmif_client_send_request_command!([self.session.object_info; 0] (id, dependencies) => (event_handle: sf::CopyHandle))
    }

    fn get_request(&mut self) -> Result<(State, u32)> {
        ipc_cmif_client_send_request_command!([self.session.object_info; 1] () => (state: State, flags: u32))
    }

    fn acknowledge(&mut self) -> Result<()> {
        ipc_cmif_client_send_request_command!([self.session.object_info; 2] () => ())
    }

    fn finalize(&mut self) -> Result<()> {
        ipc_cmif_client_send_request_command!([self.session.object_info; 3] () => ())
    }

    fn acknowledge_ex(&mut self, state: State) -> Result<()> {
        ipc_cmif_client_send_request_command!([self.session.object_info; 4] (state) => ())
    }
}

pub struct PmService {
    session: sf::Session
}

impl sf::IObject for PmService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(get_pm_module: 0)
        ]
    }
}

impl service::cmif::IClientObject for PmService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IPmService for PmService {
    fn get_pm_module(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_cmif_client_send_request_command!([self.session.object_info; 0] () => (pm_module: mem::Shared<PmModule>))
    }
}

impl service::cmif::IService for PmService {
    fn get_name() -> &'static str {
        nul!("psc:m")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}