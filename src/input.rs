use crate::result::*;
use crate::service::applet;
use crate::service::hid;
use crate::service::hid::IAppletResource;
use crate::service::hid::IHidServer;
use crate::ipc::sf;
use crate::svc;
use crate::mem;
use crate::vmem;
use crate::service;
use core::mem as cmem;

bit_enum! {
    Key (u64) {
        A = bit!(0),
        B = bit!(1),
        X = bit!(2),
        Y = bit!(3),
        LStick = bit!(4),
        RStick = bit!(5),
        L = bit!(6),
        R = bit!(7),
        ZL = bit!(8),
        ZR = bit!(9),
        Plus = bit!(10),
        Minus = bit!(11),
        Left = bit!(12),
        Right = bit!(13),
        Up = bit!(14),
        Down = bit!(15),
        LStickLeft = bit!(16),
        LStickUp = bit!(17),
        LStickRight = bit!(18),
        LStickDown = bit!(19),
        RStickLeft = bit!(20),
        RStickUp = bit!(21),
        RStickRight = bit!(22),
        RStickDown = bit!(23),
        SLLeft = bit!(24),
        SRLeft = bit!(25),
        SLRight = bit!(26),
        SRRight = bit!(27),
        Touch = bit!(28)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TouchData {
    pub timestamp: u64,
    pub pad: u32,
    pub index: u32,
    pub x: u32,
    pub y: u32,
    pub diameter_x: u32,
    pub diameter_y: u32,
    pub angle: u32,
    pub pad_2: u32
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TouchEntry {
    pub timestamp: u64,
    pub count: u64,
    pub touches: [TouchData; 16],
    pub pad: u64
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct TouchState {
    pub timestamp_ticks: u64,
    pub entry_count: u64,
    pub latest_index: u64,
    pub max_index: u64,
    pub timestamp: u64,
    pub entries: [TouchEntry; 17]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct JoystickPosition {
    pub x: u32,
    pub y: u32
}

bit_enum! {
    ConnectionState (u64) {
        None = 0,
        Connected = bit!(0),
        Wired = bit!(1)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerStateEntry {
    pub timestamp: u64,
    pub timestamp_2: u64,
    pub button_state: u64,
    pub left_position: JoystickPosition,
    pub right_position: JoystickPosition,
    pub connection_state: ConnectionState
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerState {
    pub timestamp: u64,
    pub entry_count: u64,
    pub latest_index: u64,
    pub max_index: u64,
    pub entries: [ControllerStateEntry; 17]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerMacAddress {
    pub address: [u8; 0x10]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerColor {
    pub body: u32,
    pub buttons: u32
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ControllerData {
    pub status: u32,
    pub is_joycon_half: bool,
    pub pad: [u8; 3],
    pub color_descriptor_single: u32,
    pub color_single: ControllerColor,
    pub color_descriptor_split: u32,
    pub color_right: ControllerColor,
    pub color_left: ControllerColor,
    pub pro_controller_state: ControllerState,
    pub handheld_state: ControllerState,
    pub joined_state: ControllerState,
    pub left_state: ControllerState,
    pub right_state: ControllerState,
    pub main_no_analog_state: ControllerState,
    pub main_state: ControllerState,
    pub unk: [u8; 0x2A78],
    pub mac_addresses: [ControllerMacAddress; 2],
    pub unk_2: [u8; 0xE10]
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct SharedMemoryData {
    pub header: [u8; 0x400],
    pub touch_state: TouchState,
    pub pad: [u8; 0x3C0],
    pub mouse: [u8; 0x400],
    pub keyboard: [u8; 0x400],
    pub unk: [u8; 0x400],
    pub unk_2: [u8; 0x400],
    pub unk_3: [u8; 0x400],
    pub unk_4: [u8; 0x400],
    pub unk_5: [u8; 0x200],
    pub unk_6: [u8; 0x200],
    pub unk_7: [u8; 0x200],
    pub unk_8: [u8; 0x800],
    pub controller_serials: [u8; 0x4000],
    pub controllers: [ControllerData; 10],
    pub unk_9: [u8; 0x4600]
}

pub struct Player {
    controller: hid::ControllerId,
    data: *const ControllerData,
    prev_button_state: u64
}

impl Player {
    pub fn new(controller: hid::ControllerId, data: *const ControllerData) -> Self {
        Self { controller: controller, data: data, prev_button_state: 0 }
    }

    fn get_button_state(&self) -> u64 {
        let last_entry = unsafe { (*self.data).main_state.entries[(*self.data).main_state.latest_index as usize] };
        last_entry.button_state
    }

    pub fn get_button_state_held(&mut self) -> Key {
        let button_state = self.get_button_state();
        self.prev_button_state = button_state;
        Key(button_state)
    }

    pub fn get_button_state_down(&mut self) -> Key {
        let button_state = self.get_button_state();
        let down_state = (!self.prev_button_state) & button_state;
        self.prev_button_state = button_state;
        Key(down_state)
    }

    pub fn get_button_state_up(&mut self) -> Key {
        let button_state = self.get_button_state();
        let up_state = self.prev_button_state & (!button_state);
        self.prev_button_state = button_state;
        Key(up_state)
    }

    pub fn get_controller(&self) -> hid::ControllerId {
        self.controller
    }
}

#[allow(dead_code)]
pub struct InputContext {
    hid_service: mem::Shared<hid::HidServer>,
    applet_resource: mem::Shared<hid::AppletResource>,
    shared_mem_handle: svc::Handle,
    aruid: applet::AppletResourceUserId,
    shared_mem_data: *const SharedMemoryData
}

macro_rules! set_all_controllers_mode_dual_impl {
    (? $srv:expr, $process_id:expr, $( $id:expr ),*) => {
        $( $srv.get().set_npad_joy_assignment_mode_dual($process_id, $id)?; )*
    };
    ($srv:expr, $process_id:expr, $( $id:expr ),*) => {
        $( let _ = $srv.get().set_npad_joy_assignment_mode_dual($process_id, $id); )*
    };
}

#[allow(unreachable_patterns)]
fn get_index_for_controller(controller: hid::ControllerId) -> Result<usize> {
    match controller {
        hid::ControllerId::Player1 | hid::ControllerId::Player2 | hid::ControllerId::Player3 | hid::ControllerId::Player4 | hid::ControllerId::Player5 | hid::ControllerId::Player6 | hid::ControllerId::Player7 | hid::ControllerId::Player8 => Ok(controller as usize),
        hid::ControllerId::Handheld => Ok(8),
        _ => Err(ResultCode::new(0xBAAF))
    }
}

impl InputContext {
    pub fn new(aruid: applet::AppletResourceUserId, supported_tags: hid::NpadStyleTag, controllers: &[hid::ControllerId]) -> Result<Self> {
        let hid_srv = service::new_service_object::<hid::HidServer>()?;
        let hid_process_id = sf::ProcessId::from(aruid);
        let applet_res = hid_srv.get().create_applet_resource(hid_process_id)?.to::<hid::AppletResource>();
        let shmem_handle = applet_res.get().get_shared_memory_handle()?;
        let shmem_size = cmem::size_of::<SharedMemoryData>();
        let shmem_address = vmem::allocate(shmem_size)?;
        svc::map_shared_memory(shmem_handle.handle, shmem_address, shmem_size, svc::MemoryPermission::Read())?;
        hid_srv.get().activate_npad(hid_process_id)?;
        hid_srv.get().set_supported_npad_style_set(hid_process_id, supported_tags)?;
        hid_srv.get().set_supported_npad_id_type(hid_process_id, sf::Buffer::from_array(controllers))?;
        hid_srv.get().activate_npad(hid_process_id)?;
        set_all_controllers_mode_dual_impl!(? hid_srv, hid_process_id, hid::ControllerId::Player1, hid::ControllerId::Player2, hid::ControllerId::Player3, hid::ControllerId::Player4, hid::ControllerId::Player5, hid::ControllerId::Player6, hid::ControllerId::Player7, hid::ControllerId::Player8, hid::ControllerId::Handheld);
        Ok(Self { hid_service: hid_srv, applet_resource: applet_res, shared_mem_handle: shmem_handle.handle, aruid: aruid, shared_mem_data: shmem_address as *const SharedMemoryData })
    }

    pub fn is_controller_connected(&mut self, controller: hid::ControllerId) -> bool {
        if let Ok(index) = get_index_for_controller(controller) {
            let controller_data = unsafe { &(*self.shared_mem_data).controllers[index] };
            let last_entry = controller_data.main_state.entries[controller_data.main_state.latest_index as usize];
            last_entry.connection_state.contains(ConnectionState::Connected())
        }
        else {
            false
        }
    }

    pub fn get_player(&mut self, controller: hid::ControllerId) -> Result<Player> {
        let index = get_index_for_controller(controller)?;
        let controller_data: *const ControllerData = unsafe { &(*self.shared_mem_data).controllers[index] };
        Ok(Player::new(controller, controller_data))
    }

    pub fn get_touch_data(&mut self, touch_index: u32) -> Result<TouchData> {
        unsafe {
            let touch_entry: *const TouchEntry = &(*self.shared_mem_data).touch_state.entries[(*self.shared_mem_data).touch_state.latest_index as usize];
            result_return_unless!((touch_index as u64) < (*touch_entry).count, 0xBAEEF);
            Ok((*touch_entry).touches[touch_index as usize])
        }
    }
}

impl Drop for InputContext {
    fn drop(&mut self) {
        let hid_process_id = sf::ProcessId::from(self.aruid);
        set_all_controllers_mode_dual_impl!(self.hid_service, hid_process_id, hid::ControllerId::Player1, hid::ControllerId::Player2, hid::ControllerId::Player3, hid::ControllerId::Player4, hid::ControllerId::Player5, hid::ControllerId::Player6, hid::ControllerId::Player7, hid::ControllerId::Player8, hid::ControllerId::Handheld);
        let _ = self.hid_service.get().deactivate_npad(hid_process_id);
        let _ = svc::unmap_shared_memory(self.shared_mem_handle, self.shared_mem_data as *mut u8, cmem::size_of::<SharedMemoryData>());
        let _ = svc::close_handle(self.shared_mem_handle);
    }
}