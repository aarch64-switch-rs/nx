use super::*;
use crate::version;
use alloc::vec::Vec;
use alloc::string::String;

pub struct Buffer<const A: BufferAttribute, T> {
    buf: *mut T,
    count: usize
}

impl<const A: BufferAttribute, T> Buffer<A, T> {
    pub const fn get_expected_size() -> usize {
        mem::size_of::<T>()
    }

    pub const fn empty() -> Self {
        Self {
            buf: ptr::null_mut(),
            count: 0
        }
    }

    // TODO: ensure that sizeof(T) is a multiple of size

    pub const fn new(addr: *mut u8, size: usize) -> Self {
        Self {
            buf: addr as *mut T,
            count: size / Self::get_expected_size()
        }
    }
    
    pub const fn from_ptr(buf: *const T, count: usize) -> Self {
        Self {
            buf: buf as *mut T,
            count
        }
    }

    pub const fn from_mut_ptr(buf: *mut T, count: usize) -> Self {
        Self {
            buf,
            count
        }
    }

    pub const fn from_var(var: &T) -> Self {
        Self::from_ptr(var as *const T, 1)
    }

    pub const fn from_mut_var(var: &mut T) -> Self {
        Self::from_mut_ptr(var as *mut T, 1)
    }

    // TODO: ensure sizeof(T) is a multiple of sizeof(U)

    pub const fn from_other_var<U>(var: &U) -> Self {
        Self::from_ptr(var as *const U as *const T, mem::size_of::<U>() / Self::get_expected_size())
    }

    pub const fn from_other_mut_var<U>(var: &mut U) -> Self {
        Self::from_mut_ptr(var as *mut U as *mut T, mem::size_of::<U>() / Self::get_expected_size())
    }

    pub const fn from_array(arr: &[T]) -> Self {
        Self::from_ptr(arr.as_ptr(), arr.len())
    }

    pub const fn from_mut_array(arr: &mut [T]) -> Self {
        Self::from_mut_ptr(arr.as_mut_ptr(), arr.len())
    }

    pub const fn from_other<const A2: BufferAttribute, U>(other: &Buffer<A2, U>) -> Self {
        Self::new(other.get_address(), other.get_size())
    }

    pub const fn get_address(&self) -> *mut u8 {
        self.buf as *mut u8
    }

    pub const fn get_size(&self) -> usize {
        self.count * Self::get_expected_size()
    }

    pub const fn get_count(&self) -> usize {
        self.count
    }

    pub const fn get_var(&self) -> &T {
        unsafe {
            &*(self.buf as *const T)
        }
    }

    pub fn get_mut_var(&mut self) -> &mut T {
        unsafe {
            &mut *self.buf
        }
    }

    pub fn set_var(&mut self, t: T) {
        unsafe {
            *self.buf = t;
        }
    }

    pub fn get_slice(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(self.buf as *const T, self.count)
        }
    }

    pub fn get_mut_slice(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.buf, self.count)
        }
    }
}

impl<const A: BufferAttribute> Buffer<A, u8> {
    pub fn get_string(&self) -> String {
        unsafe {
            let mut string = String::with_capacity(self.count);
            for i in 0..self.count {
                let cur_char = *self.buf.add(i) as char;
                if cur_char == '\0' {
                    break;
                }
                string.push(cur_char);
            }
            string
        }
    }

    pub fn set_string(&mut self, string: String) {
        unsafe {
            // First memset to zero so that it will be a valid nul-terminated string
            core::ptr::write_bytes(self.buf, 0, self.count);
            core::ptr::copy(string.as_ptr(), self.buf, core::cmp::min(self.count - 1, string.len()));
        }
    }
}

pub type InMapAliasBuffer<T> = Buffer<{bit_group!{ BufferAttribute [In, MapAlias] }}, T>;
pub type OutMapAliasBuffer<T> = Buffer<{bit_group!{ BufferAttribute [Out, MapAlias] }}, T>;
pub type InNonSecureMapAliasBuffer<T> = Buffer<{bit_group!{ BufferAttribute [In, MapAlias, MapTransferAllowsNonSecure] }}, T>;
pub type OutNonSecureMapAliasBuffer<T> = Buffer<{bit_group!{ BufferAttribute [Out, MapAlias, MapTransferAllowsNonSecure] }}, T>;
pub type InAutoSelectBuffer<T> = Buffer<{bit_group!{ BufferAttribute [In, AutoSelect] }}, T>;
pub type OutAutoSelectBuffer<T> = Buffer<{bit_group!{ BufferAttribute [Out, AutoSelect] }}, T>;
pub type InPointerBuffer<T> = Buffer<{bit_group!{ BufferAttribute [In, Pointer] }}, T>;
pub type OutPointerBuffer<T> = Buffer<{bit_group!{ BufferAttribute [Out, Pointer] }}, T>;
pub type InFixedPointerBuffer<T> = Buffer<{bit_group!{ BufferAttribute [In, Pointer, FixedSize] }}, T>;
pub type OutFixedPointerBuffer<T> = Buffer<{bit_group!{ BufferAttribute [Out, Pointer, FixedSize] }}, T>;

#[derive(Clone)]
pub struct Handle<const M: HandleMode> {
    pub handle: svc::Handle
}

