use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::ldr::*;

ipc_sf_client_object_define_default_impl!(ShellInterface);

impl IShellInterface for ShellInterface {
    fn set_program_argument_deprecated(&mut self, program_id: u64, args_size: u32, args_buf: sf::InPointerBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (program_id, args_size, args_buf) => ())
    }

    fn set_program_argument(&mut self, program_id: u64, args_buf: sf::InPointerBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (program_id, args_buf) => ())
    }

    fn flush_arguments(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => ())
    }

    fn atmosphere_register_external_code(&mut self, program_id: u64) -> Result<sf::MoveHandle> {
        ipc_client_send_request_command!([self.session.object_info; 65000] (program_id) => (session_handle: sf::MoveHandle))
    }

    fn atmosphere_unregister_external_code(&mut self, program_id: u64) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 65001] (program_id) => ())
    }
}

impl service::IService for ShellInterface {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("ldr:shel")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}