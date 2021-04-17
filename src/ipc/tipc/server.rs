use crate::result::*;
use crate::results;
use crate::svc;
use crate::service;
use crate::service::tipc::sm;
use crate::service::tipc::sm::IUserInterface;
use crate::mem;
use super::*;
use alloc::vec::Vec;
use core::mem as cmem;

// TODO: proper result codes

pub struct ServerContext<'a> {
    pub ctx: &'a mut CommandContext,
    pub raw_data_walker: DataWalker,
    pub new_sessions: &'a mut Vec<ServerHolder>
}

impl<'a> ServerContext<'a> {
    pub const fn new(ctx: &'a mut CommandContext, raw_data_walker: DataWalker, new_sessions: &'a mut Vec<ServerHolder>) -> Self {
        Self { ctx: ctx, raw_data_walker: raw_data_walker, new_sessions: new_sessions }
    }
}

#[inline(always)]
pub fn read_command_from_ipc_buffer(ctx: &mut CommandContext) -> u32 {
    unsafe {
        let mut ipc_buf = get_ipc_buffer();

        let command_header = ipc_buf as *mut CommandHeader;
        ipc_buf = command_header.offset(1) as *mut u8;

        let command_type = (*command_header).get_command_type();
        let data_size = (*command_header).get_data_word_count() * cmem::size_of::<u32>() as u32;
        ctx.in_params.data_size = data_size;

        if (*command_header).get_has_special_header() {
            let special_header = ipc_buf as *mut CommandSpecialHeader;
            ipc_buf = special_header.offset(1) as *mut u8;

            ctx.in_params.send_process_id = (*special_header).get_send_process_id();
            if ctx.in_params.send_process_id {
                let process_id_ptr = ipc_buf as *mut u64;
                ctx.in_params.process_id = *process_id_ptr;
                ipc_buf = process_id_ptr.offset(1) as *mut u8;
            }

            let copy_handle_count = (*special_header).get_copy_handle_count();
            ipc_buf = read_array_from_buffer(ipc_buf, copy_handle_count, &mut ctx.in_params.copy_handles);
            let move_handle_count = (*special_header).get_move_handle_count();
            ipc_buf = read_array_from_buffer(ipc_buf, move_handle_count, &mut ctx.in_params.move_handles);
        }

        let send_buffer_count = (*command_header).get_send_buffer_count();
        ipc_buf = read_array_from_buffer(ipc_buf, send_buffer_count, &mut ctx.send_buffers);
        let receive_buffer_count = (*command_header).get_receive_buffer_count();
        ipc_buf = read_array_from_buffer(ipc_buf, receive_buffer_count, &mut ctx.receive_buffers);
        let exchange_buffer_count = (*command_header).get_exchange_buffer_count();
        ipc_buf = read_array_from_buffer(ipc_buf, exchange_buffer_count, &mut ctx.exchange_buffers);

        ctx.in_params.data_words_offset = ipc_buf;
        command_type
    }
}

