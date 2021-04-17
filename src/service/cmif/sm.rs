use crate::result::*;
use crate::ipc::cmif::sf;
use crate::service;

pub use crate::ipc::cmif::sf::sm::*;

pub struct UserInterface {
    session: sf::Session
}

impl sf::IObject for UserInterface {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            nipc_cmif_interface_make_command_meta!(register_client: 0),
            nipc_cmif_interface_make_command_meta!(get_service_handle: 1),
            nipc_cmif_interface_make_command_meta!(register_service: 2),
            nipc_cmif_interface_make_command_meta!(unregister_service: 3),
            nipc_cmif_interface_make_command_meta!(detach_client: 4),
            nipc_cmif_interface_make_command_meta!(atmosphere_install_mitm: 65000),
            nipc_cmif_interface_make_command_meta!(atmosphere_uninstall_mitm: 65001),
            nipc_cmif_interface_make_command_meta!(atmosphere_acknowledge_mitm_session: 65003),
            nipc_cmif_interface_make_command_meta!(atmosphere_has_mitm: 65004),
            nipc_cmif_interface_make_command_meta!(atmosphere_wait_mitm: 64005),
            nipc_cmif_interface_make_command_meta!(atmosphere_declare_future_mitm: 65006),
            nipc_cmif_interface_make_command_meta!(atmosphere_clear_future_mitm: 65007),
            nipc_cmif_interface_make_command_meta!(atmosphere_has_service: 65100),
            nipc_cmif_interface_make_command_meta!(atmosphere_wait_service: 65101)
        ]
    }
}

impl service::cmif::IClientObject for UserInterface {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IUserInterface for UserInterface {
    fn register_client(&mut self, process_id: sf::ProcessId) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 0] (process_id) => ())
    }

    fn get_service_handle(&mut self, name: ServiceName) -> Result<sf::MoveHandle> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 1] (name) => (service_handle: sf::MoveHandle))
    }

    fn register_service(&mut self, name: ServiceName, max_sessions: i32, is_light: bool) -> Result<sf::MoveHandle> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 2] (name, max_sessions, is_light) => (port_handle: sf::MoveHandle))
    }

    fn unregister_service(&mut self, name: ServiceName) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 3] (name) => ())
    }

    fn detach_client(&mut self, process_id: sf::ProcessId) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 4] (process_id) => ())
    }

    fn atmosphere_install_mitm(&mut self, name: ServiceName) -> Result<(sf::MoveHandle, sf::MoveHandle)> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65000] (name) => (port_handle: sf::MoveHandle, query_handle: sf::MoveHandle))
    }

    fn atmosphere_uninstall_mitm(&mut self, name: ServiceName) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65001] (name) => ())
    }
    
    fn atmosphere_acknowledge_mitm_session(&mut self, name: ServiceName) -> Result<(MitmProcessInfo, sf::MoveHandle)> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65003] (name) => (info: MitmProcessInfo, session_handle: sf::MoveHandle))
    }

    fn atmosphere_has_mitm(&mut self, name: ServiceName) -> Result<bool> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65004] (name) => (has: bool))
    }

    fn atmosphere_wait_mitm(&mut self, name: ServiceName) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65005] (name) => ())
    }

    fn atmosphere_declare_future_mitm(&mut self, name: ServiceName) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65006] (name) => ())
    }

    fn atmosphere_clear_future_mitm(&mut self, name: ServiceName) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65007] (name) => ())
    }

    fn atmosphere_has_service(&mut self, name: ServiceName) -> Result<bool> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65100] (name) => (has: bool))
    }

    fn atmosphere_wait_service(&mut self, name: ServiceName) -> Result<()> {
        nipc_cmif_client_send_request_command!([self.session.object_info; 65101] (name) => ())
    }
}

impl service::cmif::INamedPort for UserInterface {
    fn get_name() -> &'static str {
        nul!("sm:")
    }

    fn post_initialize(&mut self) -> Result<()> {
        self.register_client(sf::ProcessId::new())
    }
}