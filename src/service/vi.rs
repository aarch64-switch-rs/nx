use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;
use crate::service::dispdrv;
use crate::service::applet;

pub use crate::ipc::sf::vi::*;

pub struct ManagerDisplayService {
    session: sf::Session
}

impl sf::IObject for ManagerDisplayService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            create_managed_layer: 2010,
            destroy_managed_layer: 2011
        }
    }
}

impl service::IClientObject for ManagerDisplayService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IManagerDisplayService for ManagerDisplayService {
    fn create_managed_layer(&mut self, flags: LayerFlags, display_id: DisplayId, aruid: applet::AppletResourceUserId) -> Result<LayerId> {
        ipc_client_send_request_command!([self.session.object_info; 2010] (flags, display_id, aruid) => (id: LayerId))
    }

    fn destroy_managed_layer(&mut self, id: LayerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2011] (id) => ())
    }
}

pub struct SystemDisplayService {
    session: sf::Session
}

impl sf::IObject for SystemDisplayService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_z_order_count_min: 1200,
            get_z_order_count_max: 1202,
            set_layer_position: 2201,
            set_layer_size: 2203,
            set_layer_z: 2205,
            set_layer_visibility: 2207
        }
    }
}

impl service::IClientObject for SystemDisplayService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl ISystemDisplayService for SystemDisplayService {
    fn get_z_order_count_min(&mut self, display_id: DisplayId) -> Result<i64> {
        ipc_client_send_request_command!([self.session.object_info; 1200] (display_id) => (z: i64))
    }

    fn get_z_order_count_max(&mut self, display_id: DisplayId) -> Result<i64> {
        ipc_client_send_request_command!([self.session.object_info; 1202] (display_id) => (z: i64))
    }

    fn set_layer_position(&mut self, x: f32, y: f32, id: LayerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2201] (x, y, id) => ())
    }

    fn set_layer_size(&mut self, id: LayerId, width: u64, height: u64) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2203] (id, width, height) => ())
    }

    fn set_layer_z(&mut self, id: LayerId, z: i64) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2205] (id, z) => ())
    }

    fn set_layer_visibility(&mut self, visible: bool, id: LayerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2207] (visible, id) => ())
    }
}

pub struct ApplicationDisplayService {
    session: sf::Session
}

impl sf::IObject for ApplicationDisplayService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_relay_service: 100,
            get_system_display_service: 101,
            get_manager_display_service: 102,
            open_display: 1010,
            close_display: 1020,
            open_layer: 2020,
            create_stray_layer: 2030,
            destroy_stray_layer: 2031,
            get_display_vsync_event: 5202
        }
    }
}

impl service::IClientObject for ApplicationDisplayService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IApplicationDisplayService for ApplicationDisplayService {
    fn get_relay_service(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 100] () => (relay_service: mem::Shared<dispdrv::HOSBinderDriver>))
    }

    fn get_system_display_service(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 101] () => (relay_service: mem::Shared<SystemDisplayService>))
    }

    fn get_manager_display_service(&mut self) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 102] () => (relay_service: mem::Shared<ManagerDisplayService>))
    }

    fn open_display(&mut self, name: DisplayName) -> Result<DisplayId> {
        ipc_client_send_request_command!([self.session.object_info; 1010] (name) => (id: DisplayId))
    }

    fn close_display(&mut self, display_id: DisplayId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1020] (display_id) => ())
    }

    fn open_layer(&mut self, name: DisplayName, id: LayerId, aruid: sf::ProcessId, out_native_window: sf::OutMapAliasBuffer) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 2020] (name, id, aruid, out_native_window) => (native_window_size: usize))
    }

    fn create_stray_layer(&mut self, flags: LayerFlags, display_id: DisplayId, out_native_window: sf::OutMapAliasBuffer) -> Result<(LayerId, usize)> {
        ipc_client_send_request_command!([self.session.object_info; 2030] (flags, display_id, out_native_window) => (id: LayerId, native_window_size: usize))
    }

    fn destroy_stray_layer(&mut self, id: LayerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2031] (id) => ())
    }

    fn get_display_vsync_event(&mut self, display_id: DisplayId) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 5202] (display_id) => (event_handle: sf::CopyHandle))
    }
}

pub struct SystemRootService {
    session: sf::Session
}

impl sf::IObject for SystemRootService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_display_service: 1
        }
    }
}

impl service::IClientObject for SystemRootService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IRootService for SystemRootService {
    fn get_display_service(&mut self, mode: DisplayServiceMode) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 1] (mode) => (display_service: mem::Shared<ApplicationDisplayService>))
    }
}

impl service::IService for SystemRootService {
    fn get_name() -> &'static str {
        nul!("vi:s")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

pub struct ManagerRootService {
    session: sf::Session
}

impl sf::IObject for ManagerRootService {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            get_display_service: 2
        }
    }
}

impl service::IClientObject for ManagerRootService {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IRootService for ManagerRootService {
    fn get_display_service(&mut self, mode: DisplayServiceMode) -> Result<mem::Shared<dyn sf::IObject>> {
        ipc_client_send_request_command!([self.session.object_info; 2] (mode) => (display_service: mem::Shared<ApplicationDisplayService>))
    }
}

impl service::IService for ManagerRootService {
    fn get_name() -> &'static str {
        nul!("vi:m")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}