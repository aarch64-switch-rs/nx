use crate::result::*;
use crate::service::hid;
use crate::service::hid::shmem;
use crate::service::hid::IAppletResource;
use crate::service::hid::IHidServer;
use crate::ipc::sf;
use crate::svc;
use crate::mem;
use crate::version;
use crate::vmem;
use crate::service;

pub mod rc;

#[inline(always)]
fn get_npad_id_shmem_entry_index(npad_id: hid::NpadIdType) -> usize {
    match npad_id {
        hid::NpadIdType::Handheld => 8,
        hid::NpadIdType::Other => 9,
        _ => npad_id as usize // (No1...8 -> 0...7)
    }
}

macro_rules! get_shmem_npad_entry_ring_lifo {
    ($shmem_ptr:expr, $npad_id_idx:expr, $shmem_npad_lifo_type:ident) => {{
        let cur_ver = version::get_version();
        if hid::shmem::SharedMemoryFormatV1::VERSION_INTERVAL.matches(cur_ver) {
            &(*($shmem_ptr as *const hid::shmem::SharedMemoryFormatV1)).npad.entries[$npad_id_idx].$shmem_npad_lifo_type
        }
        else if hid::shmem::SharedMemoryFormatV2::VERSION_INTERVAL.matches(cur_ver) {
            &(*($shmem_ptr as *const hid::shmem::SharedMemoryFormatV2)).npad.entries[$npad_id_idx].$shmem_npad_lifo_type
        }
        else if hid::shmem::SharedMemoryFormatV3::VERSION_INTERVAL.matches(cur_ver) {
            &(*($shmem_ptr as *const hid::shmem::SharedMemoryFormatV3)).npad.entries[$npad_id_idx].$shmem_npad_lifo_type
        } 
        else if hid::shmem::SharedMemoryFormatV4::VERSION_INTERVAL.matches(cur_ver) {
            &(*($shmem_ptr as *const hid::shmem::SharedMemoryFormatV4)).npad.entries[$npad_id_idx].$shmem_npad_lifo_type
        }
        else if hid::shmem::SharedMemoryFormatV5::VERSION_INTERVAL.matches(cur_ver) {
            &(*($shmem_ptr as *const hid::shmem::SharedMemoryFormatV5)).npad.entries[$npad_id_idx].$shmem_npad_lifo_type
        }
        else if hid::shmem::SharedMemoryFormatV6::VERSION_INTERVAL.matches(cur_ver) {
            &(*($shmem_ptr as *const hid::shmem::SharedMemoryFormatV6)).npad.entries[$npad_id_idx].$shmem_npad_lifo_type
        }
        else {
            // TODO: result?
            panic!("Unexpected version mismatch");
        }
    }};
}

macro_rules! get_state_one_tag {
    ($shmem_ptr:expr, $npad_id_idx:expr, $style_tag:expr, $state_field:ident) => { unsafe {
        if $style_tag.contains(hid::NpadStyleTag::FullKey()) {
            get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, full_key_lifo).get_tail_item().state.$state_field
        }
        else if $style_tag.contains(hid::NpadStyleTag::Handheld()) {
            get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, handheld_lifo).get_tail_item().state.$state_field
        }
        else if $style_tag.contains(hid::NpadStyleTag::JoyDual()) {
            get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, joy_dual_lifo).get_tail_item().state.$state_field
        }
        else if $style_tag.contains(hid::NpadStyleTag::JoyLeft()) {
            get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, joy_left_lifo).get_tail_item().state.$state_field
        }
        else if $style_tag.contains(hid::NpadStyleTag::JoyRight()) {
            get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, joy_right_lifo).get_tail_item().state.$state_field
        }
        else if $style_tag.contains(hid::NpadStyleTag::System()) || $style_tag.contains(hid::NpadStyleTag::SystemExt()) {
            get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, system_ext_lifo).get_tail_item().state.$state_field
        }
        else {
            Default::default()
        }
    }};
}