#[inline(always)]
pub fn write_command_response_on_ipc_buffer(ctx: &mut CommandContext, command_type: u32, data_size: u32) {
    unsafe {
        let mut ipc_buf = get_ipc_buffer();
        
        let command_header = ipc_buf as *mut CommandHeader;
        ipc_buf = command_header.offset(1) as *mut u8;

        let data_word_count = (data_size + 3) / 4;
        let has_special_header = ctx.out_params.send_process_id || (ctx.out_params.copy_handles.len() > 0) || (ctx.out_params.move_handles.len() > 0);
        *command_header = CommandHeader::new(command_type, 0, ctx.send_buffers.len() as u32, ctx.receive_buffers.len() as u32, ctx.exchange_buffers.len() as u32, data_word_count, 0, has_special_header);

        if has_special_header {
            let special_header = ipc_buf as *mut CommandSpecialHeader;
            ipc_buf = special_header.offset(1) as *mut u8;

            *special_header = CommandSpecialHeader::new(ctx.out_params.send_process_id, ctx.out_params.copy_handles.len() as u32, ctx.out_params.move_handles.len() as u32);
            if ctx.out_params.send_process_id {
                ipc_buf = ipc_buf.offset(cmem::size_of::<u64>() as isize);
            }

            ipc_buf = write_array_to_buffer(ipc_buf, ctx.out_params.copy_handles.len() as u32, &ctx.out_params.copy_handles);
            ipc_buf = write_array_to_buffer(ipc_buf, ctx.out_params.move_handles.len() as u32, &ctx.out_params.move_handles);
        }

        ipc_buf = write_array_to_buffer(ipc_buf, ctx.send_buffers.len() as u32, &ctx.send_buffers);
        ipc_buf = write_array_to_buffer(ipc_buf, ctx.receive_buffers.len() as u32, &ctx.receive_buffers);
        ipc_buf = write_array_to_buffer(ipc_buf, ctx.exchange_buffers.len() as u32, &ctx.exchange_buffers);
        ctx.out_params.data_words_offset = ipc_buf;
    }
}

#[inline(always)]
pub fn read_request_command_from_ipc_buffer(ctx: &mut CommandContext) -> Result<()> {
    let ipc_buf = get_ipc_buffer();
    let data_offset = get_aligned_data_offset(ctx.in_params.data_words_offset, ipc_buf);

    ctx.in_params.data_offset = data_offset;
    Ok(())
}

#[inline(always)]
pub fn write_request_command_response_on_ipc_buffer(ctx: &mut CommandContext, result: ResultCode, request_type: u32) {
    unsafe {
        let ipc_buf = get_ipc_buffer();
        let mut data_size = DATA_PADDING + cmem::size_of::<DataHeader>() as u32 + ctx.out_params.data_size;
        data_size = (data_size + 1) & !1;

        write_command_response_on_ipc_buffer(ctx, request_type, data_size);
        let data_offset = get_aligned_data_offset(ctx.out_params.data_words_offset, ipc_buf);
        let rc_ref = data_offset as *mut ResultCode;
        *rc_ref = result;

        ctx.out_params.data_offset = rc_ref.offset(1) as *mut u8;
    }
}

#[inline(always)]
pub fn write_close_command_response_on_ipc_buffer(ctx: &mut CommandContext) {
    write_command_response_on_ipc_buffer(ctx, CommandType::CloseSession as u32, 0);
}

pub trait CommandParameter<O> {
    fn after_request_read(ctx: &mut ServerContext) -> Result<O>;
    fn before_response_write(var: &Self, ctx: &mut ServerContext) -> Result<()>;
    fn after_response_write(var: &Self, ctx: &mut ServerContext) -> Result<()>;
}

impl<T: Copy> CommandParameter<T> for T {
    default fn after_request_read(ctx: &mut ServerContext) -> Result<Self> {
        Ok(ctx.raw_data_walker.advance_get())
    }

    default fn before_response_write(_raw: &Self, ctx: &mut ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance::<Self>();
        Ok(())
    }

    default fn after_response_write(raw: &Self, ctx: &mut ServerContext) -> Result<()> {
        ctx.raw_data_walker.advance_set(*raw);
        Ok(())
    }
}

impl<const A: BufferAttribute, const S: usize> CommandParameter<sf::Buffer<A, S>> for sf::Buffer<A, S> {
    fn after_request_read(ctx: &mut ServerContext) -> Result<Self> {
        ctx.ctx.pop_buffer(&mut ctx.raw_data_walker)
    }

