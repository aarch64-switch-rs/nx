use crate::result::*;
use core::ptr;
use core::mem;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum BreakReason {
    Panic = 0,
    Assert = 1,
    User = 2,
    PreLoadDll = 3,
    PostLoadDll = 4,
    PreUnloadDll = 5,
    PostUnloadDll = 6,
    CppException = 7,
    NotificationOnlyFlag = 0x80000000
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum MemoryState {
    Free = 0x0,
    Io = 0x1,
    Static = 0x2,
    Code = 0x3,
    CodeData = 0x4,
    Normal = 0x5,
    Shared = 0x6,
    Alias = 0x7,
    AliasCode = 0x8,
    AliasCodeData = 0x9,
    Ipc = 0xA,
    Stack = 0xB,
    ThreadLocal = 0xC,
    Transfered = 0xD,
    SharedTransfered = 0xE,
    SharedCode = 0xF,
    Inaccessible = 0x10,
    NonSecureIpc = 0x11,
    NonDeviceIpc = 0x12,
    Kernel = 0x13,
    GeneratedCode = 0x14,
    CodeOut = 0x15
}

bit_enum! {
    MemoryPermission (u32) {
        None = 0,
        Read = bit!(0),
        Write = bit!(1),
        Execute = bit!(2),
        DontCare = bit!(28)
    }
}

bit_enum! {
    MemoryAttribute (u32) {
        None = 0,
        Borrowed = bit!(0),
        IpcMapped = bit!(1),
        DeviceMapped = bit!(2),
        Uncached = bit!(3)
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MemoryInfo {
    pub base_address: *mut u8,
    pub size: u64,
    pub memory_state: MemoryState,
    pub memory_attribute: MemoryAttribute,
    pub memory_permission: MemoryPermission,
    pub ipc_refcount: u32,
    pub device_refcount: u32,
    pub pad: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum InfoId {
    CoreMask = 0,
    PriorityMask = 1,
    AliasRegionAddress = 2,
    AliasRegionSize = 3,
    HeapRegionAddress = 4,
    HeapRegionSize = 5,
    TotalMemorySize = 6,
    UsedMemorySize = 7,
    DebuggerAttached = 8,
    ResourceLimit = 9,
    IdleTickCount = 10,
    RandomEntropy = 11,
    AslrRegionAddress = 12,
    AslrRegionSize = 13,
    StackRegionAddress = 14,
    StackRegionSize = 15,
    SystemResourceSizeTotal = 16,
    SystemResourceSizeUsed = 17,
    ProgramId = 18,
    InitialProcessIdRange = 19,
    UserExceptionContextAddress = 20,
    TotalNonSystemMemorySize = 21,
    UsedNonSystemMemorySize = 22,
    IsApplication = 23,
}

pub type PageInfo = u32;
pub type Address = *const u8;
pub type Size = usize;
pub type ThreadEntrypointFn = extern fn(*mut u8) -> !;
pub type Handle = u32;

pub const CURRENT_THREAD_PSEUDO_HANDLE: Handle = 0xFFFF8000;
pub const CURRENT_PROCESS_PSEUDO_HANDLE: Handle = 0xFFFF8001;

pub fn set_heap_size(size: Size) -> Result<*mut u8> {
    extern "C" {
        fn __nx_svc_set_heap_size(out_address: *mut *mut u8, size: Size) -> ResultCode;
    }

    unsafe {
        let mut address: *mut u8 = ptr::null_mut();

        let rc = __nx_svc_set_heap_size(&mut address, size);
        wrap(rc, address)
    }
}

pub fn set_memory_attribute(address: Address, size: Size, mask: u32, value: MemoryAttribute) -> Result<()> {
    extern "C" {
        fn __nx_svc_set_memory_attribute(address: Address, size: Size, mask: u32, value: MemoryAttribute) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_set_memory_attribute(address, size, mask, value);
        wrap(rc, ())
    }
}

pub fn query_memory(address: *const u8) -> Result<(MemoryInfo, PageInfo)> {
    extern "C" {
        fn __nx_svc_query_memory(out_info: *mut MemoryInfo, out_page_info: *mut PageInfo, address: *const u8) -> ResultCode;
    }

    unsafe {
        let mut memory_info: MemoryInfo = mem::zeroed();
        let mut page_info: PageInfo = 0;

        let rc = __nx_svc_query_memory(&mut memory_info, &mut page_info, address);
        wrap(rc, (memory_info, page_info))
    }
}

pub fn exit_process() -> ! {
    extern "C" {
        fn __nx_svc_exit_process() -> !;
    }

    unsafe {
        __nx_svc_exit_process()
    }
}

pub fn create_thread(entry: ThreadEntrypointFn, entry_arg: Address, stack_top: Address, priority: i32, cpu_id: i32) -> Result<Handle> {
    extern "C" {
        fn __nx_svc_create_thread(handle: *mut Handle, entry: ThreadEntrypointFn, entry_arg: Address, stack_top: Address, priority: i32, cpu_id: i32) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_create_thread(&mut handle, entry, entry_arg, stack_top, priority, cpu_id);
        wrap(rc, handle)
    }
}

pub fn start_thread(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_start_thread(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_start_thread(handle);
        wrap(rc, ())
    }
}

pub fn exit_thread() -> ! {
    extern "C" {
        fn __nx_svc_exit_thread() -> !;
    }

    unsafe {
        __nx_svc_exit_thread()
    }
}

pub fn sleep_thread(timeout: i64) -> Result<()> {
    extern "C" {
        fn __nx_svc_sleep_thread(timeout: i64) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_sleep_thread(timeout);
        wrap(rc, ())
    }
}

pub fn get_thread_priority(handle: Handle) -> Result<i32> {
    extern "C" {
        fn __nx_svc_get_thread_priority(out_priority: *mut i32, handle: Handle) -> ResultCode;
    }

    unsafe {
        let mut priority: i32 = 0;

        let rc = __nx_svc_get_thread_priority(&mut priority, handle);
        wrap(rc, priority)
    }
}

pub fn signal_event(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_signal_event(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_signal_event(handle);
        wrap(rc, ())
    }
}

pub fn map_shared_memory(handle: Handle, address: Address, size: Size, permission: MemoryPermission) -> Result<()> {
    extern "C" {
        fn __nx_svc_map_shared_memory(handle: Handle, address: Address, size: Size, permission: MemoryPermission) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_map_shared_memory(handle, address, size, permission);
        wrap(rc, ())
    }
}

pub fn unmap_shared_memory(handle: Handle, address: Address, size: Size) -> Result<()> {
    extern "C" {
        fn __nx_svc_unmap_shared_memory(handle: Handle, address: Address, size: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_unmap_shared_memory(handle, address, size);
        wrap(rc, ())
    }
}

pub fn create_transfer_memory(address: Address, size: Size, permissions: MemoryPermission) -> Result<Handle> {
    extern "C" {
        fn __nx_svc_create_transfer_memory(out_handle: *mut Handle, address: Address, size: Size, permissions: MemoryPermission) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_create_transfer_memory(&mut handle, address, size, permissions);
        wrap(rc, handle)
    }
}

pub fn close_handle(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_close_handle(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_close_handle(handle);
        wrap(rc, ())
    }
}

pub fn reset_signal(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_reset_signal(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_reset_signal(handle);
        wrap(rc, ())
    }
}

pub fn wait_synchronization(handles: *const Handle, handle_count: u32, timeout: i64) -> Result<i32> {
    extern "C" {
        fn __nx_svc_wait_synchronization(out_index: *mut i32, handles: *const Handle, handle_count: u32, timeout: i64) -> ResultCode;
    }

    unsafe {
        let mut index: i32 = 0;

        let rc = __nx_svc_wait_synchronization(&mut index, handles, handle_count, timeout);
        wrap(rc, index)
    }
}

pub fn arbitrate_lock(thread_handle: Handle, tag_location: Address, tag: u32) -> Result<()> {
    extern "C" {
        fn __nx_svc_arbitrate_lock(thread_handle: Handle, tag_location: Address, tag: u32) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_arbitrate_lock(thread_handle, tag_location, tag);
        wrap(rc, ())
    }
}

pub fn arbitrate_unlock(tag_location: Address) -> Result<()> {
    extern "C" {
        fn __nx_svc_arbitrate_unlock(tag_location: Address) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_arbitrate_unlock(tag_location);
        wrap(rc, ())
    }
}

pub fn connect_to_named_port(name: Address) -> Result<Handle> {
    extern "C" {
        fn __nx_svc_connect_to_named_port(out_handle: *mut Handle, name: Address) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_connect_to_named_port(&mut handle, name);
        wrap(rc, handle)
    }
}

pub fn send_sync_request(handle: Handle) -> Result<()> {
    extern "C" {
        fn __nx_svc_send_sync_request(handle: Handle) -> ResultCode;
    }
    
    unsafe {
        let rc = __nx_svc_send_sync_request(handle);
        wrap(rc, ())
    }
}

#[inline(always)]
pub fn get_process_id(process_handle: Handle) -> Result<u64> {
    extern "C" {
        fn __nx_svc_get_process_id(out_process_id: *mut u64, process_handle: Handle) -> ResultCode;
    }
    
    unsafe {
        let mut process_id: u64 = 0;

        let rc = __nx_svc_get_process_id(&mut process_id, process_handle);
        wrap(rc, process_id)
    }
}

#[inline(always)]
pub fn get_thread_id(process_handle: Handle) -> Result<u64> {
    extern "C" {
        fn __nx_svc_get_thread_id(out_thread_id: *mut u64, process_handle: Handle) -> ResultCode;
    }
    
    unsafe {
        let mut thread_id: u64 = 0;

        let rc = __nx_svc_get_thread_id(&mut thread_id, process_handle);
        wrap(rc, thread_id)
    }
}

// Note: original name is just break/Break, but that's a reserved keyword :P

#[inline(always)]
pub fn break_(reason: BreakReason, arg: Address, size: Size) -> Result<()> {
    extern "C" {
        fn __nx_svc_break(reason: BreakReason, arg: Address, size: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_break(reason, arg, size);
        wrap(rc, ())
    }
}

#[inline(always)]
pub fn output_debug_string(msg: Address, len: Size) -> Result<()> {
    extern "C" {
        fn __nx_svc_output_debug_string(msg: Address, len: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_output_debug_string(msg, len);
        wrap(rc, ())
    }
}

#[inline(always)]
pub fn return_from_exception(res: ResultCode) {
    extern "C" {
        fn __nx_svc_return_from_exception(res: ResultCode);
    }

    unsafe {
        __nx_svc_return_from_exception(res);
    }
}

#[inline(always)]
pub fn get_info(id: InfoId, handle: Handle, sub_id: u64) -> Result<u64> {
    extern "C" {
        fn __nx_svc_get_info(out_info: *mut u64, id: InfoId, handle: Handle, sub_id: u64) -> ResultCode;
    }
    
    unsafe {
        let mut info: u64 = 0;

        let rc = __nx_svc_get_info(&mut info, id, handle, sub_id);
        wrap(rc, info)
    }
}

pub fn create_session(is_light: bool, unk_name: u64) -> Result<(Handle, Handle)> {
    extern "C" {
        fn __nx_svc_create_session(out_server_handle: *mut Handle, out_client_handle: *mut Handle, is_light: bool, unk_name: u64) -> ResultCode;
    }

    unsafe {
        let mut server_handle: Handle = 0;
        let mut client_handle: Handle = 0;

        let rc = __nx_svc_create_session(&mut server_handle, &mut client_handle, is_light, unk_name);
        wrap(rc, (server_handle, client_handle))
    }
}

pub fn accept_session(handle: Handle) -> Result<Handle> {
    extern "C" {
        fn __nx_svc_accept_session(out_session_handle: *mut Handle, handle: Handle) -> ResultCode;
    }

    unsafe {
        let mut session_handle: Handle = 0;
        
        let rc = __nx_svc_accept_session(&mut session_handle, handle);
        wrap(rc, session_handle)
    }
}

pub fn reply_and_receive(handles: *const Handle, handle_count: u32, reply_target: Handle, timeout: i64) -> Result<i32> {
    extern "C" {
        fn __nx_svc_reply_and_receive(out_index: *mut i32, handles: *const Handle, handle_count: u32, reply_target: Handle, timeout: i64) -> ResultCode;
    }

    unsafe {
        let mut index: i32 = 0;

        let rc = __nx_svc_reply_and_receive(&mut index, handles, handle_count, reply_target, timeout);
        wrap(rc, index)
    }
}

pub fn create_event() -> Result<(Handle, Handle)> {
    extern "C" {
        fn __nx_svc_create_event(out_server_handle: *mut Handle, out_client_handle: *mut Handle) -> ResultCode;
    }

    unsafe {
        let mut server_handle: Handle = 0;
        let mut client_handle: Handle = 0;

        let rc = __nx_svc_create_event(&mut server_handle, &mut client_handle);
        wrap(rc, (server_handle, client_handle))
    }
}

pub fn manage_named_port(name: Address, max_sessions: i32) -> Result<Handle> {
    extern "C" {
        fn __nx_svc_manage_named_port(out_handle: *mut Handle, name: Address, max_sessions: i32) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_manage_named_port(&mut handle, name, max_sessions);
        wrap(rc, handle)
    }
}