//! Initial code/entrypoint support and utils
//! 
//! # Custom entrypoint
//! 
//! If you wish to define your custom entrypoint, you can do so by redefining the `__nx_rrt0_entry` weak fn.
//! 
//! Example (check [here](https://switchbrew.org/wiki/Homebrew_ABI#Entrypoint_Arguments) for more entrypoint details):
//! ```
//! #[no_mangle]
//! unsafe extern "C" fn __nx_rrt0_entry(arg0: usize, arg1: usize) {
//!     // ...
//! }
//! ```
//! 
//! # Custom version setup
//! 
//! On the default entrypoint routine, the internal system version (see [`get_version`][`version::get_version`] and [`set_version`][`version::set_version`]) gets set the following way:
//! * If the process was launched through HBL, use the "HOS version" value we got from it
//! * Otherwise (and if using the `services` feature), use settings services ([`SystemSettingsServer`][`crate::service::set::SystemSettingsServer`]) to get it
//! 
//! If you wish to define your custom version setup (for instance, in contexts in which you wish to avoid executing the aforementioned setup), you can do so by redefining the `initialize_version` weak fn.
//! 
//! Example:
//! ```
//! #[no_mangle]
//! fn initialize_version(hbl_hos_version_opt: Option<hbl::Version>) {
//!     // ...
//! }
//! ```

use crate::elf;
use crate::result::*;
use crate::svc;
use crate::mem::alloc;
use crate::svc::Handle;
use crate::sync;
use crate::util;
use crate::hbl;
use crate::thread;
use crate::vmem;
use crate::version;

#[cfg(feature = "services")]
use crate::ipc::sf;

#[cfg(feature = "services")]
use crate::service;

#[cfg(feature = "services")]
use crate::service::set;

#[cfg(feature = "services")]
use crate::service::set::ISystemSettingsServer;

use core::ptr;
use core::mem;
use core::arch::asm;

// These functions must be implemented by any binary using this crate

extern "Rust" {
    fn main() -> Result<()>;
    fn initialize_heap(hbl_heap: util::PointerAndSize) -> util::PointerAndSize;
}

/// Represents the fn pointer used for exiting
pub type ExitFn = extern "C" fn(ResultCode) -> !;

/// Represents the executable type of the current process
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum ExecutableType {
    #[default]
    None,
    Nso,
    Nro
}

static mut G_EXECUTABLE_TYPE: ExecutableType = ExecutableType::None;

pub(crate) fn set_executable_type(exec_type: ExecutableType) {
    unsafe {
        G_EXECUTABLE_TYPE = exec_type;
    }
}

/// Gets the current process's executable type
/// 
/// Note that this can be used to determine if this process was launched through HBL or not (if so, we would be a homebrew NRO and this would return [`ExecutableType::Nro`])
pub fn get_executable_type() -> ExecutableType {
    unsafe {
        G_EXECUTABLE_TYPE
    }
}

/// Represents the process module format used by processes
/// 
/// This layout has to be present at the start of the process's `.rodata` section, containing its module name
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ModulePath {
    /// Unused value
    pub zero: u32,
    /// The length of the module name
    pub path_len: u32,
    /// The module name string
    pub path: util::CString<0x200>
}

impl ModulePath {
    /// Creates a [`ModulePath`] with the given module name
    /// 
    /// # Arguments
    /// 
    /// * `name`: The module name
    #[inline]
    pub const fn new(name: &str) -> Self {
        Self {
            zero: 0,
            path_len: name.len() as u32,
            path: util::CString::from_str(name)
        }
    }
}

#[no_mangle]
#[used]
#[linkage = "weak"]
#[link_section = ".module_name"]
#[export_name = "__nx_rrt0_module_name"]
static G_MODULE_NAME: ModulePath = ModulePath::new("aarch64-switch-rs (unknown module)");

/// Gets this process's module name
/// 
/// The module name is `aarch64-switch-rs (unknown module)` by default, but it can be set to a custom one with [`rrt0_define_module_name`] or [`rrt0_define_default_module_name`] macros
pub fn get_module_name() -> ModulePath {
    G_MODULE_NAME
}

static mut G_EXIT_FN: sync::Locked<Option<ExitFn>> = sync::Locked::new(false, None);
static mut G_MAIN_THREAD: thread::Thread = thread::Thread::empty();

/// Exits the current process
/// 
/// This will call the HBL-specific exit fn if running as a homebrew NRO, or [`exit_process`][`svc::exit_process`] otherwise
pub fn exit(rc: ResultCode) -> ! {
    unsafe {
        match G_EXIT_FN.get() {
            Some(exit_fn) => exit_fn(rc),
            None => svc::exit_process()
        }
    }
}