    fn before_response_write(_buffer: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn after_response_write(_buffer: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }
}

impl<const M: HandleMode> CommandParameter<sf::Handle<M>> for sf::Handle<M> {
    fn after_request_read(_ctx: &mut ServerContext) -> Result<Self> {
        // TODO: pop copy/move
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn before_response_write(handle: &Self, ctx: &mut ServerContext) -> Result<()> {
        ctx.ctx.out_params.push_handle(*handle)
    }

    fn after_response_write(_handle: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Ok(())
    }
}

impl CommandParameter<sf::ProcessId> for sf::ProcessId {
    fn after_request_read(ctx: &mut ServerContext) -> Result<Self> {
        if ctx.ctx.in_params.send_process_id {
            // TODO: is this really how process ID works? (is the in raw u64 just placeholder data?)
            let _ = ctx.raw_data_walker.advance_get::<u64>();
            Ok(sf::ProcessId::from(ctx.ctx.in_params.process_id)) 
        }
        else {
            Err(results::hipc::ResultUnsupportedOperation::make())
        }
    }

    fn before_response_write(_process_id: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn after_response_write(_process_id: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }
}

impl CommandParameter<mem::Shared<dyn sf::IObject>> for mem::Shared<dyn sf::IObject> {
    fn after_request_read(_ctx: &mut ServerContext) -> Result<Self> {
        Err(results::hipc::ResultUnsupportedOperation::make())
    }

    fn before_response_write(session: &Self, ctx: &mut ServerContext) -> Result<()> {
        let (server_handle, client_handle) = svc::create_session(false, 0)?;
        ctx.ctx.out_params.push_handle(sf::MoveHandle::from(client_handle))?;
        session.get().set_info(ObjectInfo::new());
        ctx.new_sessions.push(ServerHolder::new_session(server_handle, session.clone()));
        Ok(())
    }

    fn after_response_write(_session: &Self, _ctx: &mut ServerContext) -> Result<()> {
        Ok(())
    }
}

pub trait IServerObject: sf::IObject {
    fn new() -> Self where Self: Sized;
}

pub trait IMitmServerObject: sf::IObject {
    fn new(info: sm::MitmProcessInfo) -> Self where Self: Sized;
}

fn create_server_object_impl<S: IServerObject + 'static>() -> mem::Shared<dyn sf::IObject> {
    mem::Shared::new(S::new())
}

fn create_mitm_server_object_impl<S: IMitmServerObject + 'static>(info: sm::MitmProcessInfo) -> mem::Shared<dyn sf::IObject> {
    mem::Shared::new(S::new(info))
}

pub type NewServerFn = fn() -> mem::Shared<dyn sf::IObject>;
pub type NewMitmServerFn = fn(sm::MitmProcessInfo) -> mem::Shared<dyn sf::IObject>;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum WaitHandleType {
    Server,
    Session
}

pub struct ServerHolder {
    pub server: mem::Shared<dyn sf::IObject>,
    pub info: ObjectInfo,
    pub new_server_fn: Option<NewServerFn>,
    pub new_mitm_server_fn: Option<NewMitmServerFn>,
    pub handle_type: WaitHandleType,
    pub mitm_forward_info: ObjectInfo,
    pub is_mitm_service: bool,
    pub service_name: sm::ServiceName
}

impl ServerHolder {
    pub fn new_server_session<S: IServerObject + 'static>(handle: svc::Handle) -> Self {
        Self { server: mem::Shared::new(S::new()), info: ObjectInfo::from_handle(handle), new_server_fn: None, new_mitm_server_fn: None, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::new(), is_mitm_service: false, service_name: sm::ServiceName::empty() } 
    }

    pub fn new_session(handle: svc::Handle, object: mem::Shared<dyn sf::IObject>) -> Self {
        Self { server: object, info: ObjectInfo::from_handle(handle), new_server_fn: None, new_mitm_server_fn: None, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::new(), is_mitm_service: false, service_name: sm::ServiceName::empty() } 
    }
    
