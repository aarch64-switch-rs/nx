use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::mii::*;

ipc_client_define_object_default!(DatabaseService);

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

    fn build_random(&mut self, age: sf::EnumAsPrimitiveType<Age, u32>, gender: sf::EnumAsPrimitiveType<Gender, u32>, race: sf::EnumAsPrimitiveType<FaceColor, u32>) -> Result<CharInfo> {
        ipc_client_send_request_command!([self.session.object_info; 6] (age, gender, race) => (char_info: CharInfo))
    }
}

ipc_client_define_object_default!(StaticService);

impl IStaticService for StaticService {
    fn get_database_service(&mut self, key_code: SpecialKeyCode) -> Result<mem::Shared<dyn IDatabaseService>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (key_code) => (database_service: mem::Shared<DatabaseService>))
    }
}

impl service::IService for StaticService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("mii:e")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}