// TODO: consider adding a default heap-init function?

#[no_mangle]
#[linkage = "weak"]
fn initialize_version(hbl_hos_version_opt: Option<hbl::Version>) {
    if let Some(hbl_hos_version) = hbl_hos_version_opt {
        version::set_version(hbl_hos_version.to_version());
    }
    else {
        #[cfg(feature = "services")]
        {
            let set_sys = service::new_service_object::<set::SystemSettingsServer>().unwrap();
            let fw_version: set::FirmwareVersion = Default::default();
            set_sys.get().get_firmware_version(sf::Buffer::from_var(&fw_version)).unwrap();

            let version = version::Version::new(fw_version.major, fw_version.minor, fw_version.micro);
            version::set_version(version);
        }
    }
}

unsafe fn normal_entry(maybe_abi_cfg_entries_ptr: *const hbl::AbiConfigEntry, maybe_main_thread_handle: usize, lr_exit_fn: ExitFn) {
    let exec_type = match !maybe_abi_cfg_entries_ptr.is_null() && (maybe_main_thread_handle == usize::MAX) {
        true => ExecutableType::Nro,
        false => ExecutableType::Nso
    };
    set_executable_type(exec_type);

    let mut heap = util::PointerAndSize::new(ptr::null_mut(), 0);
    let mut main_thread_handle = maybe_main_thread_handle as svc::Handle;
    let mut hos_version_opt: Option<hbl::Version> = None;

    // If we are a NRO, parse the config entries hbloader sent us
    if exec_type == ExecutableType::Nro {
        let mut abi_entry = maybe_abi_cfg_entries_ptr;
        loop {
            match (*abi_entry).key {
                hbl::AbiConfigEntryKey::EndOfList => {
                    let loader_info_data = (*abi_entry).value[0] as *mut u8;
                    let loader_info_data_len = (*abi_entry).value[1] as usize;
                    if loader_info_data_len > 0 {
                        let loader_info_slice = core::slice::from_raw_parts(loader_info_data, loader_info_data_len);
                        if let Ok(loader_info) = core::str::from_utf8(loader_info_slice) {
                            hbl::set_loader_info(loader_info);
                        }
                    }
                    break;
                },
                hbl::AbiConfigEntryKey::MainThreadHandle => {
                    main_thread_handle = (*abi_entry).value[0] as svc::Handle;
                },
                hbl::AbiConfigEntryKey::NextLoadPath => {
                    let next_load_path_data = (*abi_entry).value[0] as *mut u8;
                    let next_load_path_data_len = util::str_ptr_len(next_load_path_data as *const u8);
                    let next_load_argv_data = (*abi_entry).value[1] as *mut u8;
                    let next_load_argv_data_len = util::str_ptr_len(next_load_argv_data as *const u8);
                    
                    let next_load_path_slice = core::slice::from_raw_parts(next_load_path_data, next_load_path_data_len);
                    let next_load_argv_slice = core::slice::from_raw_parts(next_load_argv_data, next_load_argv_data_len);
                    if let Ok(next_load_path) = core::str::from_utf8(next_load_path_slice) {
                        if let Ok(next_load_argv) = core::str::from_utf8(next_load_argv_slice) {
                            hbl::set_next_load_entry_ptr(next_load_path, next_load_argv);
                        }
                    }
                },
                hbl::AbiConfigEntryKey::OverrideHeap => {
                    heap.address = (*abi_entry).value[0] as *mut u8;
                    heap.size = (*abi_entry).value[1] as usize;
                },
                hbl::AbiConfigEntryKey::OverrideService => {
                    // todo!("OverrideService");
                },
                hbl::AbiConfigEntryKey::Argv => {
                    // todo!("Argv");
                },
                hbl::AbiConfigEntryKey::SyscallAvailableHint => {
                    // todo!("SyscallAvailableHint");
                },
                hbl::AbiConfigEntryKey::AppletType => {
                    let applet_type: hbl::AppletType = mem::transmute((*abi_entry).value[0] as u32);
                    hbl::set_applet_type(applet_type);
                },
                hbl::AbiConfigEntryKey::ProcessHandle => {
                    let proc_handle = (*abi_entry).value[0] as Handle;
                    hbl::set_process_handle(proc_handle);
                },
                hbl::AbiConfigEntryKey::LastLoadResult => {
                    let last_load_rc = ResultCode::new((*abi_entry).value[0] as u32);
                    hbl::set_last_load_result(last_load_rc);
                },
                hbl::AbiConfigEntryKey::RandomSeed => {
                    let random_seed = ((*abi_entry).value[0], (*abi_entry).value[1]);
                    hbl::set_random_seed(random_seed);
                },
                hbl::AbiConfigEntryKey::UserIdStorage => {
                    // todo!("UserIdStorage");
                },
                hbl::AbiConfigEntryKey::HosVersion => {
                    let hos_version_v = (*abi_entry).value[0] as u32;
                    let os_impl_magic = (*abi_entry).value[1];
                    hos_version_opt = Some(hbl::Version::new(hos_version_v, os_impl_magic));
                },
                _ => {
                    // TODO: invalid config entries?
                }
            }
            abi_entry = abi_entry.offset(1);
        }
    }

    // Initialize the main thread object and initialize its TLS section
    // TODO: query memory for main thread stack address/size?
    
    G_MAIN_THREAD = thread::Thread::new_remote(main_thread_handle, "MainThread", ptr::null_mut(), 0).unwrap();
    thread::set_current_thread(ptr::addr_of_mut!(G_MAIN_THREAD));

    // Initialize virtual memory
    vmem::initialize().unwrap();

    // Set exit function (will be null for non-hbl NROs)
    match exec_type {
        ExecutableType::Nro => G_EXIT_FN.set(Some(lr_exit_fn)),
        ExecutableType::Nso => G_EXIT_FN.set(None),
        _ => {}
    };
    
    // Initialize heap and memory allocation
    heap = initialize_heap(heap);
    alloc::initialize(heap);

    // Initialize version support
    initialize_version(hos_version_opt);

    // TODO: extend this (init more stuff, etc.)?

    // Unwrap main(), which will trigger a panic if it didn't succeed
    main().unwrap();

    // Successful exit by default
    exit(ResultSuccess::make());
}

