use super::*;
use crate::service;
use crate::ipc::sf;

pub use crate::ipc::sf::ncm::*;

ipc_client_define_object_default!(ContentMetaDatabase);

impl IContentMetaDatabase for ContentMetaDatabase {
    fn set(&mut self, meta_key: ContentMetaKey, in_rec_buf: sf::InMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (meta_key, in_rec_buf) => ())
    }

    fn get(&mut self, meta_key: ContentMetaKey, out_rec_buf: sf::OutMapAliasBuffer<u8>) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 1] (meta_key, out_rec_buf) => (size: usize))
    }

    fn remove(&mut self, meta_key: ContentMetaKey) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2] (meta_key) => ())
    }

    fn get_content_id_by_type(&mut self, meta_key: ContentMetaKey, cnt_type: ContentType) -> Result<ContentId> {
        ipc_client_send_request_command!([self.session.object_info; 3] (meta_key, cnt_type) => (id: ContentId))
    }

    fn list_content_info(&mut self, out_rec_buf: sf::OutMapAliasBuffer<ContentInfo>, meta_key: ContentMetaKey, offset: u32) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 4] (out_rec_buf, meta_key, offset) => (count: u32))
    }

    fn list(&mut self, out_meta_keys: sf::OutMapAliasBuffer<ContentMetaKey>, meta_type: ContentMetaType, program_id: ProgramId, program_id_min: ProgramId, program_id_max: ProgramId, install_type: ContentInstallType) -> Result<(u32, u32)> {
        ipc_client_send_request_command!([self.session.object_info; 5] (out_meta_keys, meta_type, program_id, program_id_min, program_id_max, install_type) => (total: u32, written: u32))
    }

    fn get_latest_content_meta_key(&mut self, program_id:ProgramId) -> Result<ContentMetaKey> {
        ipc_client_send_request_command!([self.session.object_info; 6] (program_id) => (meta_key: ContentMetaKey))
    }

    fn list_application(&mut self, out_app_meta_keys: sf::OutMapAliasBuffer<ApplicationContentMetaKey>, meta_type: ContentMetaType) -> Result<(u32, u32)> {
        ipc_client_send_request_command!([self.session.object_info; 7] (out_app_meta_keys, meta_type) => (total: u32, written: u32))
    }

    fn has(&mut self, meta_key: ContentMetaKey) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 8] (meta_key) => (has: bool))
    }

    fn has_all(&mut self, meta_keys_buf: sf::InMapAliasBuffer<ContentMetaKey>) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 9] (meta_keys_buf) => (has: bool))
    }

    fn get_size(&mut self, meta_key: ContentMetaKey) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 10] (meta_key) => (size: usize))
    }

    fn get_required_system_version(&mut self, meta_key: ContentMetaKey) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 11] (meta_key) => (version: u32))
    }

    fn get_patch_content_meta_id(&mut self, meta_key: ContentMetaKey) -> Result<ProgramId> {
        ipc_client_send_request_command!([self.session.object_info; 12] (meta_key) => (patch_id: ProgramId))
    }

    fn disable_forcibly(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 13] () => ())
    }

    fn lookup_orphan_content(&mut self, content_ids_buf: sf::InMapAliasBuffer<ContentId>, out_orphaned_buf: sf::OutMapAliasBuffer<bool>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 14] (content_ids_buf, out_orphaned_buf) => ())
    }

    fn commit(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 15] () => ())
    }

    fn has_content(&mut self, meta_key: ContentMetaKey, id: ContentId) -> Result<bool> {
        ipc_client_send_request_command!([self.session.object_info; 16] (meta_key, id) => (has: bool))
    }

    fn list_content_meta_info(&mut self, out_meta_infos: sf::OutMapAliasBuffer<ContentMetaInfo>, meta_key: ContentMetaKey, offset: u32) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 17] (out_meta_infos, meta_key, offset) => (written: u32))
    }

    fn get_attributes(&mut self, meta_key: ContentMetaKey) -> Result<u8> {
        ipc_client_send_request_command!([self.session.object_info; 18] (meta_key) => (attrs: u8))
    }

    fn get_required_application_version(&mut self, meta_key: ContentMetaKey) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 19] (meta_key) => (version: u32))
    }

    fn get_content_id_by_type_and_offset(&mut self, meta_key: ContentMetaKey, cnt_type: ContentType, id_offset: u8) -> Result<ContentId> {
        ipc_client_send_request_command!([self.session.object_info; 20] (meta_key, cnt_type, id_offset) => (id: ContentId))
    }

    fn get_count(&mut self) -> Result<u32> {
        ipc_client_send_request_command!([self.session.object_info; 21] () => (count: u32))
    }

    fn get_owner_application_id(&mut self, meta_key: ContentMetaKey) -> Result<ApplicationId> {
        ipc_client_send_request_command!([self.session.object_info; 22] (meta_key) => (app_id: ApplicationId))
    }
}

ipc_client_define_object_default!(ContentManager);

impl IContentManager for ContentManager {
    fn open_content_meta_database(&mut self, storage_id: StorageId) -> Result<mem::Shared<dyn IContentMetaDatabase>> {
        ipc_client_send_request_command!([self.session.object_info; 5] (storage_id) => (meta_db: mem::Shared<ContentMetaDatabase>))
    }
}

impl service::IService for ContentManager {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("ncm")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}