impl<const M: HandleMode> Handle<M> {
    pub const fn from(handle: svc::Handle) -> Self {
        Self { handle }
    }
}

pub type CopyHandle = Handle<{HandleMode::Copy}>;
pub type MoveHandle = Handle<{HandleMode::Move}>;

#[derive(Clone, Default)]
pub struct ProcessId {
    pub process_id: u64
}

impl ProcessId {
    pub const fn from(process_id: u64) -> Self {
        Self { process_id }
    }

    pub const fn new() -> ProcessId {
        Self {
            process_id: 0
        }
    }
}

// This is used, for instance, with u8-sized enums which are sent/received as u32s in commands

#[derive(Copy, Clone)]
#[repr(C)]
pub union EnumAsPrimitiveType<E: Copy + Clone, T: Copy + Clone> {
    val: T,
    enum_val: E
}

impl<E: Copy + Clone, T: Copy + Clone> EnumAsPrimitiveType<E, T> {
    pub fn from(enum_val: E) -> Self {
        Self {
            enum_val
        }
    }

    pub fn from_val(val: T) -> Self {
        Self {
            val
        }
    }

    pub fn get(&self) -> E {
        unsafe { self.enum_val }
    }

    pub fn set(&mut self, enum_val: E) {
        self.enum_val = enum_val;
    }

    pub fn get_value(&self) -> T {
        unsafe { self.val }
    }

    pub fn set_value(&mut self, val: T) {
        self.val = val;
    }
}

#[derive(Default)]
pub struct Session {
    pub object_info: ObjectInfo
}

impl Session {
    pub const fn new() -> Self  {
        Self { object_info: ObjectInfo::new() }
    }

    pub const fn from(object_info: ObjectInfo) -> Self {
        Self { object_info }
    }
    
    pub const fn from_handle(handle: svc::Handle) -> Self {
        Self::from(ObjectInfo::from_handle(handle))
    }

    pub fn convert_to_domain(&mut self) -> Result<()> {
        self.object_info.domain_object_id = self.object_info.convert_current_object_to_domain()?;
        Ok(())
    }

    pub fn get_info(&mut self) -> &mut ObjectInfo {
        &mut self.object_info
    }

    pub fn set_info(&mut self, info: ObjectInfo) {
        self.object_info = info;
    }

    pub fn close(&mut self) {
        if self.object_info.is_valid() {
            if self.object_info.is_domain() {
                let mut ctx = CommandContext::new_client(self.object_info);
                cmif::client::write_request_command_on_msg_buffer(&mut ctx, None, cmif::DomainCommandType::Close);
                let _ = svc::send_sync_request(self.object_info.handle);
            }
            else if self.object_info.owns_handle {
                let mut ctx = CommandContext::new_client(self.object_info);
                
                match self.object_info.protocol {
                    CommandProtocol::Cmif => cmif::client::write_close_command_on_msg_buffer(&mut ctx),
                    CommandProtocol::Tipc => tipc::client::write_close_command_on_msg_buffer(&mut ctx)
                };

                let _ = svc::send_sync_request(self.object_info.handle);
            }
            if self.object_info.owns_handle {
                let _ = svc::close_handle(self.object_info.handle);
            }
            self.object_info = ObjectInfo::new();
        }
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        self.close();
    }
}

pub struct CommandMetadata {
    pub rq_id: u32,
    pub command_fn: server::CommandFn,
    pub ver_intv: version::VersionInterval
}

pub type CommandMetadataTable = Vec<CommandMetadata>;

impl CommandMetadata {
    pub const fn new(rq_id: u32, command_fn: server::CommandFn, ver_intv: version::VersionInterval) -> Self {
        Self {
            rq_id,
            command_fn,
            ver_intv
        }
    }

    pub fn matches(&self, rq_id: u32) -> bool {
        let cur_ver = version::get_version();
        (self.rq_id == rq_id) && self.ver_intv.contains(cur_ver)
    }
}

// This trait is analogous to N's nn::sf::IServiceObject type - the base trait for any kind of IPC interface
// IClientObject / {IService, INamedPort} (on client module) and ISessionObject / {IServerObject, IMitmServerObject} (on server module) are superior types for specific kind of objects

// TODO: make use of the command metadata on client side too (for instance for checking if the command is valid on the current system version, etc.)
// TODO: think of a proper way to migrate call_self_server_command / command_fn stuff to server and avoid it being on every single IObject?

pub trait IObject {
    fn get_session(&mut self) -> &mut Session;

    fn get_command_metadata_table(&self) -> CommandMetadataTable;

    fn call_self_server_command(&mut self, command_fn: server::CommandFn, protocol: CommandProtocol, ctx: &mut server::ServerContext) -> Result<()> {
        let self_fn: server::CommandSpecificFn<Self> = unsafe { core::mem::transmute(command_fn) };
        (self_fn)(self, protocol, ctx)
    }
}

pub mod sm;

pub mod psm;

pub mod applet;

pub mod lm;

pub mod fatal;

pub mod dispdrv;

pub mod fsp;

pub mod hid;

pub mod nv;

pub mod vi;

pub mod hipc;

pub mod psc;

pub mod pm;

pub mod nfp;

pub mod mii;

pub mod set;

pub mod spl;

pub mod usb;

pub mod ldr;

pub mod ncm;

pub mod lr;
