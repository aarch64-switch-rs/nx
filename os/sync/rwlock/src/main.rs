#![no_std]
#![no_main]

extern crate alloc;
use alloc::format;

use alloc::sync::Arc;
use nx::diag::abort;
use nx::gpu;
use nx::input;
use nx::ipc::sf::Buffer;
use nx::result::*;
use nx::service::hid;
use nx::service::new_service_object;
use nx::service::spl::{IRandomClient, RandomService};
use nx::sync::RwLock;
use nx::thread;

use core::panic;
use core::sync::atomic::AtomicBool;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

use nx::gpu::canvas::Canvas;

nx::rrt0_define_module_name!("rwlock-test");
nx::rrt0_initialize_heap!();

type RGBType = nx::gpu::canvas::RGBA4;

#[unsafe(no_mangle)]
pub fn main() {
    let supported_tags =
        hid::NpadStyleTag::Handheld() | hid::NpadStyleTag::FullKey() | hid::NpadStyleTag::JoyDual();
    let input_ctx = match input::Context::new(supported_tags, 1) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{:?}", e),
    };

    let _c_empty = RGBType::new();
    let _c_white = RGBType::new_scaled(0xff, 0xff, 0xff, 0xff);
    let c_black = RGBType::new_scaled(0, 0, 0, 0xff);
    let _c_royal_blue = RGBType::new_scaled(65, 105, 225, 255);

    let gpu_ctx = match gpu::Context::new(
        gpu::NvDrvServiceKind::Applet,
        gpu::ViServiceKind::System,
        0x40000,
    ) {
        Ok(ctx) => ctx,
        Err(e) => panic!("{:?}", e),
    };

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

    let writer1_state = Arc::new((AtomicUsize::new(0), AtomicU8::new(0)));
    let writer2_state = Arc::new((AtomicUsize::new(0), AtomicU8::new(0)));
    let shared_state = Arc::new(RwLock::new(0u8));
    let reader1_state = Arc::new((AtomicUsize::new(0), AtomicU8::new(0)));
    let reader2_state = Arc::new((AtomicUsize::new(0), AtomicU8::new(0)));

    let kill = Arc::new(AtomicBool::new(false));
    let rand = if let Ok(rand) = new_service_object::<RandomService>() {
        Arc::new(rand)
    } else {
        return;
    };

    let _t1 = {
        let rand = rand.clone();
        let shared_state = shared_state.clone();
        let writer_state = writer1_state.clone();
        let kill = kill.clone();
        thread::spawn(move || {
            let mut sleep_time = [0u8; 8];
            let mut new_val: u8 = 0;
            while !kill.load(Ordering::Relaxed) {
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 100_000);

                let mut writer_handle = shared_state.write();
                rand.generate_random_bytes(Buffer::from_mut_var(&mut new_val))
                    .unwrap();
                *writer_handle = new_val;

                writer_state.0.fetch_add(1, Ordering::Release);
                writer_state.1.store(new_val, Ordering::Release);

                // sleep while holding write lock
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 10_000);
            }
        })
    };

    let _t2 = {
        let rand = rand.clone();
        let shared_state = shared_state.clone();
        let writer_state = writer2_state.clone();
        let kill = kill.clone();
        thread::spawn(move || {
            let mut sleep_time = [0u8; 8];
            let mut new_val: u8 = 0;
            while !kill.load(Ordering::Relaxed) {
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 100_000);

                let mut writer_handle = shared_state.write();
                rand.generate_random_bytes(Buffer::from_mut_var(&mut new_val))
                    .unwrap();
                *writer_handle = new_val;

                writer_state.0.fetch_add(1, Ordering::Release);
                writer_state.1.store(new_val, Ordering::Release);

                // sleep while holding write lock
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 10_000);
            }
        })
    };

    let _t3 = {
        let rand = rand.clone();
        let shared_state = shared_state.clone();
        let reader_state = reader1_state.clone();
        let kill = kill.clone();
        thread::spawn(move || {
            let mut sleep_time = [0u8; 8];
            while !kill.load(Ordering::Relaxed) {
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 500);

                let reader_handle = shared_state.read();

                reader_state.0.fetch_add(1, Ordering::Release);
                reader_state.1.store(*reader_handle, Ordering::Release);

                // sleep while holding read lock, less read time than write
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 100_000);
            }
        })
    };

    let _t4 = {
        let rand = rand.clone();
        let shared_state = shared_state.clone();
        let reader_state = reader2_state.clone();
        let kill = kill.clone();
        thread::spawn(move || {
            let mut sleep_time = [0u8; 8];
            while !kill.load(Ordering::Relaxed) {
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 500);

                let reader_handle = shared_state.read();

                reader_state.0.fetch_add(1, Ordering::Release);
                reader_state.1.store(*reader_handle, Ordering::Release);

                // sleep while holding read lock, less read time than write
                rand.generate_random_bytes(Buffer::from_mut_array(&mut sleep_time))
                    .unwrap();
                let _ = thread::sleep(i64::from_ne_bytes(sleep_time).abs() % 50_000);
            }
        })
    };

    'render: loop {
        for input_buttons in [hid::NpadIdType::Handheld, hid::NpadIdType::No1]
            .iter()
            .cloned()
            .map(|controller| input_ctx.get_player(controller).get_buttons_down())
        {
            if input_buttons.contains(hid::NpadButton::Plus()) {
                kill.store(true, Ordering::Release);
                // Exit if Plus/+ is pressed.
                break 'render;
            }
        }

        frame += 1;

        let _ = surface.render(Some(RGBType::new_scaled(255, 255, 255, 255)), |c| {
            c.draw_ascii_bitmap_text(
                format!("frame #: {}", frame),
                c_black,
                3,
                20,
                0,
                gpu::canvas::AlphaBlend::Destination,
            );

            c.draw_ascii_bitmap_text(
                format!(
                    "writer1_state: {} {}",
                    writer1_state.0.load(Ordering::Relaxed),
                    writer1_state.1.load(Ordering::Relaxed)
                ),
                c_black,
                3,
                20,
                100,
                gpu::canvas::AlphaBlend::Destination,
            );

            c.draw_ascii_bitmap_text(
                format!(
                    "writer2_state: {} {}",
                    writer2_state.0.load(Ordering::Relaxed),
                    writer2_state.1.load(Ordering::Relaxed)
                ),
                c_black,
                3,
                20,
                200,
                gpu::canvas::AlphaBlend::Destination,
            );

            c.draw_ascii_bitmap_text(
                format!(
                    "reader1_state: {} {}",
                    reader1_state.0.load(Ordering::Relaxed),
                    reader1_state.1.load(Ordering::Relaxed)
                ),
                c_black,
                3,
                20,
                300,
                gpu::canvas::AlphaBlend::Destination,
            );

            c.draw_ascii_bitmap_text(
                format!(
                    "reader2_state: {} {}",
                    reader2_state.0.load(Ordering::Relaxed),
                    reader2_state.1.load(Ordering::Relaxed)
                ),
                c_black,
                3,
                20,
                400,
                gpu::canvas::AlphaBlend::Destination,
            );

            Ok(())
        });
        let _ = surface.wait_vsync_event(None);
    }
}

#[panic_handler]
fn panic_handler(_info: &panic::PanicInfo) -> ! {
    nx::diag::abort::abort(abort::AbortLevel::Panic(), nx::rc::ResultPanicked::make())
}
