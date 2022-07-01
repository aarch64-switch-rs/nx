use crate::ipc::sf::sm;
use crate::result::*;
use crate::ipc::sf;
use crate::service;
use crate::mem;
use crate::service::dispdrv;
use crate::service::applet;

pub use crate::ipc::sf::vi::*;

ipc_client_define_object_default!(ManagerDisplayService);

impl IManagerDisplayService for ManagerDisplayService {
    fn create_managed_layer(&mut self, flags: LayerFlags, display_id: DisplayId, aruid: applet::AppletResourceUserId) -> Result<LayerId> {
        ipc_client_send_request_command!([self.session.object_info; 2010] (flags, display_id, aruid) => (id: LayerId))
    }

    fn destroy_managed_layer(&mut self, id: LayerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2011] (id) => ())
    }
}

ipc_client_define_object_default!(SystemDisplayService);

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

ipc_client_define_object_default!(ApplicationDisplayService);

impl IApplicationDisplayService for ApplicationDisplayService {
    fn get_relay_service(&mut self) -> Result<mem::Shared<dyn dispdrv::IHOSBinderDriver>> {
        ipc_client_send_request_command!([self.session.object_info; 100] () => (relay_service: mem::Shared<dispdrv::HOSBinderDriver>))
    }

    fn get_system_display_service(&mut self) -> Result<mem::Shared<dyn ISystemDisplayService>> {
        ipc_client_send_request_command!([self.session.object_info; 101] () => (relay_service: mem::Shared<SystemDisplayService>))
    }

    fn get_manager_display_service(&mut self) -> Result<mem::Shared<dyn IManagerDisplayService>> {
        ipc_client_send_request_command!([self.session.object_info; 102] () => (relay_service: mem::Shared<ManagerDisplayService>))
    }

    fn open_display(&mut self, name: DisplayName) -> Result<DisplayId> {
        ipc_client_send_request_command!([self.session.object_info; 1010] (name) => (id: DisplayId))
    }

    fn close_display(&mut self, display_id: DisplayId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1020] (display_id) => ())
    }

    fn open_layer(&mut self, name: DisplayName, id: LayerId, aruid: sf::ProcessId, out_native_window: sf::OutMapAliasBuffer<u8>) -> Result<usize> {
        ipc_client_send_request_command!([self.session.object_info; 2020] (name, id, aruid, out_native_window) => (native_window_size: usize))
    }

    fn create_stray_layer(&mut self, flags: LayerFlags, display_id: DisplayId, out_native_window: sf::OutMapAliasBuffer<u8>) -> Result<(LayerId, usize)> {
        ipc_client_send_request_command!([self.session.object_info; 2030] (flags, display_id, out_native_window) => (id: LayerId, native_window_size: usize))
    }

    fn destroy_stray_layer(&mut self, id: LayerId) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 2031] (id) => ())
    }

    fn get_display_vsync_event(&mut self, display_id: DisplayId) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 5202] (display_id) => (event_handle: sf::CopyHandle))
    }
}

ipc_client_define_object_default!(ApplicationRootService);

impl IApplicationRootService for ApplicationRootService {
    fn get_display_service(&mut self, mode: DisplayServiceMode) -> Result<mem::Shared<dyn IApplicationDisplayService>> {
        ipc_client_send_request_command!([self.session.object_info; 0] (mode) => (display_service: mem::Shared<ApplicationDisplayService>))
    }
}

impl service::IService for ApplicationRootService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("vi:u")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

ipc_client_define_object_default!(SystemRootService);

impl ISystemRootService for SystemRootService {
    fn get_display_service(&mut self, mode: DisplayServiceMode) -> Result<mem::Shared<dyn IApplicationDisplayService>> {
        ipc_client_send_request_command!([self.session.object_info; 1] (mode) => (display_service: mem::Shared<ApplicationDisplayService>))
    }
}

impl service::IService for SystemRootService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("vi:s")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}

ipc_client_define_object_default!(ManagerRootService);

impl IManagerRootService for ManagerRootService {
    fn get_display_service(&mut self, mode: DisplayServiceMode) -> Result<mem::Shared<dyn IApplicationDisplayService>> {
        ipc_client_send_request_command!([self.session.object_info; 2] (mode) => (display_service: mem::Shared<ApplicationDisplayService>))
    }
}

impl service::IService for ManagerRootService {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("vi:m")
    }

    fn as_domain() -> bool {
        true
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}