macro_rules! get_state_multi_tag {
    ($shmem_ptr:expr, $npad_id_idx:expr, $style_tag:expr, $state_type:ty, $state_field:ident) => { unsafe {
        let mut state: $state_type = Default::default();
        if $style_tag.contains(hid::NpadStyleTag::FullKey()) {
            state |= get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, full_key_lifo).get_tail_item().state.$state_field;
        }
        if $style_tag.contains(hid::NpadStyleTag::Handheld()) {
            state |= get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, handheld_lifo).get_tail_item().state.$state_field;
        }
        if $style_tag.contains(hid::NpadStyleTag::JoyDual()) {
            state |= get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, joy_dual_lifo).get_tail_item().state.$state_field;
        }
        if $style_tag.contains(hid::NpadStyleTag::JoyLeft()) {
            state |= get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, joy_left_lifo).get_tail_item().state.$state_field;
        }
        if $style_tag.contains(hid::NpadStyleTag::JoyRight()) {
            state |= get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, joy_right_lifo).get_tail_item().state.$state_field;
        }
        if $style_tag.contains(hid::NpadStyleTag::System()) || $style_tag.contains(hid::NpadStyleTag::SystemExt()) {
            state |= get_shmem_npad_entry_ring_lifo!($shmem_ptr, $npad_id_idx, system_ext_lifo).get_tail_item().state.$state_field;
        }
        state
    }};
}

pub struct Player {
    npad_id: hid::NpadIdType,
    npad_id_idx: usize,
    supported_style_tags: hid::NpadStyleTag,
    shmem_ptr: *const u8,
    prev_buttons: hid::NpadButton
}

impl Player {
    pub fn new(npad_id: hid::NpadIdType, supported_style_tags: hid::NpadStyleTag, shmem_ptr: *const u8) -> Self {
        Self {
            npad_id,
            npad_id_idx: get_npad_id_shmem_entry_index(npad_id),
            supported_style_tags,
            shmem_ptr,
            prev_buttons: Default::default()
        }
    }

    #[inline]
    pub fn get_style_tag_attributes(&self, style_tag: hid::NpadStyleTag) -> hid::NpadAttribute {
        get_state_one_tag!(self.shmem_ptr, self.npad_id_idx, style_tag, attributes)
    }

    pub fn get_style_tag_buttons(&mut self, style_tag: hid::NpadStyleTag) -> hid::NpadButton {
        let cur_buttons = get_state_one_tag!(self.shmem_ptr, self.npad_id_idx, style_tag, buttons);
        self.prev_buttons = cur_buttons;
        cur_buttons
    }

    pub fn get_style_tag_buttons_down(&mut self, style_tag: hid::NpadStyleTag) -> hid::NpadButton {
        let prev_buttons = self.prev_buttons;
        let cur_buttons = self.get_style_tag_buttons(style_tag);
        (!prev_buttons) & cur_buttons
    }

    pub fn get_style_tag_buttons_up(&mut self, style_tag: hid::NpadStyleTag) -> hid::NpadButton {
        let prev_buttons = self.prev_buttons;
        let cur_buttons = self.get_style_tag_buttons(style_tag);
        prev_buttons & (!cur_buttons)
    }

    pub fn get_buttons(&mut self) -> hid::NpadButton {
        let cur_buttons = get_state_multi_tag!(self.shmem_ptr, self.npad_id_idx, self.supported_style_tags, hid::NpadButton, buttons);
        self.prev_buttons = cur_buttons;
        cur_buttons
    }

    pub fn get_buttons_down(&mut self) -> hid::NpadButton {
        let prev_buttons = self.prev_buttons;
        let cur_buttons = self.get_buttons();
        (!prev_buttons) & cur_buttons
    }

    pub fn get_buttons_up(&mut self) -> hid::NpadButton {
        let prev_buttons = self.prev_buttons;
        let cur_buttons = self.get_buttons();
        prev_buttons & (!cur_buttons)
    }

    #[inline]
    pub fn get_npad_id(&self) -> hid::NpadIdType {
        self.npad_id
    }

