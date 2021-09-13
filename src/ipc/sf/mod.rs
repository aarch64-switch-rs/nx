use super::*;
use crate::svc;
use crate::version;
use core::mem;
use alloc::vec::Vec;
use alloc::string::String;

#[derive(Clone)]
pub struct Buffer<const A: BufferAttribute, const S: usize> {
    pub buf: *const u8,
    pub size: usize
}

impl<const A: BufferAttribute, const S: usize> Buffer<A, S> {
    pub const fn new() -> Self {
        Self { buf: ptr::null_mut(), size: 0 }
    }

    pub const fn from_other<const B: BufferAttribute, const Z: usize>(other: &Buffer<B, Z>) -> Self {
        Self { buf: other.buf, size: other.size }
    }
    
    pub const fn from_const<T>(buf: *const T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_mut<T>(buf: *mut T, size: usize) -> Self {
        Self { buf: buf as *const u8, size: size }
    }

    pub const fn from_var<T>(var: &T) -> Self {
        Self::from_const(var as *const T, mem::size_of::<T>())
    }

    pub const fn from_array<T>(arr: &[T]) -> Self {
        Self::from_const(arr.as_ptr(), arr.len() * mem::size_of::<T>())
    }

    pub const fn get_as<T>(&self) -> &T {
        unsafe {
            &*(self.buf as *const T)
        }
    }

    pub fn get_mut_as<T>(&self) -> &mut T {
        unsafe {
            &mut *(self.buf as *mut T)
        }
    }

    pub fn set_as<T>(&mut self, t: T) {
        unsafe {
            *(self.buf as *mut T) = t;
        }
    }

    pub fn get_slice<T>(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(self.buf as *const T, self.size / mem::size_of::<T>())
        }
    }

    pub fn get_mut_slice<T>(&self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(self.buf as *mut T, self.size / mem::size_of::<T>())
        }
    }

    pub fn get_string(&self) -> String {
        unsafe {
            let mut string = String::with_capacity(self.size);
            for i in 0..self.size {
                let cur_char = *self.buf.offset(i as isize) as char;
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
            core::ptr::write_bytes(self.buf as *mut u8, 0, self.size);
            core::ptr::copy(string.as_ptr(), self.buf as *mut u8, core::cmp::min(self.size - 1, string.len()));
        }
    }
}

pub type InMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [In, MapAlias] }}, 0>;
pub type OutMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [Out, MapAlias] }}, 0>;
pub type InNonSecureMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [In, MapAlias, MapTransferAllowsNonSecure] }}, 0>;
pub type OutNonSecureMapAliasBuffer = Buffer<{bit_group!{ BufferAttribute [Out, MapAlias, MapTransferAllowsNonSecure] }}, 0>;
pub type InAutoSelectBuffer = Buffer<{bit_group!{ BufferAttribute [In, AutoSelect] }}, 0>;
pub type OutAutoSelectBuffer = Buffer<{bit_group!{ BufferAttribute [Out, AutoSelect] }}, 0>;
pub type InPointerBuffer = Buffer<{bit_group!{ BufferAttribute [In, Pointer] }}, 0>;
pub type OutPointerBuffer = Buffer<{bit_group!{ BufferAttribute [Out, Pointer] }}, 0>;
pub type InFixedPointerBuffer<T> = Buffer<{bit_group!{ BufferAttribute [In, Pointer, FixedSize] }}, {mem::size_of::<T>()}>;
pub type OutFixedPointerBuffer<T> = Buffer<{bit_group!{ BufferAttribute [Out, Pointer, FixedSize] }}, {mem::size_of::<T>()}>;

#[derive(Clone)]
pub struct Handle<const M: HandleMode> {
    pub handle: svc::Handle
}

impl<const M: HandleMode> Handle<M> {
    pub const fn from(handle: svc::Handle) -> Self {
        Self { handle: handle }
    }
}

