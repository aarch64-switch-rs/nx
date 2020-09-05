use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::sm::*;

pub struct UserInterface {
    session: sf::Session
}

impl sf::IObject for UserInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            initialize: 0,
            get_service_handle: 1,
            register_service: 2,
            unregister_service: 3,
            atmosphere_install_mitm: 65000,
            atmosphere_uninstall_mitm: 65001,
            atmosphere_acknowledge_mitm_session: 65003,
            atmosphere_has_service: 65100
        }
    }
}

impl service::IClientObject for UserInterface {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IUserInterface for UserInterface {
    fn initialize(&mut self, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id) => ())
    }

    fn get_service_handle(&mut self, name: ServiceName) -> Result<sf::MoveHandle> {
        ipc_client_send_request_command!([self.session.object_info; 1] (name) => (service_handle: sf::MoveHandle))
    }

    fn register_service(&mut self, name: ServiceName, is_light: bool, max_sessions: i32) -> Result<sf::MoveHandle> {
        ipc_client_send_request_command!([self.session.object_info; 2] (name, is_light, max_sessions) => (port_handle: sf::MoveHandle))
    }

    fn unregister_service(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (name) => ())
    }

    fn atmosphere_install_mitm(&mut self, name: ServiceName) -> Result<(sf::MoveHandle, sf::MoveHandle)> {
        ipc_client_send_request_command!([self.session.object_info; 65000] (name) => (port_handle: sf::MoveHandle, query_handle: sf::MoveHandle))
    }

    fn atmosphere_uninstall_mitm(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 65001] (name) => ())
    }
    
    fn atmosphere_acknowledge_mitm_session(&mut self, name: ServiceName) -> Result<(MitmProcessInfo, sf::MoveHandle)> {
        ipc_client_send_request_command!([self.session.object_info; 65003] (name) => (info: MitmProcessInfo, session_handle: sf::MoveHandle))
    }

    fn atmosphere_has_service(&mut self, name: ServiceName) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 65100] (name) => (has: bool))
    }
}

impl service::INamedPort for UserInterface {
    fn get_name() -> &'static str {
        nul!("sm:")
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.initialize(sf::ProcessId::new())
    }
}