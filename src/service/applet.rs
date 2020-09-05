use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub use crate::ipc::sf::applet::*;

pub struct StorageAccessor {
    session: sf::Session
}

impl sf::IObject for StorageAccessor {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_size: 0,
            write: 10,
            read: 11
        }
    }
}

impl service::IClientObject for StorageAccessor {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IStorageAccessor for StorageAccessor {
    fn get_size(&mut self) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (size: usize))
    }

    fn write(&mut self, offset: usize, buf: sf::InAutoSelectBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (offset, buf) => ())
    }

    fn read(&mut self, offset: usize, buf: sf::OutAutoSelectBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 11] (offset, buf) => ())
    }
}

pub struct Storage {
    session: sf::Session
}

impl sf::IObject for Storage {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            open: 0
        }
    }
}

impl service::IClientObject for Storage {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IStorage for Storage {
    fn open(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (storage_accessor: mem::Shared<StorageAccessor>))
    }
}

pub struct LibraryAppletAccessor {
    session: sf::Session
}

impl sf::IObject for LibraryAppletAccessor {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_applet_state_changed_event: 0,
            start: 10,
            push_in_data: 100
        }
    }
}

impl service::IClientObject for LibraryAppletAccessor {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl ILibraryAppletAccessor for LibraryAppletAccessor {
    fn get_applet_state_changed_event(&mut self) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 0] () => (applet_state_changed_event: sf::CopyHandle))
    }

    fn start(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] () => ())
    }

    fn push_in_data(&mut self, storage: mem::Shared<dyn sf::IObject>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 100] (storage) => ())
    }
}

pub struct LibraryAppletCreator {
    session: sf::Session
}

impl sf::IObject for LibraryAppletCreator {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            create_library_applet: 0,
            create_storage: 10
        }
    }
}

impl service::IClientObject for LibraryAppletCreator {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl ILibraryAppletCreator for LibraryAppletCreator {
    fn create_library_applet(&mut self, id: AppletId, mode: LibraryAppletMode) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (id, mode) => (library_applet_accessor: mem::Shared<LibraryAppletAccessor>))
    }

    fn create_storage(&mut self, size: usize) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 10] (size) => (storage: mem::Shared<Storage>))
    }
}

pub struct WindowController {
    session: sf::Session
}

impl sf::IObject for WindowController {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            acquire_foreground_rights: 10
        }
    }
}

impl service::IClientObject for WindowController {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IWindowController for WindowController {
    fn acquire_foreground_rights(&mut self) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] () => ())
    }
}

pub struct SelfController {
    session: sf::Session
}

impl sf::IObject for SelfController {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            set_screenshot_permission: 10
        }
    }
}

impl service::IClientObject for SelfController {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl ISelfController for SelfController {
    fn set_screenshot_permission(&mut self, permission: ScreenShotPermission) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 10] (permission) => ())
    }
}

pub struct LibraryAppletProxy {
    session: sf::Session
}

impl sf::IObject for LibraryAppletProxy {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_self_controller: 1,
            get_window_controller: 2,
            get_library_applet_creator: 11
        }
    }
}

impl service::IClientObject for LibraryAppletProxy {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl ILibraryAppletProxy for LibraryAppletProxy {
    fn get_self_controller(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 1] () => (self_controller: mem::Shared<SelfController>))
    }

    fn get_window_controller(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 2] () => (window_controller: mem::Shared<WindowController>))
    }

    fn get_library_applet_creator(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 11] () => (library_applet_creator: mem::Shared<LibraryAppletCreator>))
    }
}

pub struct AllSystemAppletProxiesService {
    session: sf::Session
}

impl sf::IObject for AllSystemAppletProxiesService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            open_library_applet_proxy: 201
        }
    }
}

impl service::IClientObject for AllSystemAppletProxiesService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IAllSystemAppletProxiesService for AllSystemAppletProxiesService {
    fn open_library_applet_proxy(&mut self, process_id: sf::ProcessId, self_process_handle: sf::CopyHandle, applet_attribute: sf::InMapAliasBuffer) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 201] (process_id, self_process_handle, applet_attribute) => (library_applet_proxy: mem::Shared<LibraryAppletProxy>))
    }
}

impl service::IService for AllSystemAppletProxiesService {
    fn get_name() -> &'static str {
        nul!("appletAE")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}