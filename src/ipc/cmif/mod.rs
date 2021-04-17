use super::*;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ControlRequestId {
    ConvertCurrentObjectToDomain = 0,
    CopyFromCurrentDomain = 1,
    CloneCurrentObject = 2,
    QueryPointerBufferSize = 3,
    CloneCurrentObjectEx = 4
}

pub type DomainObjectId = u32;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct ObjectInfo {
    pub handle: svc::Handle,
    pub domain_object_id: DomainObjectId,
    pub owns_handle: bool
}

impl ObjectInfo {
    pub const fn new() -> Self {
        Self { handle: 0, domain_object_id: 0, owns_handle: false }
    }

    pub const fn from_handle(handle: svc::Handle) -> Self {
        Self { handle: handle, domain_object_id: 0, owns_handle: true }
    }

    pub const fn from_domain_object_id(parent_handle: svc::Handle, domain_object_id: DomainObjectId) -> Self {
        Self { handle: parent_handle, domain_object_id: domain_object_id, owns_handle: false }
    }

    pub const fn is_valid(&self) -> bool {
        self.handle != 0
    }

    pub const fn is_domain(&self) -> bool {
        self.domain_object_id != 0
    }

    pub fn convert_current_object_to_domain(&mut self) -> Result<DomainObjectId> {
        ipc_cmif_client_send_control_command!([*self; ControlRequestId::ConvertCurrentObjectToDomain] () => (domain_object_id: DomainObjectId))
    }

    pub fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        ipc_cmif_client_send_control_command!([*self; ControlRequestId::QueryPointerBufferSize] () => (pointer_buffer_size: u16))
    }

    pub fn clone_current_object(&mut self) -> Result<sf::MoveHandle> {
        ipc_cmif_client_send_control_command!([*self; ControlRequestId::CloneCurrentObject] () => (cloned_handle: sf::MoveHandle))
    }
}

pub const IN_DATA_HEADER_MAGIC: u32 = 0x49434653;
pub const OUT_DATA_HEADER_MAGIC: u32 = 0x4F434653;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Derivative)]
#[derivative(Default)]
#[repr(u8)]
pub enum DomainCommandType {
    #[derivative(Default)]
    Invalid = 0,
    SendMessage = 1,
    Close = 2
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DomainInDataHeader {
    pub command_type: DomainCommandType,
    pub object_count: u8,
    pub data_size: u16,
    pub domain_object_id: DomainObjectId,
    pub pad: u32,
    pub token: u32,
}

impl DomainInDataHeader {
    pub const fn empty() -> Self {
        Self { command_type: DomainCommandType::Invalid, object_count: 0, data_size: 0, domain_object_id: 0, pad: 0, token: 0 }
    }

    pub const fn new(command_type: DomainCommandType, object_count: u8, data_size: u16, domain_object_id: DomainObjectId, token: u32) -> Self {
        Self { command_type: command_type, object_count: object_count, data_size: data_size, domain_object_id: domain_object_id, pad: 0, token: token }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(C)]
pub struct DomainOutDataHeader {
    pub out_object_count: u32,
    pub pad: [u32; 3],
}

impl DomainOutDataHeader {
    pub const fn empty() -> Self {
        Self { out_object_count: 0, pad: [0; 3] }
    }

    pub const fn new(out_object_count: u32) -> Self {
        let mut header = Self::empty();
        header.out_object_count = out_object_count;
        header
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Derivative)]
#[derivative(Default)]
#[repr(u16)]
pub enum CommandType {
    #[derivative(Default)]
    Invalid = 0,
    LegacyRequest = 1,
    Close = 2,
    LegacyControl = 3,
    Request = 4,
    Control = 5,
    RequestWithContext = 6,
    ControlWithContext = 7
}

pub fn convert_command_type(command_type: u32) -> CommandType {
    match command_type {
        1 => CommandType::LegacyRequest,
        2 => CommandType::Close,
        3 => CommandType::LegacyControl,
        4 => CommandType::Request,
        5 => CommandType::Control,
        6 => CommandType::RequestWithContext,
        7 => CommandType::ControlWithContext,
        _ => CommandType::Invalid
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
    pub objects_offset: *mut u8,
    copy_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    move_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    objects: ArrayVec<[DomainObjectId; MAX_COUNT]>,
    out_pointer_sizes: ArrayVec<[u16; MAX_COUNT]>,
}

impl CommandIn {
    pub fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), objects_offset: ptr::null_mut(), copy_handles: ArrayVec::new(), move_handles: ArrayVec::new(), objects: ArrayVec::new(), out_pointer_sizes: ArrayVec::new() }
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

