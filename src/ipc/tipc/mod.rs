use super::*;
use crate::ipc::cmif;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum CommandType {
    Invalid = 0,
    CloseSession = 15
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ObjectInfo {
    pub handle: svc::Handle,
    pub owns_handle: bool
}

impl ObjectInfo {
    pub const fn new() -> Self {
        Self { handle: 0, owns_handle: false }
    }

    pub const fn from_handle(handle: svc::Handle) -> Self {
        Self { handle: handle, owns_handle: true }
    }

    pub const fn is_valid(&self) -> bool {
        self.handle != 0
    }

    pub const fn convert_to_cmif(&self) -> cmif::ObjectInfo {
        cmif::ObjectInfo {
            handle: self.handle,
            domain_object_id: 0,
            owns_handle: self.owns_handle
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DataHeader {
    pub magic: u32,
    pub version: u32,
    pub value: u32,
    pub token: u32,
}

impl DataHeader {
    pub const fn empty() -> Self {
        Self { magic: 0, version: 0, value: 0, token: 0 }
    }

    pub const fn new(magic: u32, version: u32, value: u32, token: u32) -> Self {
        Self { magic: magic, version: version, value: value, token: token }
    }
}

pub struct CommandIn {
    pub send_process_id: bool,
    pub process_id: u64,
    pub data_size: u32,
    pub data_offset: *mut u8,
    pub data_words_offset: *mut u8,
    copy_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    move_handles: ArrayVec<[svc::Handle; MAX_COUNT]>
}

impl CommandIn {
    pub fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), copy_handles: ArrayVec::new(), move_handles: ArrayVec::new() }
    }
    
    pub fn add_copy_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.copy_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_move_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.move_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_handle<const M: HandleMode>(&mut self, handle: sf::Handle<M>) -> Result<()> {
        match M {
            HandleMode::Copy => self.add_copy_handle(handle.handle),
            HandleMode::Move => self.add_move_handle(handle.handle)
        }
    }
}

pub struct CommandOut {
    pub send_process_id: bool,
    pub process_id: u64,
    pub data_size: u32,
    pub data_offset: *mut u8,
    pub data_words_offset: *mut u8,
    copy_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    move_handles: ArrayVec<[svc::Handle; MAX_COUNT]>
}

impl CommandOut {
    pub fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), copy_handles: ArrayVec::new(), move_handles: ArrayVec::new() }
    }
    
    pub fn pop_copy_handle(&mut self) -> Result<svc::Handle> {
        match self.copy_handles.pop_at(0) {
            Some(handle) => Ok(handle),
            None => Err(results::cmif::ResultInvalidOutObjectCount::make())
        }
    }

    pub fn pop_move_handle(&mut self) -> Result<svc::Handle> {
        match self.move_handles.pop_at(0) {
            Some(handle) => Ok(handle),
            None => Err(results::cmif::ResultInvalidOutObjectCount::make())
        }
    }

    pub fn pop_handle<const M: HandleMode>(&mut self) -> Result<sf::Handle<M>> {
        let handle = match M {
            HandleMode::Copy => sf::Handle::from(self.pop_copy_handle()?),
            HandleMode::Move => sf::Handle::from(self.pop_move_handle()?),
        };
        Ok(handle)
    }

    pub fn push_copy_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.copy_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn push_move_handle(&mut self, handle: svc::Handle) -> Result<()> {
        match self.move_handles.try_push(handle) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn push_handle<const M: HandleMode>(&mut self, handle: sf::Handle<M>) -> Result<()> {
        match M {
            HandleMode::Copy => self.push_copy_handle(handle.handle),
            HandleMode::Move => self.push_move_handle(handle.handle)
        }
    }
}

pub struct CommandContext {
    pub object_info: ObjectInfo,
    pub in_params: CommandIn,
    pub out_params: CommandOut,
    send_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>,
    receive_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>,
    exchange_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>
}

impl CommandContext {
    pub fn empty() -> Self {
        Self { object_info: ObjectInfo::new(), in_params: CommandIn::empty(), out_params: CommandOut::empty(), send_buffers: ArrayVec::new(), receive_buffers: ArrayVec::new(), exchange_buffers: ArrayVec::new() }
    }

    pub fn new_client(object_info: ObjectInfo) -> Self {
        let mut ctx = Self::empty();
        ctx.object_info = object_info;
        ctx
    }

    pub fn add_send_buffer(&mut self, send_buffer: BufferDescriptor) -> Result<()> {
        match self.send_buffers.try_push(send_buffer) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_receive_buffer(&mut self, receive_buffer: BufferDescriptor) -> Result<()> {
        match self.receive_buffers.try_push(receive_buffer) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_exchange_buffer(&mut self, exchange_buffer: BufferDescriptor) -> Result<()> {
        match self.exchange_buffers.try_push(exchange_buffer) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_buffer<const A: BufferAttribute, const S: usize>(&mut self, buffer: sf::Buffer<A, S>) -> Result<()> {
        let is_in = A.contains(BufferAttribute::In());
        let is_out = A.contains(BufferAttribute::Out());

        if A.contains(BufferAttribute::MapAlias()) {
            let mut flags = BufferFlags::Normal;
            if A.contains(BufferAttribute::MapTransferAllowsNonSecure()) {
                flags = BufferFlags::NonSecure;
            }
            else if A.contains(BufferAttribute::MapTransferAllowsNonDevice()) {
                flags = BufferFlags::NonDevice;
            }
            let buf_desc = BufferDescriptor::new(buffer.buf, buffer.size, flags);
            if is_in && is_out {
                self.add_exchange_buffer(buf_desc)?;
            }
            else if is_in {
                self.add_send_buffer(buf_desc)?;
            }
            else if is_out {
                self.add_receive_buffer(buf_desc)?;
            }
        }
        // In TIPC, only MapAlias buffers are supported

        Ok(())
    }

    pub fn pop_send_buffer(&mut self) -> Result<BufferDescriptor> {
        match self.send_buffers.pop_at(0) {
            Some(send_buffer) => Ok(send_buffer),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_receive_buffer(&mut self) -> Result<BufferDescriptor> {
        match self.receive_buffers.pop_at(0) {
            Some(receive_buffer) => Ok(receive_buffer),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_exchange_buffer(&mut self) -> Result<BufferDescriptor> {
        match self.exchange_buffers.pop_at(0) {
            Some(exchange_buffer) => Ok(exchange_buffer),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_buffer<const A: BufferAttribute, const S: usize>(&mut self, _raw_data_walker: &mut DataWalker) -> Result<sf::Buffer<A, S>> {
        let is_in = A.contains(BufferAttribute::In());
        let is_out = A.contains(BufferAttribute::Out());

        if A.contains(BufferAttribute::MapAlias()) {
            if is_in && is_out {
                if let Ok(exch_desc) = self.pop_exchange_buffer() {
                    return Ok(sf::Buffer::from_mut(exch_desc.get_address(), exch_desc.get_size()));
                }
            }
            else if is_in {
                if let Ok(send_desc) = self.pop_send_buffer() {
                    return Ok(sf::Buffer::from_mut(send_desc.get_address(), send_desc.get_size()));
                }
            }
            else if is_out {
                if let Ok(recv_desc) = self.pop_receive_buffer() {
                    return Ok(sf::Buffer::from_mut(recv_desc.get_address(), recv_desc.get_size()));
                }
            }
        }
        // In TIPC, only MapAlias buffers are supported

        Err(ResultCode::new(0xBABE))
    }
}

pub mod sf;

pub mod client;

pub mod server;