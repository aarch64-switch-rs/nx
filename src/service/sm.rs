use crate::result::*;
use crate::ipc;
use crate::ipc::sf;
use crate::service;
use crate::version;

pub use crate::ipc::sf::sm::*;

ipc_client_define_object_default!(UserInterface);

impl IUserInterface for UserInterface {
    fn register_client(&mut self, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (process_id) => ())
    }

    fn get_service_handle(&mut self, name: ServiceName) -> Result<sf::MoveHandle> {
        ipc_client_send_request_command!([self.session.object_info; 1] (name) => (service_handle: sf::MoveHandle))
    }

    fn register_service(&mut self, name: ServiceName, is_light: bool, max_sessions: i32) -> Result<sf::MoveHandle> {
        match self.session.object_info.protocol {
            ipc::CommandProtocol::Cmif => ipc_client_send_request_command!([self.session.object_info; 2] (name, is_light, max_sessions) => (port_handle: sf::MoveHandle)),
            ipc::CommandProtocol::Tipc => ipc_client_send_request_command!([self.session.object_info; 2] (name, max_sessions, is_light) => (port_handle: sf::MoveHandle))
        }
    }

    fn unregister_service(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (name) => ())
    }

    fn detach_client(&mut self, process_id: sf::ProcessId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 4] (process_id) => ())
    }

    fn atmosphere_install_mitm(&mut self, name: ServiceName) -> Result<(sf::MoveHandle, sf::MoveHandle)> {
        ipc_client_send_request_command!([self.session.object_info; 65000] (name) => (port_handle: sf::MoveHandle, query_handle: sf::MoveHandle))
    }

    fn atmosphere_uninstall_mitm(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 65001] (name) => ())
    }
    
    fn atmosphere_acknowledge_mitm_session(&mut self, name: ServiceName) -> Result<(mitm::MitmProcessInfo, sf::MoveHandle)> {
        ipc_client_send_request_command!([self.session.object_info; 65003] (name) => (info: mitm::MitmProcessInfo, session_handle: sf::MoveHandle))
    }

    fn atmosphere_has_mitm(&mut self, name: ServiceName) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 65004] (name) => (has: bool))
    }

    fn atmosphere_wait_mitm(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 65005] (name) => ())
    }

    fn atmosphere_declare_future_mitm(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 65006] (name) => ())
    }

    fn atmosphere_clear_future_mitm(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 65007] (name) => ())
    }

    fn atmosphere_has_service(&mut self, name: ServiceName) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 65100] (name) => (has: bool))
    }

    fn atmosphere_wait_service(&mut self, name: ServiceName) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 65101] (name) => ())
    }
}

impl service::INamedPort for UserInterface {
    fn get_name() -> &'static str {
        nul!("sm:")
    }

    fn post_initialize(&mut self) -> Result<()> {
        if version::get_version() >= version::Version::new(12,0,0) {
            self.session.object_info.protocol = ipc::CommandProtocol::Tipc;
        }

        self.register_client(sf::ProcessId::new())
    }
}