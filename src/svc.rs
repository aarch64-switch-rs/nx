//! This module wraps svc calls provided by `svc.s`.
//! There is no function-level Safety docs, but the core requirement is that all raw pointers provided must be
//! validated by the caller.

use crate::arm;
use crate::ipc::sf::ncm;
use crate::result::*;
use crate::util;
use crate::version;
use core::mem;
use core::mem::ManuallyDrop;
use core::ptr;

pub mod rc;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ArbitrationType {
    WaitIfLessThan = 0,
    DecrementAndWaitIfLessThan = 1,
    WaitIfEqual = 2,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum SignalType {
    Signal = 0,
    SignalAndIncrementIfEqual = 1,
    SignalAndModifyBasedOnWaitingThreadCountIfEqual = 2,
}

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
    NotificationOnlyFlag = 0x80000000,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u32)]
pub enum MemoryState {
    #[default]
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
    Transferred = 0xD,
    SharedTransferred = 0xE,
    SharedCode = 0xF,
    Inaccessible = 0x10,
    NonSecureIpc = 0x11,
    NonDeviceIpc = 0x12,
    Kernel = 0x13,
    GeneratedCode = 0x14,
    CodeOut = 0x15,
}

define_bit_enum! {
    MemoryPermission (u32) {
        None = 0,
        Read = bit!(0),
        Write = bit!(1),
        Execute = bit!(2),
        DontCare = bit!(28)
    }
}

