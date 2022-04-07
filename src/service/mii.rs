use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::mii::*;

pub struct DatabaseService {
    session: sf::Session
}

impl sf::IObject for DatabaseService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(is_updated: 0),
            ipc_cmif_interface_make_command_meta!(is_full: 1),
            ipc_cmif_interface_make_command_meta!(get_count: 2),
            ipc_cmif_interface_make_command_meta!(get_1: 4),
            ipc_cmif_interface_make_command_meta!(build_random: 6)
        ]
    }
}

impl service::IClientObject for DatabaseService {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl IDatabaseService for DatabaseService {
    fn is_updated(&mut self, flag: SourceFlag) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 0] (flag) => (updated: bool))
    }

    fn is_full(&mut self) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => (full: bool))
    }

    fn get_count(&mut self, flag: SourceFlag) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 2] (flag) => (count: u32))
    }

    fn get_1(&mut self, flag: SourceFlag, out_char_infos: sf::OutMapAliasBuffer<CharInfo>) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 4] (flag, out_char_infos) => (count: u32))
    }

    fn build_random(&mut self, age: sf::EnumAsPrimitiveType<Age, u32>, gender: sf::EnumAsPrimitiveType<Gender, u32>, race: sf::EnumAsPrimitiveType<Race, u32>) -> Result<CharInfo> {
        ipc_client_send_request_command!([self.session.object_info; 6] (age, gender, race) => (char_info: CharInfo))
    }
}

pub struct StaticService {
    session: sf::Session
}

impl sf::IObject for StaticService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(get_database_service: 0)
        ]
    }
}

impl service::IClientObject for StaticService {
    fn new(session: sf::Session) -> Self {
        Self { session }
    }
}

impl IStaticService for StaticService {
    fn get_database_service(&mut self, key_code: SpecialKeyCode) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (key_code) => (database_service: mem::Shared<DatabaseService>))
    }
}

impl service::IService for StaticService {
    fn get_name() -> &'static str {
        nul!("mii:e")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}