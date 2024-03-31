use super::*;
use crate::service;
use self::sf::ncm;

pub use crate::ipc::sf::lr::*;

ipc_client_define_object_default!(LocationResolver);

impl ILocationResolver for LocationResolver {
    fn redirect_program_path(&mut self, program_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (program_id, path_buf) => ())
    }
}

ipc_client_define_object_default!(RegisteredLocationResolver);

impl IRegisteredLocationResolver for RegisteredLocationResolver {
    fn register_program_path_deprecated(&mut self, program_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (program_id, path_buf) => ())
    }
    
    fn register_program_path(&mut self, program_id: ncm::ProgramId, owner_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (program_id, owner_id, path_buf) => ())
    }

    fn redirect_program_path_deprecated(&mut self, program_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (program_id, path_buf) => ())
    }

    fn redirect_program_path(&mut self, program_id: ncm::ProgramId, owner_id: ncm::ProgramId, path_buf: sf::InPointerBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (program_id, owner_id, path_buf) => ())
    }
}

ipc_client_define_object_default!(LocationResolverManager);

impl ILocationResolverManager for LocationResolverManager {
    fn open_location_resolver(&mut self, storage_id: ncm::StorageId) -> Result<mem::Shared<dyn ILocationResolver>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (storage_id) => (resolver: mem::Shared<LocationResolver>))
    }

    fn open_registered_location_resolver(&mut self) -> Result<mem::Shared<dyn IRegisteredLocationResolver>> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => (resolver: mem::Shared<RegisteredLocationResolver>))
    }
}

impl service::IService for LocationResolverManager {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("lr")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