define_bit_enum! {
    MemoryAttribute (u32) {
        None = 0,
        Borrowed = bit!(0),
        IpcMapped = bit!(1),
        DeviceMapped = bit!(2),
        Uncached = bit!(3)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct MemoryInfo {
    pub base_address: usize,
    pub size: usize,
    pub state: MemoryState,
    pub attribute: MemoryAttribute,
    pub permission: MemoryPermission,
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

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct AttachProcessDebugEventInfo {
    pub program_id: ncm::ProgramId,
    pub process_id: u64,
    pub name: util::ArrayString<12>,
    pub flags: u32,
    pub user_exception_context_address: u64,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct AttachThreadDebugEventInfo {
    pub thread_id: u64,
    pub tls_ptr: usize,
    pub entrypoint: usize,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ExitDebugEventInfo {
    pub exit_type: u32,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ExceptionDebugEventInfo {
    pub exception_type: u32,
    pub fault_register: u32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub union DebugEventInfo {
    pub attach_process: AttachProcessDebugEventInfo,
    pub attach_thread: AttachThreadDebugEventInfo,
    pub exit_process: ExitDebugEventInfo,
    pub exit_thread: ExitDebugEventInfo,
    pub exception: ExceptionDebugEventInfo,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum DebugEventType {
    AttachProcess,
    AttachThread,
    ExitProcess,
    ExitThread,
    Exception,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct DebugEvent {
    pub event_type: DebugEventType,
    pub flags: u32,
    pub thread_id: u32,
    pub info: DebugEventInfo,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ExceptionType {
    Init = 0x000,
    InstructionAbort = 0x100,
    DataAbort = 0x101,
    UnalignedInstruction = 0x102,
    UnalignedData = 0x103,
    UndefinedInstruction = 0x104,
    ExceptionInstruction = 0x105,
    MemorySystemError = 0x106,
    FpuException = 0x200,
    InvalidSystemCall = 0x301,
    SystemCallBreak = 0x302,
    AtmosphereStdAbort = 0xFFE,
}

pub type PageInfo = u32;
pub type Address = *const u8;
pub type Size = usize;
pub type ThreadEntrypointFn = unsafe extern "C" fn(*mut u8) -> !;
pub type Handle = u32;

pub struct ScopedHandle(pub Handle);
impl ScopedHandle {
    /// Creates a scoped guard for the handle.
    /// The handle can still be accessed and copied, but will become invalid when this struct is dropped.
    pub fn guard(handle: Handle) -> Self {
        Self(handle)
    }

    // Take the value out without running the destructor and closing the handle, consuming the guard
    pub unsafe fn take(guard: Self) -> Handle {
        ManuallyDrop::new(guard).0
    }
}

impl Drop for ScopedHandle {
    fn drop(&mut self) {
        if self.0 != INVALID_HANDLE {
            // ignore the error as it will only happen if the handle has already become invalid.
            let _ = close_handle(self.0);
        }
    }
}

pub const INVALID_HANDLE: Handle = 0;

pub const CURRENT_THREAD_PSEUDO_HANDLE: Handle = 0xFFFF8000;
pub const CURRENT_PROCESS_PSEUDO_HANDLE: Handle = 0xFFFF8001;

pub const DEFAULT_PROCESS_PROCESSOR_ID: i32 = -2;

#[inline(always)]
pub fn set_heap_size(size: Size) -> Result<*mut u8> {
    unsafe extern "C" {
        fn __nx_svc_set_heap_size(out_address: *mut *mut u8, size: Size) -> ResultCode;
    }

    unsafe {
        let mut address: *mut u8 = ptr::null_mut();

        let rc = __nx_svc_set_heap_size(&mut address, size);
        pack(rc, address)
    }
}

#[inline(always)]
pub unsafe fn set_memory_attribute(
    address: Address,
    size: Size,
    mask: u32,
    value: MemoryAttribute,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_set_memory_attribute(
            address: Address,
            size: Size,
            mask: u32,
            value: MemoryAttribute,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_set_memory_attribute(address, size, mask, value);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn set_memory_permission(
    address: Address,
    size: Size,
    value: MemoryPermission,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_set_memory_permission(
            address: Address,
            size: Size,
            value: MemoryPermission,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_set_memory_permission(address, size, value);
        pack(rc, ())
    }
}

#[inline(always)]
// SAFETY: null pointers are OK here, as we are just querying memory properties
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn query_memory(address: Address) -> Result<(MemoryInfo, PageInfo)> {
    unsafe extern "C" {
        fn __nx_svc_query_memory(
            out_info: *mut MemoryInfo,
            out_page_info: *mut PageInfo,
            address: Address,
        ) -> ResultCode;
    }

    unsafe {
        let mut memory_info: MemoryInfo = Default::default();
        let mut page_info: PageInfo = 0;

        let rc = __nx_svc_query_memory(&mut memory_info, &mut page_info, address);
        pack(rc, (memory_info, page_info))
    }
}

#[inline(always)]
pub fn exit_process() -> ! {
    unsafe extern "C" {
        fn __nx_svc_exit_process() -> !;
    }

    unsafe { __nx_svc_exit_process() }
}

#[inline(always)]
pub unsafe fn create_thread(
    entry: ThreadEntrypointFn,
    entry_arg: Address,
    stack_top: Address,
    priority: i32,
    processor_id: i32,
) -> Result<Handle> {
    unsafe extern "C" {
        fn __nx_svc_create_thread(
            handle: *mut Handle,
            entry: ThreadEntrypointFn,
            entry_arg: Address,
            stack_top: Address,
            priority: i32,
            processor_id: i32,
        ) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_create_thread(
            &mut handle,
            entry,
            entry_arg,
            stack_top,
            priority,
            processor_id,
        );
        pack(rc, handle)
    }
}

#[inline(always)]
pub fn start_thread(handle: Handle) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_start_thread(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_start_thread(handle);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn exit_thread() -> ! {
    unsafe extern "C" {
        fn __nx_svc_exit_thread() -> !;
    }

    unsafe { __nx_svc_exit_thread() }
}

#[inline(always)]
pub fn sleep_thread(timeout: i64) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_sleep_thread(timeout: i64) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_sleep_thread(timeout);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn get_thread_priority(handle: Handle) -> Result<i32> {
    unsafe extern "C" {
        fn __nx_svc_get_thread_priority(out_priority: *mut i32, handle: Handle) -> ResultCode;
    }

    unsafe {
        let mut priority: i32 = 0;

        let rc = __nx_svc_get_thread_priority(&mut priority, handle);
        pack(rc, priority)
    }
}

#[inline(always)]
pub fn set_thread_priority(handle: Handle, priority: i32) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_set_thread_priority(handle: Handle, priority: i32) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_set_thread_priority(handle, priority);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn get_current_processor_number() -> u32 {
    unsafe extern "C" {
        fn __nx_svc_get_current_processor_number() -> u32;
    }

    unsafe { __nx_svc_get_current_processor_number() }
}

#[inline(always)]
pub fn signal_event(handle: Handle) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_signal_event(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_signal_event(handle);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn map_shared_memory(
    handle: Handle,
    address: Address,
    size: Size,
    permission: MemoryPermission,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_map_shared_memory(
            handle: Handle,
            address: Address,
            size: Size,
            permission: MemoryPermission,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_map_shared_memory(handle, address, size, permission);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn unmap_shared_memory(handle: Handle, address: Address, size: Size) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_unmap_shared_memory(handle: Handle, address: Address, size: Size)
        -> ResultCode;
    }
    unsafe {
        let rc = __nx_svc_unmap_shared_memory(handle, address, size);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn create_transfer_memory(
    address: Address,
    size: Size,
    permissions: MemoryPermission,
) -> Result<Handle> {
    unsafe extern "C" {
        fn __nx_svc_create_transfer_memory(
            out_handle: *mut Handle,
            address: Address,
            size: Size,
            permissions: MemoryPermission,
        ) -> ResultCode;
    }
    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_create_transfer_memory(&mut handle, address, size, permissions);
        pack(rc, handle)
    }
}

#[inline(always)]
pub fn close_handle(handle: Handle) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_close_handle(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_close_handle(handle);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn reset_signal(handle: Handle) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_reset_signal(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_reset_signal(handle);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn wait_synchronization(
    handles: *const Handle,
    handle_count: u32,
    timeout: i64,
) -> Result<i32> {
    unsafe extern "C" {
        fn __nx_svc_wait_synchronization(
            out_index: *mut i32,
            handles: *const Handle,
            handle_count: u32,
            timeout: i64,
        ) -> ResultCode;
    }

    unsafe {
        let mut index: i32 = 0;

        let rc = __nx_svc_wait_synchronization(&mut index, handles, handle_count, timeout);
        pack(rc, index)
    }
}

#[inline(always)]
pub unsafe fn arbitrate_lock(thread_handle: Handle, tag_location: Address, tag: u32) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_arbitrate_lock(
            thread_handle: Handle,
            tag_location: Address,
            tag: u32,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_arbitrate_lock(thread_handle, tag_location, tag);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn arbitrate_unlock(tag_location: Address) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_arbitrate_unlock(tag_location: Address) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_arbitrate_unlock(tag_location);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn wait_process_wide_key_atomic(
    wait_location: Address,
    tag_location: Address,
    desired_tag: u32,
    timeout: i64,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_wait_process_wide_key_atomic(
            wait_location: Address,
            tag_location: Address,
            desired_tag: u32,
            timeout: i64,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_wait_process_wide_key_atomic(
            wait_location,
            tag_location,
            desired_tag,
            timeout,
        );
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn signal_process_wide_key(tag_location: Address, desired_tag: i32) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_signal_process_wide_key(tag_location: Address, desired_tag: i32) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_signal_process_wide_key(tag_location, desired_tag);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn get_system_tick() -> u64 {
    unsafe extern "C" {
        fn __nx_svc_get_system_tick() -> u64;
    }

    unsafe { __nx_svc_get_system_tick() }
}

#[inline(always)]
pub unsafe fn connect_to_named_port(name: Address) -> Result<Handle> {
    unsafe extern "C" {
        fn __nx_svc_connect_to_named_port(out_handle: *mut Handle, name: Address) -> ResultCode;
    }
    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_connect_to_named_port(&mut handle, name);
        pack(rc, handle)
    }
}

#[inline(always)]
pub fn send_sync_request(handle: Handle) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_send_sync_request(handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_send_sync_request(handle);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn get_process_id(process_handle: Handle) -> Result<u64> {
    unsafe extern "C" {
        fn __nx_svc_get_process_id(out_process_id: *mut u64, process_handle: Handle) -> ResultCode;
    }

    unsafe {
        let mut process_id: u64 = 0;

        let rc = __nx_svc_get_process_id(&mut process_id, process_handle);
        pack(rc, process_id)
    }
}

#[inline(always)]
pub fn get_thread_id(handle: Handle) -> Result<u64> {
    unsafe extern "C" {
        fn __nx_svc_get_thread_id(out_thread_id: *mut u64, handle: Handle) -> ResultCode;
    }

    unsafe {
        let mut thread_id: u64 = 0;

        let rc = __nx_svc_get_thread_id(&mut thread_id, handle);
        pack(rc, thread_id)
    }
}

#[inline(always)]
pub unsafe fn r#break(reason: BreakReason, arg: Address, size: Size) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_break(reason: BreakReason, arg: Address, size: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_break(reason, arg, size);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn output_debug_string(msg: Address, len: Size) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_output_debug_string(msg: Address, len: Size) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_output_debug_string(msg, len);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn return_from_exception(res: ResultCode) -> ! {
    unsafe extern "C" {
        fn __nx_svc_return_from_exception(res: ResultCode) -> !;
    }

    unsafe {
        __nx_svc_return_from_exception(res);
    }
}

#[inline(always)]
pub fn get_info(id: InfoId, handle: Handle, sub_id: u64) -> Result<u64> {
    unsafe extern "C" {
        fn __nx_svc_get_info(
            out_info: *mut u64,
            id: InfoId,
            handle: Handle,
            sub_id: u64,
        ) -> ResultCode;
    }

    unsafe {
        let mut info: u64 = 0;

        let rc = __nx_svc_get_info(&mut info, id, handle, sub_id);
        pack(rc, info)
    }
}

#[inline(always)]
pub unsafe fn wait_for_address(
    address: Address,
    arbitration_type: ArbitrationType,
    value: u32,
    timeout: i64,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_wait_for_address(
            address: Address,
            arbitration_type: u32,
            value: u32,
            timeout: i64,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_wait_for_address(address, arbitration_type as u32, value, timeout);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn signal_to_address(
    address: Address,
    signal: SignalType,
    value: u32,
    thread_signal_count: i32,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_signal_to_address(
            address: Address,
            signal: u32,
            value: u32,
            signal_count: i32,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_signal_to_address(address, signal as u32, value, thread_signal_count);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn create_session(is_light: bool, unk_name: u64) -> Result<(Handle, Handle)> {
    unsafe extern "C" {
        fn __nx_svc_create_session(
            out_server_handle: *mut Handle,
            out_client_handle: *mut Handle,
            is_light: bool,
            unk_name: u64,
        ) -> ResultCode;
    }

    unsafe {
        let mut server_handle: Handle = 0;
        let mut client_handle: Handle = 0;

        let rc =
            __nx_svc_create_session(&mut server_handle, &mut client_handle, is_light, unk_name);
        pack(rc, (server_handle, client_handle))
    }
}

#[inline(always)]
pub fn accept_session(handle: Handle) -> Result<Handle> {
    unsafe extern "C" {
        fn __nx_svc_accept_session(out_session_handle: *mut Handle, handle: Handle) -> ResultCode;
    }

    unsafe {
        let mut session_handle: Handle = 0;

        let rc = __nx_svc_accept_session(&mut session_handle, handle);
        pack(rc, session_handle)
    }
}

#[inline(always)]
pub unsafe fn reply_and_receive(
    handles: *const Handle,
    handle_count: u32,
    reply_target: Handle,
    timeout: i64,
) -> Result<i32> {
    unsafe extern "C" {
        fn __nx_svc_reply_and_receive(
            out_index: *mut i32,
            handles: *const Handle,
            handle_count: u32,
            reply_target: Handle,
            timeout: i64,
        ) -> ResultCode;
    }

    unsafe {
        let mut index: i32 = 0;

        let rc =
            __nx_svc_reply_and_receive(&mut index, handles, handle_count, reply_target, timeout);
        pack(rc, index)
    }
}

#[inline(always)]
pub fn create_event() -> Result<(Handle, Handle)> {
    unsafe extern "C" {
        fn __nx_svc_create_event(
            out_server_handle: *mut Handle,
            out_client_handle: *mut Handle,
        ) -> ResultCode;
    }

    unsafe {
        let mut server_handle: Handle = 0;
        let mut client_handle: Handle = 0;

        let rc = __nx_svc_create_event(&mut server_handle, &mut client_handle);
        pack(rc, (server_handle, client_handle))
    }
}

#[inline(always)]
pub fn debug_active_process(process_id: u64) -> Result<Handle> {
    unsafe extern "C" {
        fn __nx_svc_debug_active_process(out_handle: *mut Handle, process_id: u64) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_debug_active_process(&mut handle, process_id);
        pack(rc, handle)
    }
}

#[inline(always)]
pub fn break_debug_process(debug_handle: Handle) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_break_debug_process(debug_handle: Handle) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_break_debug_process(debug_handle);
        pack(rc, ())
    }
}

#[inline(always)]
pub fn get_debug_event(debug_handle: Handle) -> Result<DebugEvent> {
    unsafe extern "C" {
        fn __nx_svc_get_debug_event(
            out_debug_event: *mut DebugEvent,
            debug_handle: Handle,
        ) -> ResultCode;
    }

    unsafe {
        let mut debug_event: DebugEvent = mem::zeroed();

        let rc = __nx_svc_get_debug_event(&mut debug_event, debug_handle);
        pack(rc, debug_event)
    }
}

#[inline(always)]
pub fn continue_debug_event(debug_handle: Handle, flags: u32, thread_ids: &[u64]) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_legacy_continue_debug_event(
            debug_handle: Handle,
            flags: u32,
            thread_id: u64,
        ) -> ResultCode;
        fn __nx_svc_continue_debug_event(
            debug_handle: Handle,
            flags: u32,
            thread_ids: *const u64,
            thread_id_count: u32,
        ) -> ResultCode;
    }

    unsafe {
        if version::get_version() < version::Version::new(3, 0, 0) {
            let rc = __nx_svc_legacy_continue_debug_event(debug_handle, flags, thread_ids[0]);
            pack(rc, ())
        } else {
            let rc = __nx_svc_continue_debug_event(
                debug_handle,
                flags,
                thread_ids.as_ptr(),
                thread_ids.len() as u32,
            );
            pack(rc, ())
        }
    }
}

#[inline(always)]
pub fn get_process_list(process_list: &mut [u64]) -> Result<usize> {
    unsafe extern "C" {
        fn __nx_svc_get_process_list(
            out_count: *mut u32,
            out_process_ids: *mut u64,
            process_id_count: u32,
        ) -> ResultCode;
    }

    unsafe {
        let mut count: u32 = 0;

        let rc = __nx_svc_get_process_list(
            &mut count,
            process_list.as_mut_ptr(),
            process_list.len() as u32,
        );
        pack(rc, count as usize)
    }
}

#[inline(always)]
pub fn get_thread_list(debug_handle: Handle, thread_id_list: &mut [u64]) -> Result<usize> {
    unsafe extern "C" {
        fn __nx_svc_get_thread_list(
            out_count: *mut u32,
            out_thread_ids: *mut u64,
            thread_id_count: u32,
            debug_handle: Handle,
        ) -> ResultCode;
    }

    unsafe {
        let mut count: u32 = 0;

        let rc = __nx_svc_get_thread_list(
            &mut count,
            thread_id_list.as_mut_ptr(),
            thread_id_list.len() as u32,
            debug_handle,
        );
        pack(rc, count as usize)
    }
}

#[inline(always)]
pub fn get_debug_thread_context(
    debug_handle: Handle,
    thread_id: u64,
    register_group: arm::RegisterGroup,
) -> Result<arm::ThreadContext> {
    unsafe extern "C" {
        fn __nx_svc_get_debug_thread_context(
            thread_context: *mut u8, // *mut arm::ThreadContext
            debug_handle: Handle,
            thread_id: u64,
            register_group: u32,
        ) -> ResultCode;
    }

    unsafe {
        let mut thread_context: arm::ThreadContext = Default::default();

        let rc = __nx_svc_get_debug_thread_context(
            &raw mut thread_context as *mut _,
            debug_handle,
            thread_id,
            register_group.get(),
        );
        pack(rc, thread_context)
    }
}

#[inline(always)]
pub fn set_debug_thread_context(
    debug_handle: Handle,
    thread_context: arm::ThreadContext,
    thread_id: u64,
    register_group: arm::RegisterGroup,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_set_debug_thread_context(
            debug_handle: Handle,
            thread_id: u64,
            thread_context: *const u8, // *const arm::ThreadContext
            register_group: u32,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_set_debug_thread_context(
            debug_handle,
            thread_id,
            &raw const thread_context as *const _,
            register_group.get(),
        );
        pack(rc, ())
    }
}

/// # Safety
///
/// null pointers are OK here, as we are just querying the memory's information
#[inline(always)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub fn query_debug_process_memory(
    debug_handle: Handle,
    address: Address,
) -> Result<(MemoryInfo, PageInfo)> {
    unsafe extern "C" {
        fn __nx_svc_query_debug_process_memory(
            out_info: *mut MemoryInfo,
            out_page_info: *mut PageInfo,
            debug_handle: Handle,
            address: Address,
        ) -> ResultCode;
    }

    unsafe {
        let mut memory_info: MemoryInfo = Default::default();
        let mut page_info: PageInfo = 0;

        let rc = __nx_svc_query_debug_process_memory(
            &mut memory_info,
            &mut page_info,
            debug_handle,
            address,
        );
        pack(rc, (memory_info, page_info))
    }
}

#[inline(always)]
pub unsafe fn read_debug_process_memory(
    debug_handle: Handle,
    read_address: usize,
    read_size: usize,
    buffer: *mut u8,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_read_debug_process_memory(
            buffer: *mut u8,
            debug_handle: Handle,
            address: usize,
            size: usize,
        ) -> ResultCode;
    }

    unsafe {
        let rc = __nx_svc_read_debug_process_memory(buffer, debug_handle, read_address, read_size);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn write_debug_process_memory(
    debug_handle: Handle,
    write_address: usize,
    write_size: usize,
    buffer: *const u8,
) -> Result<()> {
    unsafe extern "C" {
        fn __nx_svc_write_debug_process_memory(
            debug_handle: Handle,
            buffer: Address,
            address: usize,
            size: usize,
        ) -> ResultCode;
    }

    unsafe {
        let rc =
            __nx_svc_write_debug_process_memory(debug_handle, buffer, write_address, write_size);
        pack(rc, ())
    }
}

#[inline(always)]
pub unsafe fn manage_named_port(name: Address, max_sessions: i32) -> Result<Handle> {
    unsafe extern "C" {
        fn __nx_svc_manage_named_port(
            out_handle: *mut Handle,
            name: Address,
            max_sessions: i32,
        ) -> ResultCode;
    }

    unsafe {
        let mut handle: Handle = 0;

        let rc = __nx_svc_manage_named_port(&mut handle, name, max_sessions);
        pack(rc, handle)
    }
}

#[inline(always)]
pub fn call_secure_monitor(mut inout: [u64; 8]) -> [u64; 8] {
    unsafe extern "C" {
        fn __nx_svc_call_secure_monitor(args: *mut u64);
    }

    unsafe {
        __nx_svc_call_secure_monitor(inout.as_mut_ptr());
    }
    inout
}
