use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::util;
use crate::ipc::sf::applet;
use crate::ipc::sf::dispdrv;
use crate::version;

pub type DisplayName = util::CString<0x40>;

bit_enum! {
    LayerFlags (u32) {
        None = 0,
        Default = bit!(0)
    }
}

pub type DisplayId = u64;

pub type LayerId = u64;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum DisplayServiceMode {
    User = 0,
    Privileged = 1
}

ipc_sf_define_interface_trait! {
    trait IManagerDisplayService {
        create_managed_layer [2010, version::VersionInterval::all()]: (flags: LayerFlags, display_id: DisplayId, aruid: applet::AppletResourceUserId) => (id: LayerId);
        destroy_managed_layer [2011, version::VersionInterval::all()]: (id: LayerId) => ();
    }
}

ipc_sf_define_interface_trait! {
    trait ISystemDisplayService {
        get_z_order_count_min [1200, version::VersionInterval::all()]: (display_id: DisplayId) => (z: i64);
        get_z_order_count_max [1202, version::VersionInterval::all()]: (display_id: DisplayId) => (z: i64);
        set_layer_position [2201, version::VersionInterval::all()]: (x: f32, y: f32, id: LayerId) => ();
        set_layer_size [2203, version::VersionInterval::all()]: (id: LayerId, width: u64, height: u64) => ();
        set_layer_z [2205, version::VersionInterval::all()]: (id: LayerId, z: i64) => ();
        set_layer_visibility [2207, version::VersionInterval::all()]: (visible: bool, id: LayerId) => ();
    }
}

ipc_sf_define_interface_trait! {
    trait IApplicationDisplayService {
        get_relay_service [100, version::VersionInterval::all()]: () => (relay_service: mem::Shared<dyn dispdrv::IHOSBinderDriver>);
        get_system_display_service [101, version::VersionInterval::all()]: () => (system_display_service: mem::Shared<dyn ISystemDisplayService>);
        get_manager_display_service [102, version::VersionInterval::all()]: () => (manager_display_service: mem::Shared<dyn IManagerDisplayService>);
        open_display [1010, version::VersionInterval::all()]: (name: DisplayName) => (id: DisplayId);
        close_display [1020, version::VersionInterval::all()]: (id: DisplayId) => ();
        open_layer [2020, version::VersionInterval::all()]: (name: DisplayName, id: LayerId, aruid: sf::ProcessId, out_native_window: sf::OutMapAliasBuffer<u8>) => (native_window_size: usize);
        create_stray_layer [2030, version::VersionInterval::all()]: (flags: LayerFlags, display_id: DisplayId, out_native_window: sf::OutMapAliasBuffer<u8>) => (id: LayerId, native_window_size: usize);
        destroy_stray_layer [2031, version::VersionInterval::all()]: (id: LayerId) => ();
        get_display_vsync_event [5202, version::VersionInterval::all()]: (id: DisplayId) => (event_handle: sf::CopyHandle);
    }
}

ipc_sf_define_interface_trait! {
    trait IApplicationRootService {
        get_display_service [0, version::VersionInterval::all()]: (mode: DisplayServiceMode) => (display_service: mem::Shared<dyn IApplicationDisplayService>);
    }
}

ipc_sf_define_interface_trait! {
    trait ISystemRootService {
        get_display_service [1, version::VersionInterval::all()]: (mode: DisplayServiceMode) => (display_service: mem::Shared<dyn IApplicationDisplayService>);
    }
}

ipc_sf_define_interface_trait! {
    trait IManagerRootService {
        get_display_service [2, version::VersionInterval::all()]: (mode: DisplayServiceMode) => (display_service: mem::Shared<dyn IApplicationDisplayService>);
    }
}