    pub fn add_domain_object(&mut self, domain_object_id: DomainObjectId) -> Result<()> {
        match self.objects.try_push(domain_object_id) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_object(&mut self, object_info: ObjectInfo) -> Result<()> {
        if object_info.is_domain() {
            self.add_domain_object(object_info.domain_object_id)
        }
        else {
            Err(ResultCode::new(0xCCC))
        }
    }

    pub fn add_out_pointer_size(&mut self, pointer_size: u16) -> Result<()> {
        match self.out_pointer_sizes.try_push(pointer_size) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
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
    move_handles: ArrayVec<[svc::Handle; MAX_COUNT]>,
    objects: ArrayVec<[DomainObjectId; MAX_COUNT]>
}

impl CommandOut {
    pub fn empty() -> Self {
        Self { send_process_id: false, process_id: 0, data_size: 0, data_offset: ptr::null_mut(), data_words_offset: ptr::null_mut(), copy_handles: ArrayVec::new(), move_handles: ArrayVec::new(), objects: ArrayVec::new() }
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

    pub fn pop_domain_object(&mut self) -> Result<DomainObjectId> {
        match self.objects.pop_at(0) {
            Some(handle) => Ok(handle),
            None => Err(results::cmif::ResultInvalidOutObjectCount::make())
        }
    }

    pub fn push_domain_object(&mut self, domain_object_id: DomainObjectId) -> Result<()> {
        match self.objects.try_push(domain_object_id) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }
}

pub struct CommandContext {
    pub object_info: ObjectInfo,
    pub in_params: CommandIn,
    pub out_params: CommandOut,
    send_statics: ArrayVec<[SendStaticDescriptor; MAX_COUNT]>,
    receive_statics: ArrayVec<[ReceiveStaticDescriptor; MAX_COUNT]>,
    send_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>,
    receive_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>,
    exchange_buffers: ArrayVec<[BufferDescriptor; MAX_COUNT]>,
    pointer_buffer: *mut u8,
    pointer_buffer_offset: usize,
    pointer_size_walker: DataWalker,
    pointer_size_walker_initialized: bool
}

impl CommandContext {
    pub fn empty() -> Self {
        Self { object_info: ObjectInfo::new(), in_params: CommandIn::empty(), out_params: CommandOut::empty(), send_statics: ArrayVec::new(), receive_statics: ArrayVec::new(), send_buffers: ArrayVec::new(), receive_buffers: ArrayVec::new(), exchange_buffers: ArrayVec::new(), pointer_buffer: core::ptr::null_mut(), pointer_buffer_offset: 0, pointer_size_walker: DataWalker::empty(), pointer_size_walker_initialized: false }
    }

    pub fn new_client(object_info: ObjectInfo) -> Self {
        let mut ctx = Self::empty();
        ctx.object_info = object_info;
        ctx
    }

    fn ensure_pointer_size_walker(&mut self, raw_data_walker: &mut DataWalker) {
        if !self.pointer_size_walker_initialized {
            let mut data_size = raw_data_walker.get_offset() + DATA_PADDING as isize + mem::size_of::<DataHeader>() as isize;
            if self.object_info.is_domain() {
                data_size += (mem::size_of::<DomainInDataHeader>() + mem::size_of::<DomainObjectId>() * self.in_params.objects.len()) as isize;
            }
            data_size = (data_size + 1) & !1;
            let out_pointer_sizes_offset = unsafe { self.in_params.data_words_offset.offset(data_size) };
            self.pointer_size_walker = DataWalker::new(out_pointer_sizes_offset);
            self.pointer_size_walker_initialized = true;
        }
    }
    
    pub fn new_server(object_info: ObjectInfo, pointer_buffer: *mut u8) -> Self {
        let mut ctx = Self::empty();
        ctx.object_info = object_info;
        ctx.pointer_buffer = pointer_buffer;
        ctx
    }

    pub fn add_send_static(&mut self, send_static: SendStaticDescriptor) -> Result<()> {
        match self.send_statics.try_push(send_static) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
    }

    pub fn add_receive_static(&mut self, receive_static: ReceiveStaticDescriptor) -> Result<()> {
        match self.receive_statics.try_push(receive_static) {
            Ok(()) => Ok(()),
            Err(_) => Err(ResultCode::new(0xB))
        }
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

        if A.contains(BufferAttribute::AutoSelect()) {
            if self.pointer_buffer.is_null() {
                self.pointer_buffer = self.object_info.query_pointer_buffer_size()? as *mut u8;
            }

            let pointer_buf_size = self.pointer_buffer as usize;
            let mut buffer_in_static = false;
            if pointer_buf_size > 0 {
                let left_size = pointer_buf_size - self.pointer_buffer_offset;
                buffer_in_static = buffer.size <= left_size;
            }
            if buffer_in_static {
                self.pointer_buffer_offset += buffer.size;
            }
            
            if is_in {
                if buffer_in_static {
                    self.add_send_buffer(BufferDescriptor::new(ptr::null(), 0, BufferFlags::Normal))?;
                    self.add_send_static(SendStaticDescriptor::new(buffer.buf, buffer.size, self.send_statics.len() as u32))?;
                }
                else {
                    self.add_send_buffer(BufferDescriptor::new(buffer.buf, buffer.size, BufferFlags::Normal))?;
                    self.add_send_static(SendStaticDescriptor::new(ptr::null(), 0, self.send_statics.len() as u32))?;
                }
            }
            else if is_out {
                if buffer_in_static {
                    self.add_receive_buffer(BufferDescriptor::new(ptr::null(), 0, BufferFlags::Normal))?;
                    self.add_receive_static(ReceiveStaticDescriptor::new(buffer.buf, buffer.size))?;
                    self.in_params.add_out_pointer_size(buffer.size as u16)?;
                }
                else {
                    self.add_receive_buffer(BufferDescriptor::new(buffer.buf, buffer.size, BufferFlags::Normal))?;
                    self.add_receive_static(ReceiveStaticDescriptor::new(ptr::null(), 0))?;
                    self.in_params.add_out_pointer_size(0)?;
                }
            }
        }
        else if A.contains(BufferAttribute::Pointer()) {
            if is_in {
                self.add_send_static(SendStaticDescriptor::new(buffer.buf, buffer.size, self.send_statics.len() as u32))?;
            }
            else if is_out {
                self.add_receive_static(ReceiveStaticDescriptor::new(buffer.buf, buffer.size))?;
                if !A.contains(BufferAttribute::FixedSize()) {
                    self.in_params.add_out_pointer_size(buffer.size as u16)?;
                }
            }
        }
        else if A.contains(BufferAttribute::MapAlias()) {
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

        Ok(())
    }

    pub fn pop_send_static(&mut self) -> Result<SendStaticDescriptor> {
        match self.send_statics.pop_at(0) {
            Some(send_static) => Ok(send_static),
            None => Err(ResultCode::new(0xBB))
        }
    }

    pub fn pop_receive_static(&mut self) -> Result<ReceiveStaticDescriptor> {
        match self.receive_statics.pop_at(0) {
            Some(receive_static) => Ok(receive_static),
            None => Err(ResultCode::new(0xBB))
        }
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

    pub fn pop_buffer<const A: BufferAttribute, const S: usize>(&mut self, raw_data_walker: &mut DataWalker) -> Result<sf::Buffer<A, S>> {
        let is_in = A.contains(BufferAttribute::In());
        let is_out = A.contains(BufferAttribute::Out());

        if A.contains(BufferAttribute::AutoSelect()) {
            if is_in {
                if let Ok(static_desc) = self.pop_send_static() {
                    if let Ok(send_desc) = self.pop_send_buffer() {
                        if !static_desc.get_address().is_null() && (static_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(static_desc.get_address(), static_desc.get_size()));
                        }
                        if !send_desc.get_address().is_null() && (send_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(send_desc.get_address(), send_desc.get_size()));
                        }
                    }
                }
            }
            else if is_out {
                if let Ok(static_desc) = self.pop_receive_static() {
                    if let Ok(recv_desc) = self.pop_receive_buffer() {
                        if !static_desc.get_address().is_null() && (static_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(static_desc.get_address(), static_desc.get_size()));
                        }
                        if !recv_desc.get_address().is_null() && (recv_desc.get_size() > 0) {
                            return Ok(sf::Buffer::from_mut(recv_desc.get_address(), recv_desc.get_size()));
                        }
                    }
                }
            }
        }
        else if A.contains(BufferAttribute::Pointer()) {
            if is_in {
                if let Ok(static_desc) = self.pop_send_static() {
                    return Ok(sf::Buffer::from_mut(static_desc.get_address(), static_desc.get_size()));
                }
            }
            else if is_out {
                let buf_size = match A.contains(BufferAttribute::FixedSize()) {
                    true => S,
                    false => {
                        self.ensure_pointer_size_walker(raw_data_walker);
                        self.pointer_size_walker.advance_get::<u16>() as usize
                    }
                };
                self.pointer_buffer_offset = crate::mem::align_down(self.pointer_buffer_offset - buf_size, 0x10);
                let buf = unsafe { self.pointer_buffer.offset(self.pointer_buffer_offset as isize) };
                self.add_send_static(SendStaticDescriptor::new(buf as *const u8, buf_size, self.send_statics.len() as u32))?;
                return Ok(sf::Buffer::from_mut(buf, buf_size));
            }
        }
        else if A.contains(BufferAttribute::MapAlias()) {
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

        Err(ResultCode::new(0xBABE))
    }

    pub fn pop_object(&mut self) -> Result<ObjectInfo> {
        let object_info: ObjectInfo;
        if self.object_info.is_domain() {
            let domain_object_id = self.out_params.pop_domain_object()?;
            object_info = ObjectInfo::from_domain_object_id(self.object_info.handle, domain_object_id);
        }
        else {
            let handle: sf::MoveHandle = self.out_params.pop_handle()?;
            object_info = ObjectInfo::from_handle(handle.handle);
        }
        Ok(object_info)
    }
}

pub mod sf;

pub mod client;

pub mod server;