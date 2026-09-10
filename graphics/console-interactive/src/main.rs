#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;
use alloc::sync::Arc;

use embedded_term::TextOnGraphic;
use nx::console::vty::PersistentBufferedCanvas;
use nx::diag::abort;
use nx::gpu;
use nx::input;
use nx::result::*;
use nx::service::hid;
use nx::service::hid::shmem::KeyboardState;
use nx::svc;
use nx::sync::RwLock;
use nx::thread;
use nx::util;

use core::fmt::Write;
use core::panic;

nx::rrt0_define_module_name!("console-interactive");

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    if hbl_heap.is_valid() {
        hbl_heap
    } else {
        let heap_size: usize = 0x10000000;
        let heap_address = svc::set_heap_size(heap_size).unwrap();
        util::PointerAndSize::new(heap_address, heap_size)
    }
}

#[no_mangle]
fn main() {
    let mut console: nx::console::vty::TextBufferConsole = {
        let gpu_ctx = match gpu::Context::new(
            gpu::NvDrvServiceKind::Applet,
            gpu::ViServiceKind::System,
            0x40000,
        ) {
            Ok(ctx) => ctx,
            Err(e) => panic!("{}", e),
        };

        let surface = match nx::gpu::canvas::CanvasManager::new_stray(
            Arc::new(RwLock::new(gpu_ctx)),
            Default::default(),
            2,
            gpu::BlockLinearHeights::FourGobs,
        ) {
            Ok(s) => s,
            Err(e) => panic!("{}", e),
        };

        let width = surface.surface.width();
        let height = surface.surface.height();

        let text_buffer = TextOnGraphic::new(PersistentBufferedCanvas::new(surface), width, height);

        embedded_term::Console::on_text_buffer(text_buffer)
    };

    let supported_style_tags = hid::NpadStyleTag::Handheld()
        | hid::NpadStyleTag::FullKey()
        | hid::NpadStyleTag::JoyDual()
        | hid::NpadStyleTag::JoyLeft()
        | hid::NpadStyleTag::JoyRight();
    let input_ctx =
        input::Context::new(supported_style_tags, 2).expect("Failed to create inpput context");

    let mut old_keyboard_state: KeyboardState = Default::default();
    'render: loop {
        for controller in [hid::NpadIdType::Handheld, hid::NpadIdType::No1]
            .iter()
            .cloned()
        {
            let mut player = input_ctx.get_player(controller);

            let buttons_down = player.get_buttons_down();
            if buttons_down.contains(hid::NpadButton::Down()) {
                let _ = console.write_str("\x1B[1B");
            } else if buttons_down.contains(hid::NpadButton::Up()) {
                let _ = console.write_str("\x1B[1A");
            } else if buttons_down.contains(hid::NpadButton::Plus()) {
                // Exit if Plus/+ is pressed.
                break 'render;
            }
        }

        let keyboard_state = input_ctx
            .get_player(hid::NpadIdType::Handheld)
            .get_keyboard_state();

        for key_down in keyboard_state.keys.clone() {
            if old_keyboard_state.keys.is_up(key_down) {
                // new key
                if let Some(ansi_str) = key_down.get_ansi() {
                    let _ = console.write_str(ansi_str);
                };
            }
        }

        old_keyboard_state = keyboard_state;

        let _ = thread::sleep(100_000);
    }
}

#[panic_handler]
fn panic_handler(info: &panic::PanicInfo) -> ! {
    let _info_message = format!("{}", info);
    nx::diag::abort::abort(abort::AbortLevel::Panic(), nx::rc::ResultPanicked::make())
}