unsafe fn exception_entry(_exc_type: svc::ExceptionType, _stack_top: *mut u8) {
    // TODO: user exception handler?
    svc::return_from_exception(svc::rc::ResultNotHandled::make());
}

#[no_mangle]
#[linkage = "weak"]
unsafe extern "C" fn __nx_rrt0_entry(arg0: usize, arg1: usize) {
    let lr_exit_fn: ExitFn;
    asm!(
        "mov {}, lr",
        out(reg) lr_exit_fn
    );

    /*
    Possible entry arguments:
    - NSO/KIP: x0 = 0, x1 = <main-thread-handle>
    - NRO (hbl): x0 = <abi-config-entries-ptr>, x1 = usize::MAX
    - Exception: x0 = <exception-type>, x1 = <stack-top>
    */

    if (arg0 != 0) && (arg1 != usize::MAX) {
        // Handle exception entry
        let exc_type: svc::ExceptionType = mem::transmute(arg0 as u32);
        let stack_top = arg1 as *mut u8;
        exception_entry(exc_type, stack_top);
    }
    else {
        // We actually want `_start` which is at the start of the .text region, but we don't know if
        // it will be close enough to support lookup via `adr`.
        // Since this function is in `.text` anyway, use QueryMemory SVC to find the actual start
        let self_base_address: *const u8;
        asm!(
            "adr {}, __nx_rrt0_entry",
            out(reg) self_base_address
        );

        let (info, _) = svc::query_memory(self_base_address).unwrap();
        let aslr_base_address = info.base_address as *const u8;

        let start_dyn = elf::mod0::find_start_dyn_address(aslr_base_address).unwrap();
        elf::relocate_with_dyn(aslr_base_address, start_dyn as *const elf::Dyn).unwrap();

        extern "C" {
            static mut __bss_start: u8;
            static mut __bss_end: u8;
        }

        let bss_start_addr = ptr::addr_of_mut!(__bss_start) as *const _ as *mut u64;
        let bss_end_addr = ptr::addr_of_mut!(__bss_end) as *const _ as *mut u64;

        let mut cur_bss_addr = bss_start_addr;
        while cur_bss_addr < bss_end_addr {
            ptr::write_volatile(cur_bss_addr, 0);
            cur_bss_addr = cur_bss_addr.add(1);
        }

        // Handle NSO/KIP/NRO normal entry
        let maybe_abi_cfg_entries_ptr = arg0 as *const hbl::AbiConfigEntry;
        let maybe_main_thread_handle = arg1;
        normal_entry(maybe_abi_cfg_entries_ptr, maybe_main_thread_handle, lr_exit_fn);
    }
}