    pub fn new_server<S: IServerObject + 'static>(handle: svc::Handle, service_name: sm::ServiceName) -> Self {
        Self { server: mem::Shared::<S>::empty(), info: ObjectInfo::from_handle(handle), new_server_fn: Some(create_server_object_impl::<S>), new_mitm_server_fn: None, handle_type: WaitHandleType::Server, mitm_forward_info: ObjectInfo::new(), is_mitm_service: false, service_name: service_name } 
    }

    pub fn new_mitm_server<S: IMitmServerObject + 'static>(handle: svc::Handle, service_name: sm::ServiceName) -> Self {
        Self { server: mem::Shared::<S>::empty(), info: ObjectInfo::from_handle(handle), new_server_fn: None, new_mitm_server_fn: Some(create_mitm_server_object_impl::<S>), handle_type: WaitHandleType::Server, mitm_forward_info: ObjectInfo::new(), is_mitm_service: true, service_name: service_name } 
    }

    pub fn make_new_session(&self, handle: svc::Handle) -> Result<Self> {
        let new_fn = self.get_new_server_fn()?;
        Ok(Self { server: (new_fn)(), info: ObjectInfo::from_handle(handle), new_server_fn: self.new_server_fn, new_mitm_server_fn: self.new_mitm_server_fn, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::new(), is_mitm_service: self.is_mitm_service, service_name: sm::ServiceName::empty() })
    }

    pub fn make_new_mitm_session(&self, handle: svc::Handle, forward_handle: svc::Handle, info: sm::MitmProcessInfo) -> Result<Self> {
        let new_mitm_fn = self.get_new_mitm_server_fn()?;
        Ok(Self { server: (new_mitm_fn)(info), info: ObjectInfo::from_handle(handle), new_server_fn: self.new_server_fn, new_mitm_server_fn: self.new_mitm_server_fn, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::from_handle(forward_handle), is_mitm_service: self.is_mitm_service, service_name: sm::ServiceName::empty() })
    }

    pub fn clone_self(&self, handle: svc::Handle, forward_handle: svc::Handle) -> Result<Self> {
        let mut object_info = self.info;
        object_info.handle = handle;
        let mut mitm_fwd_info = self.mitm_forward_info;
        mitm_fwd_info.handle = forward_handle;
        Ok(Self { server: self.server.clone(), info: object_info, new_server_fn: self.new_server_fn, new_mitm_server_fn: self.new_mitm_server_fn, handle_type: WaitHandleType::Session, mitm_forward_info: mitm_fwd_info, is_mitm_service: forward_handle != 0, service_name: sm::ServiceName::empty() })
    }

    pub fn get_new_server_fn(&self) -> Result<NewServerFn> {
        match self.new_server_fn {
            Some(new_server_fn) => Ok(new_server_fn),
            None => Err(results::hipc::ResultSessionClosed::make())
        }
    }

    pub fn get_new_mitm_server_fn(&self) -> Result<NewMitmServerFn> {
        match self.new_mitm_server_fn {
            Some(new_mitm_server_fn) => Ok(new_mitm_server_fn),
            None => Err(results::hipc::ResultSessionClosed::make())
        }
    }

    pub fn close(&mut self) -> Result<()> {
        if !self.service_name.is_empty() {
            let sm = service::tipc::new_named_port_object::<sm::UserInterface>()?;
            match self.is_mitm_service {
                true => sm.get().atmosphere_uninstall_mitm(self.service_name)?,
                false => sm.get().unregister_service(self.service_name)?
            };
            sm.get().detach_client(sf::ProcessId::new())?;
        }

        // Don't close our session like a normal one (like the forward session below) as we allocated the object IDs ourselves, the only thing we do have to close is the handle
        if self.info.owns_handle {
            svc::close_handle(self.info.handle)?;
        }
        sf::Session::from(self.mitm_forward_info).close();
        Ok(())
    }
}

impl Drop for ServerHolder {
    fn drop(&mut self) {
        self.close().unwrap();
    }
}

pub trait IService: IServerObject {
    fn get_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

pub trait INamedPort: IServerObject {
    fn get_port_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

// TODO: implement mitms, implement ServerManager, merge it with cmif one...?