pub type CopyHandle = Handle<{HandleMode::Copy}>;
pub type MoveHandle = Handle<{HandleMode::Move}>;

#[derive(Clone)]
pub struct ProcessId {
    pub process_id: u64
}

impl ProcessId {
    pub const fn from(process_id: u64) -> Self {
        Self { process_id: process_id }
    }

    pub const fn new() -> ProcessId {
        Self::from(0)
    }
}

pub struct Session {
    pub object_info: ObjectInfo
}

impl Session {
    pub const fn new() -> Self  {
        Self { object_info: ObjectInfo::new() }
    }

    pub const fn from(object_info: ObjectInfo) -> Self {
        Self { object_info: object_info }
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
                cmif::client::write_request_command_on_ipc_buffer(&mut ctx, None, cmif::DomainCommandType::Close);
                let _ = svc::send_sync_request(self.object_info.handle);
            }
            else if self.object_info.owns_handle {
                let mut ctx = CommandContext::new_client(self.object_info);
                
                match self.object_info.protocol {
                    CommandProtocol::Cmif => cmif::client::write_close_command_on_ipc_buffer(&mut ctx),
                    CommandProtocol::Tipc => tipc::client::write_close_command_on_ipc_buffer(&mut ctx)
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

pub type CommandFn = fn(&mut dyn IObject, &mut server::ServerContext) -> Result<()>;
pub type CommandSpecificFn<T> = fn(&mut T, &mut server::ServerContext) -> Result<()>;

pub struct CommandMetadata {
    pub protocol: CommandProtocol,
    pub rq_id: u32,
    pub command_fn: CommandFn,
    pub min_ver: Option<version::Version>,
    pub max_ver: Option<version::Version>
}

pub type CommandMetadataTable = Vec<CommandMetadata>;

impl CommandMetadata {
    pub fn new(protocol: CommandProtocol, rq_id: u32, command_fn: CommandFn, min_ver: Option<version::Version>, max_ver: Option<version::Version>) -> Self {
        Self { protocol: protocol, rq_id: rq_id, command_fn: command_fn, min_ver: min_ver, max_ver: max_ver }
    }

    pub fn validate_version(&self) -> bool {
        let ver = version::get_version();
        if let Some(min_v) = self.min_ver {
            if ver < min_v {
                return false;
            }
        }
        if let Some(max_v) = self.max_ver {
            if ver > max_v {
                return false;
            }
        }
        true
    }

    pub fn matches(&self, protocol: CommandProtocol, rq_id: u32) -> bool {
        self.validate_version() && (self.protocol == protocol) && (self.rq_id == rq_id)
    }
}

// This trait is analogous to N's IServiceObject type - the base for any kind of IPC interface
// IClientObject (on service module) and IServerObject (on server module) are wrappers for some specific kind of objects

pub trait IObject {
    fn get_session(&mut self) -> &mut Session;
    fn get_command_table(&self) -> CommandMetadataTable;

    fn get_info(&mut self) -> ObjectInfo {
        self.get_session().object_info
    }

    fn set_info(&mut self, info: ObjectInfo) {
        self.get_session().set_info(info);
    }

    fn convert_to_domain(&mut self) -> Result<()> {
        self.get_session().convert_to_domain()
    }

    fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        self.get_info().query_pointer_buffer_size()
    }

    fn close_session(&mut self) {
        self.get_session().close()
    }

    fn is_valid(&mut self) -> bool {
        self.get_info().is_valid()
    }
    
    fn is_domain(&mut self) -> bool {
        self.get_info().is_domain()
    }

    fn call_self_command(&mut self, command_fn: CommandFn, ctx: &mut server::ServerContext) -> Result<()> {
        let original_fn: CommandSpecificFn<Self> = unsafe { mem::transmute(command_fn) };
        (original_fn)(self, ctx)
    }
}

pub mod sm;

pub mod psm;

pub mod applet;

pub mod lm;

pub mod fatal;

pub mod dispdrv;

pub mod fspsrv;

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