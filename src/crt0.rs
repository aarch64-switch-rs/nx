use crate::result::*;
use crate::results;
use crate::svc;
use crate::mem;
use crate::dynamic;
use crate::sync;
use crate::util;
use crate::hbl;
use crate::thread;
use crate::vmem;
use crate::version;
use crate::ipc::sf;
use crate::service;
use crate::service::set;
use crate::service::set::ISystemSettingsServer;
use core::ptr;

// These functions must be implemented by any executable homebrew project using this crate
extern "Rust" {
    fn __nx_internal_main() -> Result<()>;
    fn __nx_initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize;
}

pub type ExitFn = fn(ResultCode) -> !;

static mut G_EXIT_FN: sync::Locked<Option<ExitFn>> = sync::Locked::new(false, None);
static mut G_MAIN_THREAD: thread::Thread = thread::Thread::empty();

#[no_mangle]
#[linkage = "weak"]
unsafe fn __nx_crt0_entry(abi_ptr: *const hbl::AbiConfigEntry, raw_main_thread_handle: u64, aslr_base_address: *const u8, lr_exit_fn: ExitFn, bss_start: *mut u8, bss_end: *mut u8) {
    let is_hbl_nro = !abi_ptr.is_null() && (raw_main_thread_handle == u64::MAX);
    
    // Clear .bss section
    let bss_size = bss_end as usize - bss_start as usize;
    ptr::write_bytes(bss_start, 0, bss_size);

    // Relocate ourselves
    dynamic::relocate(aslr_base_address).unwrap();

    let mut heap = util::PointerAndSize::new(ptr::null_mut(), 0);
    let mut main_thread_handle = raw_main_thread_handle as svc::Handle;
    let mut hos_version = hbl::Version::empty();

    // If we are a NRO, parse the config entries hbloader sent us
    if is_hbl_nro {
        let mut abi_entry = abi_ptr;
        loop {
            match (*abi_entry).key {
                hbl::AbiConfigEntryKey::EndOfList => {
                    break;
                },
                hbl::AbiConfigEntryKey::OverrideHeap => {
                    heap.address = (*abi_entry).value[0] as *mut u8;
                    heap.size = (*abi_entry).value[1] as usize;
                },
                hbl::AbiConfigEntryKey::MainThreadHandle => {
                    main_thread_handle = (*abi_entry).value[0] as svc::Handle;
                },
                hbl::AbiConfigEntryKey::HosVersion => {
                    let hos_version_v = (*abi_entry).value[0] as u32;
                    hos_version = hbl::Version::new(hos_version_v);
                }
                _ => {
                    
                }
            }
            abi_entry = abi_entry.offset(1);
        }
    }

    // Initialize the main thread object and initialize its TLS section
    G_MAIN_THREAD = thread::Thread::existing(main_thread_handle, "MainThread", ptr::null_mut(), 0, false, None, ptr::null_mut()).unwrap();
    thread::set_current_thread(&mut G_MAIN_THREAD);

    // Initialize virtual memory
    vmem::initialize().unwrap();

    // Set exit function (will be null for non-hbl NROs)
    if is_hbl_nro {
        G_EXIT_FN.set(Some(lr_exit_fn));
    }
    else {
        G_EXIT_FN.set(None);
    }
    
    // Initialize heap and memory allocation
    heap = __nx_initialize_heap(heap);
    mem::initialize(heap.address, heap.size);

    // Initialize version support
    if hos_version.is_valid() {
        version::set_version(hos_version.to_version());
    }
    else {
        let setsys = service::new_service_object::<set::SystemSettingsServer>().unwrap();
        let fw_version: set::FirmwareVersion = Default::default();
        setsys.get().get_firmware_version(sf::Buffer::from_var(&fw_version)).unwrap();
        let version = version::Version::new(fw_version.major, fw_version.minor, fw_version.micro);
        version::set_version(version);
    }

    // TODO: finish implementing CRT0

    // Unwrap main(), which will trigger a panic if it didn't succeed
    __nx_internal_main().unwrap();

    // Successful exit by default
    exit(ResultSuccess::make());
}

#[no_mangle]
#[linkage = "weak"]
unsafe fn __nx_crt0_exception_entry(_error_desc: u32, _stack_top: *mut u8) {
    svc::return_from_exception(results::os::ResultUnhandledException::make());
}

pub fn exit(rc: ResultCode) -> ! {
    unsafe {
        match G_EXIT_FN.get() {
            Some(exit_fn) => exit_fn(rc),
            None => svc::exit_process()
        }
    }
}