    #[inline]
    pub fn get_supported_style_tags(&self) -> hid::NpadStyleTag {
        self.supported_style_tags
    }
}

#[allow(dead_code)]
pub struct Context {
    hid_service: mem::Shared<dyn IHidServer>,
    applet_resource: mem::Shared<dyn IAppletResource>,
    supported_style_tags: hid::NpadStyleTag,
    shmem_handle: svc::Handle,
    shmem_ptr: *const u8
}

impl Context {
    pub fn new(supported_style_tags: hid::NpadStyleTag, supported_npad_ids: &[hid::NpadIdType]) -> Result<Self> {
        let hid_srv = service::new_service_object::<hid::HidServer>()?;
        let applet_res = hid_srv.get().create_applet_resource(sf::ProcessId::new())?;
        
        let shmem_handle = applet_res.get().get_shared_memory_handle()?;
        let shmem_address = vmem::allocate(shmem::SHMEM_SIZE)?;
        svc::map_shared_memory(shmem_handle.handle, shmem_address, shmem::SHMEM_SIZE, svc::MemoryPermission::Read())?;

        hid_srv.get().activate_npad(sf::ProcessId::new())?;
        hid_srv.get().set_supported_npad_style_set(sf::ProcessId::new(), supported_style_tags)?;
        hid_srv.get().set_supported_npad_id_type(sf::ProcessId::new(), sf::Buffer::from_array(supported_npad_ids))?;

        Ok(Self {
            hid_service: hid_srv,
            applet_resource: applet_res,
            supported_style_tags,
            shmem_handle: shmem_handle.handle,
            shmem_ptr: shmem_address as *const u8
        })
    }

    #[inline]
    pub fn get_player(&self, npad_id: hid::NpadIdType) -> Player {
        Player::new(npad_id, self.supported_style_tags, self.shmem_ptr)
    }

    pub fn get_touches(&mut self, touch_states: &mut [hid::TouchState]) -> usize {
        unsafe {
            let cur_ver = version::get_version();
            let touch_screen_shmem = if hid::shmem::SharedMemoryFormatV1::VERSION_INTERVAL.matches(cur_ver) {
                &(*(self.shmem_ptr as *const hid::shmem::SharedMemoryFormatV1)).touch_screen
            }
            else if hid::shmem::SharedMemoryFormatV2::VERSION_INTERVAL.matches(cur_ver) {
                &(*(self.shmem_ptr as *const hid::shmem::SharedMemoryFormatV2)).touch_screen
            }
            else if hid::shmem::SharedMemoryFormatV3::VERSION_INTERVAL.matches(cur_ver) {
                &(*(self.shmem_ptr as *const hid::shmem::SharedMemoryFormatV3)).touch_screen
            } 
            else if hid::shmem::SharedMemoryFormatV4::VERSION_INTERVAL.matches(cur_ver) {
                &(*(self.shmem_ptr as *const hid::shmem::SharedMemoryFormatV4)).touch_screen
            }
            else if hid::shmem::SharedMemoryFormatV5::VERSION_INTERVAL.matches(cur_ver) {
                &(*(self.shmem_ptr as *const hid::shmem::SharedMemoryFormatV5)).touch_screen
            }
            else if hid::shmem::SharedMemoryFormatV6::VERSION_INTERVAL.matches(cur_ver) {
                &(*(self.shmem_ptr as *const hid::shmem::SharedMemoryFormatV6)).touch_screen
            }
            else {
                // TODO: result?
                panic!("Unexpected version mismatch");
            };

            let screen_state = touch_screen_shmem.lifo.get_tail_item().state;
            let min_count = touch_states.len().min(screen_state.count as usize);
            for i in 0..min_count {
                touch_states[i] = screen_state.touches[i];
            }
            min_count
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        let _ = self.hid_service.get().deactivate_npad(sf::ProcessId::new());
        let _ = svc::unmap_shared_memory(self.shmem_handle, self.shmem_ptr as *mut u8, shmem::SHMEM_SIZE);
        let _ = svc::close_handle(self.shmem_handle);
    }
}