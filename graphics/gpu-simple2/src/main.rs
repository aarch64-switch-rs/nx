#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use alloc::sync::Arc;
use nx::diag::abort;
use nx::gpu;
use nx::input;
use nx::result::*;
use nx::service::hid;
use nx::sync::RwLock;
use nx::util;

use core::panic;

use nx::gpu::canvas::Canvas;

nx::rrt0_define_module_name!("gpu-simple2");

#[no_mangle]
pub fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize {
    hbl_heap
}

type RGBType = nx::gpu::canvas::RGBA8;

#[no_mangle]
pub fn main() {
    let supported_tags =
        hid::NpadStyleTag::Handheld() | hid::NpadStyleTag::FullKey() | hid::NpadStyleTag::JoyDual();
    let input_ctx = input::Context::new(supported_tags, 1).unwrap();

    let _c_empty = RGBType::new();
    let _c_white = RGBType::new_scaled(0xff, 0xff, 0xff, 0xff);
    let c_black = RGBType::new_scaled(0, 0, 0, 0xff);
    let _c_royal_blue = RGBType::new_scaled(65, 105, 225, 255);

    let font =
        nx::gpu::canvas::Font::try_from_slice(include_bytes!("../../font/Roboto-Medium.ttf"))
            .unwrap();

    let gpu_ctx = gpu::Context::new(
        gpu::NvDrvServiceKind::Applet,
        gpu::ViServiceKind::System,
        0x40000,
    )
    .unwrap();
    let mut surface = match nx::gpu::canvas::CanvasManager::new_stray(
        Arc::new(RwLock::new(gpu_ctx)),
        Default::default(),
        3,
        gpu::BlockLinearHeights::FourGobs,
    ) {
        Ok(s) => s,
        Err(e) => panic!("{:?}", e),
    };

    let mut frame: usize = 0;

    'render: loop {
        for input_buttons in [hid::NpadIdType::Handheld, hid::NpadIdType::No1]
            .iter()
            .cloned()
            .map(|controller| input_ctx.get_player(controller).get_buttons_down())
        {
            if input_buttons.contains(hid::NpadButton::Plus()) {
                // Exit if Plus/+ is pressed.
                break 'render;
            }
        }

        frame += 1;

        let _ = surface.render(Some(RGBType::new_scaled(255, 255, 255, 255)), |c| {
            c.draw_ascii_bitmap_text(
                format!("frame #: {}", frame),
                c_black,
                5,
                600,
                0,
                gpu::canvas::AlphaBlend::Destination,
            );

            c.draw_rect(
                0,
                0,
                50,
                50,
                RGBType::new_scaled(255, 0, 0, 255),
                gpu::canvas::AlphaBlend::None,
            );
            c.draw_ascii_bitmap_text(
                "r",
                RGBType::new_scaled(255, 0, 0, 255),
                5,
                60,
                10,
                gpu::canvas::AlphaBlend::Destination,
            );

            c.draw_rect(
                0,
                100,
                50,
                50,
                RGBType::new_scaled(0, 255, 0, 255),
                gpu::canvas::AlphaBlend::None,
            );
            c.draw_font_text(
                &font,
                "g",
                RGBType::new_scaled(0, 255, 0, 255),
                30.0,
                60,
                110,
                gpu::canvas::AlphaBlend::Destination,
            );

            c.draw_rect(
                0,
                200,
                50,
                50,
                RGBType::new_scaled(0, 0, 255, 255),
                gpu::canvas::AlphaBlend::None,
            );
            c.draw_font_text(
                &font,
                "b",
                RGBType::new_scaled(0, 0, 0255, 255),
                50.0,
                60,
                210,
                gpu::canvas::AlphaBlend::Source,
            );

            for py in 200..456 {
                for px in 200..456 {
                    c.draw_single(
                        px,
                        py,
                        RGBType::new_scaled((py - 200) as u8, 255 - (px - 200) as u8, 0, 255),
                        gpu::canvas::AlphaBlend::None,
                    );
                }
            }

            c.draw_line(
                (600, 350),
                (700, 250),
                10,
                RGBType::new_scaled(0, 0, 0, 128),
                gpu::canvas::AlphaBlend::Source,
            );
            c.draw_line(
                (700, 250),
                (750, 250),
                10,
                RGBType::new_scaled(0, 0, 0, 255),
                gpu::canvas::AlphaBlend::None,
            );
            c.draw_line(
                (750, 250),
                (850, 350),
                10,
                RGBType::new_scaled(0, 0, 0, 128),
                gpu::canvas::AlphaBlend::Destination,
            );
            c.draw_line(
                (850, 350),
                (850, 400),
                10,
                RGBType::new_scaled(0, 0, 0, 255),
                gpu::canvas::AlphaBlend::None,
            );
            c.draw_line(
                (850, 400),
                (750, 500),
                10,
                RGBType::new_scaled(0, 0, 0, 128),
                gpu::canvas::AlphaBlend::Source,
            );
            c.draw_line(
                (750, 500),
                (700, 500),
                10,
                RGBType::new_scaled(0, 0, 0, 255),
                gpu::canvas::AlphaBlend::None,
            );
            c.draw_line(
                (700, 500),
                (600, 400),
                10,
                RGBType::new_scaled(0, 0, 0, 255),
                gpu::canvas::AlphaBlend::Destination,
            );
            c.draw_line(
                (600, 400),
                (600, 350),
                10,
                RGBType::new_scaled(0, 0, 0, 255),
                gpu::canvas::AlphaBlend::None,
            );

            Ok(())
        });
        let _ = surface.wait_vsync_event(None);
    }
}

#[panic_handler]
fn panic_handler(_info: &panic::PanicInfo) -> ! {
    //let panic_str = format!("{}", info);
    nx::diag::abort::abort(abort::AbortLevel::Panic(), nx::rc::ResultPanicked::make())
    //util::simple_panic_handler::<LmLogger>(info, abort::AbortLevel::FatalThrow())
}
