use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::applet::*;

ipc_client_define_object_default!(StorageAccessor);

impl IStorageAccessor for StorageAccessor {
    fn get_size(&mut self) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (size: usize))
    }

    fn write(&mut self, offset: usize, buf: sf::InAutoSelectBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (offset, buf) => ())
    }

    fn read(&mut self, offset: usize, buf: sf::OutAutoSelectBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 11] (offset, buf) => ())
    }
}

ipc_client_define_object_default!(Storage);

impl IStorage for Storage {
    fn open(&mut self) -> Result<mem::Shared<dyn IStorageAccessor>> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (storage_accessor: mem::Shared<StorageAccessor>))
    }
}

ipc_client_define_object_default!(LibraryAppletAccessor);

impl ILibraryAppletAccessor for LibraryAppletAccessor {
    fn get_applet_state_changed_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (applet_state_changed_event: sf::CopyHandle))
    }

    fn start(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] () => ())
    }

    fn push_in_data(&mut self, storage: mem::Shared<dyn IStorage>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 100] (storage) => ())
    }

    fn pop_out_data(&mut self) -> Result<mem::Shared<dyn IStorage>> {
        ipc_client_send_request_command!([self.session.object_info; 101] () => (storage: mem::Shared<Storage>))
    }
}

ipc_client_define_object_default!(LibraryAppletCreator);

impl ILibraryAppletCreator for LibraryAppletCreator {
    fn create_library_applet(&mut self, id: AppletId, mode: LibraryAppletMode) -> Result<mem::Shared<dyn ILibraryAppletAccessor>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (id, mode) => (library_applet_accessor: mem::Shared<LibraryAppletAccessor>))
    }

    fn create_storage(&mut self, size: usize) -> Result<mem::Shared<dyn IStorage>> {
        ipc_client_send_request_command!([self.session.object_info; 10] (size) => (storage: mem::Shared<Storage>))
    }
}

ipc_client_define_object_default!(WindowController);

impl IWindowController for WindowController {
    fn acquire_foreground_rights(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] () => ())
    }
}

ipc_client_define_object_default!(SelfController);

impl ISelfController for SelfController {
    fn set_screenshot_permission(&mut self, permission: ScreenShotPermission) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (permission) => ())
    }
}

ipc_client_define_object_default!(LibraryAppletProxy);

impl ILibraryAppletProxy for LibraryAppletProxy {
    fn get_self_controller(&mut self) -> Result<mem::Shared<dyn ISelfController>> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => (self_controller: mem::Shared<SelfController>))
    }

    fn get_window_controller(&mut self) -> Result<mem::Shared<dyn IWindowController>> {
        ipc_client_send_request_command!([self.session.object_info; 2] () => (window_controller: mem::Shared<WindowController>))
    }

    fn get_library_applet_creator(&mut self) -> Result<mem::Shared<dyn ILibraryAppletCreator>> {
        ipc_client_send_request_command!([self.session.object_info; 11] () => (library_applet_creator: mem::Shared<LibraryAppletCreator>))
    }
}

ipc_client_define_object_default!(AllSystemAppletProxiesService);

impl IAllSystemAppletProxiesService for AllSystemAppletProxiesService {
    fn open_library_applet_proxy(&mut self, process_id: sf::ProcessId, self_process_handle: sf::CopyHandle, applet_attribute: sf::InMapAliasBuffer<AppletAttribute>) -> Result<mem::Shared<dyn ILibraryAppletProxy>> {
        ipc_client_send_request_command!([self.session.object_info; 201] (process_id, self_process_handle, applet_attribute) => (library_applet_proxy: mem::Shared<LibraryAppletProxy>))
    }
}

impl service::IService for AllSystemAppletProxiesService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("appletAE")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}