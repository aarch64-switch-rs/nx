use crate::result::*;
use crate::ipc::sf;
use crate::mem;

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

pub trait IAppletResource {
    ipc_interface_define_command!(get_shared_memory_handle: () => (shmem_handle: sf::CopyHandle));
}

pub trait IHidServer {
    ipc_interface_define_command!(create_applet_resource: (aruid: sf::ProcessId) => (applet_resource: mem::Shared<dyn sf::IObject>));
    ipc_interface_define_command!(set_supported_npad_style_set: (aruid: sf::ProcessId, npad_style_tag: NpadStyleTag) => ());
    ipc_interface_define_command!(set_supported_npad_id_type: (aruid: sf::ProcessId, controllers: sf::InPointerBuffer) => ());
    ipc_interface_define_command!(activate_npad: (aruid: sf::ProcessId) => ());
    ipc_interface_define_command!(deactivate_npad: (aruid: sf::ProcessId) => ());
    ipc_interface_define_command!(set_npad_joy_assignment_mode_single: (aruid: sf::ProcessId, controller: ControllerId, joy_type: NpadJoyDeviceType) => ());
    ipc_interface_define_command!(set_npad_joy_assignment_mode_dual: (aruid: sf::ProcessId, controller: ControllerId) => ());
}