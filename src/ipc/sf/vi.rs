use crate::result::*;
use crate::ipc::sf;
use crate::util;
use crate::version;

use super::applet::AppletResourceUserId;

pub type DisplayName = util::ArrayString<0x40>;

define_bit_enum! {
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

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum LayerStackId {
    #[default]
    Default,
    Lcd,
    Screenshot,
    Recording,
    LastFrame,
    Arbitrary,
    ApplicationForDebug,
    Null
}

//api_mark_request_command_parameters_types_as_copy!(DisplayServiceMode, LayerStackId);

ipc_sf_define_default_interface_client!(ManagerDisplayService);
ipc_sf_define_interface_trait! {
	trait ManagerDisplayService {
        create_managed_layer [2010, version::VersionInterval::all(), mut]: (flags: LayerFlags, display_id: DisplayId, raw_aruid: u64) => (id: LayerId);
        destroy_managed_layer [2011, version::VersionInterval::all()]: (id: LayerId) => ();
        add_to_layer_stack [6000, version::VersionInterval::all()]: (stack: LayerStackId, layer: LayerId) => ();
    }
}

ipc_sf_define_default_interface_client!(SystemDisplayService);
ipc_sf_define_interface_trait! {
	trait SystemDisplayService {
        get_z_order_count_min [1200, version::VersionInterval::all()]: (display_id: DisplayId) => (z: i64);
        get_z_order_count_max [1202, version::VersionInterval::all()]: (display_id: DisplayId) => (z: i64);
        set_layer_position [2201, version::VersionInterval::all(), mut]: (x: f32, y: f32, id: LayerId) => ();
        set_layer_size [2203, version::VersionInterval::all(), mut]: (id: LayerId, width: u64, height: u64) => ();
        set_layer_z [2205, version::VersionInterval::all(), mut]: (id: LayerId, z: i64) => ();
        set_layer_visibility [2207, version::VersionInterval::all(), mut]: (visible: bool, id: LayerId) => ();
    }
}

ipc_sf_define_default_interface_client!(ApplicationDisplayService);
ipc_sf_define_interface_trait! {
	trait ApplicationDisplayService {
        get_relay_service [100, version::VersionInterval::all()]: () => (relay_service: sf::dispdrv::HOSBinderDriver);
        get_system_display_service [101, version::VersionInterval::all()]: () => (system_display_service: SystemDisplayService);
        get_manager_display_service [102, version::VersionInterval::all()]: () => (manager_display_service: ManagerDisplayService);
        open_display [1010, version::VersionInterval::all(), mut]: (name: DisplayName) => (id: DisplayId);
        close_display [1020, version::VersionInterval::all(), mut]: (id: DisplayId) => ();
        open_layer [2020, version::VersionInterval::all(), mut]: (name: DisplayName, id: LayerId, aruid: AppletResourceUserId, out_native_window: sf::OutMapAliasBuffer<u8>) => (native_window_size: usize);
        create_stray_layer [2030, version::VersionInterval::all(), mut]: (flags: LayerFlags, display_id: DisplayId, out_native_window: sf::OutMapAliasBuffer<u8>) => (id: LayerId, native_window_size: usize);
        destroy_stray_layer [2031, version::VersionInterval::all(), mut]: (id: LayerId) => ();
        get_display_vsync_event [5202, version::VersionInterval::all()]: (id: DisplayId) => (event_handle: sf::CopyHandle);
    }
}

ipc_sf_define_default_interface_client!(ApplicationRootService);
ipc_sf_define_interface_trait! {
	trait ApplicationRootService {
        get_display_service [0, version::VersionInterval::all()]: (mode: DisplayServiceMode) => (display_service: ApplicationDisplayService);
    }
}

ipc_sf_define_default_interface_client!(SystemRootService);
ipc_sf_define_interface_trait! {
	trait SystemRootService {
        get_display_service [1, version::VersionInterval::all()]: (mode: DisplayServiceMode) => (display_service: ApplicationDisplayService);
    }
}

ipc_sf_define_default_interface_client!(ManagerRootService);
ipc_sf_define_interface_trait! {
	trait ManagerRootService {
        get_display_service [2, version::VersionInterval::all()]: (mode: DisplayServiceMode) => (display_service: ApplicationDisplayService);
    }
}