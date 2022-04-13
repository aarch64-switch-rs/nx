use crate::result::*;
use crate::ipc::sf;
use crate::mem;
use crate::version;

bit_enum! {
    NpadStyleTag (u32) {
        ProController = bit!(0),
        Handheld = bit!(1),
        JoyconPair = bit!(2),
        JoyconLeft = bit!(3),
        JoyconRight = bit!(4),
        SystemExt = bit!(29),
        System = bit!(30)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i64)]
pub enum NpadJoyDeviceType {
    Left,
    Right
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ControllerId {
    Player1 = 0,
    Player2 = 1,
    Player3 = 2,
    Player4 = 3,
    Player5 = 4,
    Player6 = 5,
    Player7 = 6,
    Player8 = 7,
    Handheld = 0x20
}

ipc_sf_define_interface_trait! {
    trait IAppletResource {
        get_shared_memory_handle [1, version::VersionInterval::all()]: () => (shmem_handle: sf::CopyHandle);
    }
}

ipc_sf_define_interface_trait! {
    trait IHidServer {
        create_applet_resource [0, version::VersionInterval::all()]: (aruid: sf::ProcessId) => (applet_resource: mem::Shared<dyn IAppletResource>);
        set_supported_npad_style_set [100, version::VersionInterval::all()]: (aruid: sf::ProcessId, npad_style_tag: NpadStyleTag) => ();
        set_supported_npad_id_type [102, version::VersionInterval::all()]: (aruid: sf::ProcessId, controllers: sf::InPointerBuffer<ControllerId>) => ();
        activate_npad [103, version::VersionInterval::all()]: (aruid: sf::ProcessId) => ();
        deactivate_npad [104, version::VersionInterval::all()]: (aruid: sf::ProcessId) => ();
        set_npad_joy_assignment_mode_single [123, version::VersionInterval::all()]: (aruid: sf::ProcessId, controller: ControllerId, joy_type: NpadJoyDeviceType) => ();
        set_npad_joy_assignment_mode_dual [124, version::VersionInterval::all()]: (aruid: sf::ProcessId, controller: ControllerId) => ();
    }
}