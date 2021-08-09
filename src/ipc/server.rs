use crate::result::*;
use crate::results;
use crate::svc;
use crate::wait;
use crate::ipc::sf::IObject;
use crate::ipc::sf::hipc::IHipcManager;
use crate::ipc::sf::hipc::IMitmQueryServer;
use crate::service;
use crate::service::sm;
use crate::service::sm::IUserInterface;
use crate::mem;
use super::*;
use alloc::vec::Vec;

// TODO: tipc support, implement remaining control commands

const MAX_COUNT: usize = wait::MAX_OBJECT_COUNT as usize;

pub struct ServerContext<'a> {
    pub ctx: &'a mut CommandContext,
    pub raw_data_walker: DataWalker,
    pub domain_table: mem::Shared<DomainTable>,
    pub new_sessions: &'a mut Vec<ServerHolder>
}

impl<'a> ServerContext<'a> {
    pub const fn new(ctx: &'a mut CommandContext, raw_data_walker: DataWalker, domain_table: mem::Shared<DomainTable>, new_sessions: &'a mut Vec<ServerHolder>) -> Self {
        Self { ctx: ctx, raw_data_walker: raw_data_walker, domain_table: domain_table, new_sessions: new_sessions }
    }
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
            // TODO: is this really how process ID works? (is the in raw u64 just placeholder data, is it always present...?)
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
        if ctx.ctx.object_info.is_domain() {
            let domain_object_id = ctx.domain_table.get().allocate_id()?;
            ctx.ctx.out_params.push_domain_object(domain_object_id)?;
            session.get().set_info(ObjectInfo::new());
            ctx.domain_table.get().domains.push(ServerHolder::new_domain_session(0, domain_object_id, session.clone()));
            Ok(())
        }
        else {
            let (server_handle, client_handle) = svc::create_session(false, 0)?;
            ctx.ctx.out_params.push_handle(sf::MoveHandle::from(client_handle))?;
            session.get().set_info(ObjectInfo::new());
            ctx.new_sessions.push(ServerHolder::new_session(server_handle, session.clone()));
            Ok(())
        }
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

pub struct DomainTable {
    pub table: Vec<cmif::DomainObjectId>,
    pub domains: Vec<ServerHolder>,
}

impl DomainTable {
    pub fn new() -> Self {
        Self { table: Vec::new(), domains: Vec::new() }
    }

    pub fn allocate_id(&mut self) -> Result<cmif::DomainObjectId> {
        let mut current_id: cmif::DomainObjectId = 1;
        loop {
            // Note: fix potential infinite loops here?
            if !self.table.contains(&current_id) {
                self.table.push(current_id);
                return Ok(current_id);
            }
            current_id += 1;
        }
    }

    pub fn allocate_specific_id(&mut self, specific_domain_object_id: cmif::DomainObjectId) -> Result<cmif::DomainObjectId> {
        if !self.table.contains(&specific_domain_object_id) {
            self.table.push(specific_domain_object_id);
            return Ok(specific_domain_object_id);
        }
        
        Err(results::lib::ipc::ResultObjectIdAlreadyAllocated::make())
    }

    pub fn find_domain(&mut self, id: cmif::DomainObjectId) -> Result<mem::Shared<dyn sf::IObject>> {
        for holder in &self.domains {
            if holder.info.domain_object_id == id {
                return Ok(holder.server.clone());
            }
        }

        Err(results::lib::ipc::ResultDomainNotFound::make())
    }
    
    pub fn deallocate_domain(&mut self, domain_object_id: cmif::DomainObjectId) {
        self.table.retain(|&id| id != domain_object_id);
        self.domains.retain(|holder| holder.info.domain_object_id != domain_object_id);
    }
}

pub struct ServerHolder {
    pub server: mem::Shared<dyn sf::IObject>,
    pub info: ObjectInfo,
    pub new_server_fn: Option<NewServerFn>,
    pub new_mitm_server_fn: Option<NewMitmServerFn>,
    pub handle_type: WaitHandleType,
    pub mitm_forward_info: ObjectInfo,
    pub is_mitm_service: bool,
    pub service_name: sm::ServiceName,
    pub domain_table: mem::Shared<DomainTable>
}

impl ServerHolder {
    pub fn new_server_session<S: IServerObject + 'static>(handle: svc::Handle) -> Self {
        Self { server: mem::Shared::new(S::new()), info: ObjectInfo::from_handle(handle), new_server_fn: None, new_mitm_server_fn: None, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::new(), is_mitm_service: false, service_name: sm::ServiceName::empty(), domain_table: mem::Shared::empty() } 
    }

    pub fn new_session(handle: svc::Handle, object: mem::Shared<dyn sf::IObject>) -> Self {
        Self { server: object, info: ObjectInfo::from_handle(handle), new_server_fn: None, new_mitm_server_fn: None, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::new(), is_mitm_service: false, service_name: sm::ServiceName::empty(), domain_table: mem::Shared::empty() } 
    }

    pub fn new_domain_session(handle: svc::Handle, domain_object_id: cmif::DomainObjectId, object: mem::Shared<dyn sf::IObject>) -> Self {
        Self { server: object, info: ObjectInfo::from_domain_object_id(handle, domain_object_id), new_server_fn: None, new_mitm_server_fn: None, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::new(), is_mitm_service: false, service_name: sm::ServiceName::empty(), domain_table: mem::Shared::empty() } 
    }
    
    pub fn new_server<S: IServerObject + 'static>(handle: svc::Handle, service_name: sm::ServiceName) -> Self {
        Self { server: mem::Shared::<S>::empty(), info: ObjectInfo::from_handle(handle), new_server_fn: Some(create_server_object_impl::<S>), new_mitm_server_fn: None, handle_type: WaitHandleType::Server, mitm_forward_info: ObjectInfo::new(), is_mitm_service: false, service_name: service_name, domain_table: mem::Shared::empty() } 
    }

    pub fn new_mitm_server<S: IMitmServerObject + 'static>(handle: svc::Handle, service_name: sm::ServiceName) -> Self {
        Self { server: mem::Shared::<S>::empty(), info: ObjectInfo::from_handle(handle), new_server_fn: None, new_mitm_server_fn: Some(create_mitm_server_object_impl::<S>), handle_type: WaitHandleType::Server, mitm_forward_info: ObjectInfo::new(), is_mitm_service: true, service_name: service_name, domain_table: mem::Shared::empty() } 
    }

    pub fn make_new_session(&self, handle: svc::Handle) -> Result<Self> {
        let new_fn = self.get_new_server_fn()?;
        Ok(Self { server: (new_fn)(), info: ObjectInfo::from_handle(handle), new_server_fn: self.new_server_fn, new_mitm_server_fn: self.new_mitm_server_fn, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::new(), is_mitm_service: self.is_mitm_service, service_name: sm::ServiceName::empty(), domain_table: mem::Shared::empty() })
    }

    pub fn make_new_mitm_session(&self, handle: svc::Handle, forward_handle: svc::Handle, info: sm::MitmProcessInfo) -> Result<Self> {
        let new_mitm_fn = self.get_new_mitm_server_fn()?;
        Ok(Self { server: (new_mitm_fn)(info), info: ObjectInfo::from_handle(handle), new_server_fn: self.new_server_fn, new_mitm_server_fn: self.new_mitm_server_fn, handle_type: WaitHandleType::Session, mitm_forward_info: ObjectInfo::from_handle(forward_handle), is_mitm_service: self.is_mitm_service, service_name: sm::ServiceName::empty(), domain_table: mem::Shared::empty() })
    }

    pub fn clone_self(&self, handle: svc::Handle, forward_handle: svc::Handle) -> Result<Self> {
        let mut object_info = self.info;
        object_info.handle = handle;
        let mut mitm_fwd_info = self.mitm_forward_info;
        mitm_fwd_info.handle = forward_handle;
        Ok(Self { server: self.server.clone(), info: object_info, new_server_fn: self.new_server_fn, new_mitm_server_fn: self.new_mitm_server_fn, handle_type: WaitHandleType::Session, mitm_forward_info: mitm_fwd_info, is_mitm_service: forward_handle != 0, service_name: sm::ServiceName::empty(), domain_table: self.domain_table.clone() })
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

    pub fn convert_to_domain(&mut self) -> Result<cmif::DomainObjectId> {
        // Check that we're not already a domain
        result_return_if!(self.info.is_domain(), 0xBADE);

        // Since we're a base domain object now, create a domain table
        self.domain_table = mem::Shared::new(DomainTable::new());

        let domain_object_id = match self.is_mitm_service {
            true => {
                let forward_object_id = self.mitm_forward_info.convert_current_object_to_domain()?;
                self.mitm_forward_info.domain_object_id = forward_object_id;
                self.domain_table.get().allocate_specific_id(forward_object_id)?
            },
            false => self.domain_table.get().allocate_id()?
        };

        self.info.domain_object_id = domain_object_id;
        Ok(domain_object_id)
    }

    pub fn close(&mut self) -> Result<()> {
        if !self.service_name.is_empty() {
            let sm = service::new_named_port_object::<sm::UserInterface>()?;
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

pub struct HipcManager<'a> {
    session: sf::Session,
    server_holder: &'a mut ServerHolder,
    pointer_buf_size: usize,
    pub cloned_object_server_handle: svc::Handle,
    pub cloned_object_forward_handle: svc::Handle
}

impl<'a> HipcManager<'a> {
    pub fn new(server_holder: &'a mut ServerHolder, pointer_buf_size: usize) -> Self {
        Self { session: sf::Session::new(), server_holder: server_holder, pointer_buf_size: pointer_buf_size, cloned_object_server_handle: 0, cloned_object_forward_handle: 0 }
    }

    pub fn has_cloned_object(&self) -> bool {
        self.cloned_object_server_handle != 0
    }

    pub fn clone_object(&self) -> Result<ServerHolder> {
        self.server_holder.clone_self(self.cloned_object_server_handle, self.cloned_object_forward_handle)
    }
}

impl<'a> IHipcManager for HipcManager<'a> {
    fn convert_current_object_to_domain(&mut self) -> Result<cmif::DomainObjectId> {
        self.server_holder.convert_to_domain()
    }

    fn copy_from_current_domain(&mut self, _domain_object_id: cmif::DomainObjectId) -> Result<sf::MoveHandle> {
        // TODO
        Err(results::lib::ResultNotImplemented::make())
    }

    fn clone_current_object(&mut self) -> Result<sf::MoveHandle> {
        let (server_handle, client_handle) = svc::create_session(false, 0)?;
        
        let mut forward_handle: svc::Handle = 0;
        if self.server_holder.is_mitm_service {
            let fwd_handle = self.server_holder.mitm_forward_info.clone_current_object()?;
            forward_handle = fwd_handle.handle;
        }

        self.cloned_object_server_handle = server_handle;
        self.cloned_object_forward_handle = forward_handle;
        Ok(sf::Handle::from(client_handle))
    }

    fn query_pointer_buffer_size(&mut self) -> Result<u16> {
        Ok(self.pointer_buf_size as u16)
    }

    fn clone_current_object_ex(&mut self, _tag: u32) -> Result<sf::MoveHandle> {
        // The tag value is unused anyways :P
        self.clone_current_object()
    }
}

impl<'a> sf::IObject for HipcManager<'a> {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(convert_current_object_to_domain: 0),
            ipc_cmif_interface_make_command_meta!(copy_from_current_domain: 1),
            ipc_cmif_interface_make_command_meta!(clone_current_object: 2),
            ipc_cmif_interface_make_command_meta!(query_pointer_buffer_size: 3),
            ipc_cmif_interface_make_command_meta!(clone_current_object_ex: 4)
        ]
    }
}

pub struct MitmQueryServer<S: IMitmService> {
    session: sf::Session,
    phantom: core::marker::PhantomData<S>
}

impl<S: IMitmService> IMitmQueryServer for MitmQueryServer<S> {
    fn should_mitm(&mut self, info: sm::MitmProcessInfo) -> Result<bool> {
        Ok(S::should_mitm(info))
    }
}

impl<S: IMitmService> sf::IObject for MitmQueryServer<S> {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        vec! [
            ipc_cmif_interface_make_command_meta!(should_mitm: 65000)
        ]
    }
}

impl<S: IMitmService> IServerObject for MitmQueryServer<S> {
    fn new() -> Self {
        Self { session: sf::Session::new(), phantom: core::marker::PhantomData }
    }
}

pub trait IService: IServerObject {
    fn get_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

pub trait IMitmService: IMitmServerObject {
    fn get_name() -> &'static str;
    fn should_mitm(info: sm::MitmProcessInfo) -> bool;
}

pub trait INamedPort: IServerObject {
    fn get_port_name() -> &'static str;
    fn get_max_sesssions() -> i32;
}

// TODO: use const generics to reduce memory usage, like libstratosphere does?

pub struct ServerManager<const P: usize> {
    server_holders: Vec<ServerHolder>,
    wait_handles: [svc::Handle; MAX_COUNT],
    pointer_buffer: [u8; P]
}

impl<const P: usize> ServerManager<P> {
    pub fn new() -> Result<Self> {
        Ok(Self { server_holders: Vec::new(), wait_handles: [0; MAX_COUNT], pointer_buffer: [0; P] })
    }
    
    #[inline(always)]
    fn prepare_wait_handles(&mut self) -> &[svc::Handle] {
        let mut handles_index: usize = 0;
        for server_holder in &mut self.server_holders {
            let server_info = server_holder.info;
            if server_info.handle != 0 {
                self.wait_handles[handles_index] = server_info.handle;
                handles_index += 1;
            }
        }

        unsafe { core::slice::from_raw_parts(self.wait_handles.as_ptr(), handles_index) }
    }

    #[inline(always)]
    fn handle_request_command(&mut self, ctx: &mut CommandContext, rq_id: u32, command_type: cmif::CommandType, domain_command_type: cmif::DomainCommandType, ipc_buf_backup: &[u8], domain_table: mem::Shared<DomainTable>) -> Result<()> {
        let is_domain = ctx.object_info.is_domain();
        let domain_table_clone = domain_table.clone();
        let mut do_handle_request = || -> Result<()> {
            let mut new_sessions: Vec<ServerHolder> = Vec::new();
            for server_holder in &mut self.server_holders {
                let server_info = server_holder.info;
                if server_info.handle == ctx.object_info.handle {
                    let send_to_forward_handle = || -> Result<()> {
                        let ipc_buf = get_ipc_buffer();
                        unsafe {
                            core::ptr::copy(ipc_buf_backup.as_ptr(), ipc_buf, ipc_buf_backup.len());
                        }
                        // Let the original service take care of the command for us.
                        svc::send_sync_request(server_holder.mitm_forward_info.handle)
                    };
                    
                    let target_server = match is_domain {
                        true => match ctx.object_info.owns_handle {
                            true => server_holder.server.clone(),
                            false => domain_table_clone.get().find_domain(ctx.object_info.domain_object_id)?
                        },
                        false => server_holder.server.clone()
                    };
                    // Nothing done on success here, as if the command succeeds it will automatically respond by itself.
                    let mut command_found = false;
                    for command in target_server.get().get_command_table() {
                        if command.matches(ctx.object_info.protocol, rq_id) {
                            command_found = true;
                            let mut server_ctx = ServerContext::new(ctx, DataWalker::empty(), domain_table_clone.clone(), &mut new_sessions);
                            if let Err(rc) = target_server.get().call_self_command(command.command_fn, &mut server_ctx) {
                                if server_holder.is_mitm_service && results::sm::mitm::ResultShouldForwardToSession::matches(rc) {
                                    if let Err(rc) = send_to_forward_handle() {
                                        cmif::server::write_request_command_response_on_ipc_buffer(ctx, rc, command_type);
                                    }
                                }
                                else {
                                    cmif::server::write_request_command_response_on_ipc_buffer(ctx, rc, command_type);
                                }
                            }
                        }
                    }
                    if !command_found {
                        if server_holder.is_mitm_service {
                            if let Err(rc) = send_to_forward_handle() {
                                cmif::server::write_request_command_response_on_ipc_buffer(ctx, rc, command_type);
                            }
                        }
                        else {
                            cmif::server::write_request_command_response_on_ipc_buffer(ctx, results::cmif::ResultInvalidCommandRequestId::make(), command_type);
                        }
                    }
                    break;
                }
            }

            self.server_holders.append(&mut new_sessions);

            Ok(())
        };

        match domain_command_type {
            cmif::DomainCommandType::Invalid => {
                // Invalid command type might mean that the session isn't a domain :P
                match is_domain {
                    false => do_handle_request()?,
                    true => return Err(results::lib::ipc::ResultInvalidDomainCommandType::make())
                };
            },
            cmif::DomainCommandType::SendMessage => do_handle_request()?,
            cmif::DomainCommandType::Close => {
                if !ctx.object_info.owns_handle {
                    domain_table.get().deallocate_domain(ctx.object_info.domain_object_id);
                }
                else {
                    // TODO: Abort? Error?
                }
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn handle_control_command(&mut self, ctx: &mut CommandContext, rq_id: u32, command_type: cmif::CommandType) -> Result<()> {
        for server_holder in &mut self.server_holders {
            let server_info = server_holder.info;
            if server_info.handle == ctx.object_info.handle {
                let mut hipc_manager = HipcManager::new(server_holder, P);
                // Nothing done on success here, as if the command succeeds it will automatically respond by itself.
                let mut command_found = false;
                for command in hipc_manager.get_command_table() {
                    if command.matches(CommandProtocol::Cmif, rq_id) {
                        command_found = true;
                        let mut unused_new_sessions: Vec<ServerHolder> = Vec::new();
                        let unused_domain_table = mem::Shared::empty();
                        let mut server_ctx = ServerContext::new(ctx, DataWalker::empty(), unused_domain_table, &mut unused_new_sessions);
                        if let Err(rc) = hipc_manager.call_self_command(command.command_fn, &mut server_ctx) {
                            cmif::server::write_control_command_response_on_ipc_buffer(ctx, rc, command_type);
                        }
                    }
                }
                if !command_found {
                    cmif::server::write_control_command_response_on_ipc_buffer(ctx, results::cmif::ResultInvalidCommandRequestId::make(), command_type);
                }

                if hipc_manager.has_cloned_object() {
                    let cloned_holder = hipc_manager.clone_object()?;
                    self.server_holders.push(cloned_holder);
                }
                break;
            }
        }

        Ok(())
    }

    fn process_signaled_handle(&mut self, handle: svc::Handle) -> Result<()> {
        let mut server_found = false;
        let mut index: usize = 0;
        let mut should_close_session = false;
        let mut new_sessions: Vec<ServerHolder> = Vec::new();

        let mut ctx = CommandContext::empty();
        let mut command_type = cmif::CommandType::Invalid;
        let mut domain_cmd_type = cmif::DomainCommandType::Invalid;
        let mut rq_id: u32 = 0;
        let mut ipc_buf_backup: [u8; 0x100] = [0; 0x100];
        let mut domain_table: mem::Shared<DomainTable> = mem::Shared::empty();

        for server_holder in &mut self.server_holders {
            let server_info = server_holder.info;
            if server_info.handle == handle {
                server_found = true;
                match server_holder.handle_type {
                    WaitHandleType::Session => {
                        if P > 0 {
                            // Send our pointer buffer as a C descriptor for kernel - why are Pointer buffers so fucking weird?
                            let mut tmp_ctx = CommandContext::new_client(server_info);
                            tmp_ctx.add_receive_static(ReceiveStaticDescriptor::new(self.pointer_buffer.as_ptr(), P))?;
                            cmif::client::write_command_on_ipc_buffer(&mut tmp_ctx, cmif::CommandType::Invalid, 0);
                        }

                        match svc::reply_and_receive(&handle, 1, 0, -1) {
                            Err(rc) => {
                                if results::os::ResultSessionClosed::matches(rc) {
                                    should_close_session = true;
                                    break;
                                }
                                else {
                                    return Err(rc);
                                }
                            },
                            _ => {}
                        };

                        unsafe { core::ptr::copy(get_ipc_buffer(), ipc_buf_backup.as_mut_ptr(), ipc_buf_backup.len()) };

                        ctx = CommandContext::new_server(server_info, self.pointer_buffer.as_mut_ptr());
                        command_type = cmif::server::read_command_from_ipc_buffer(&mut ctx);
                        match command_type {
                            cmif::CommandType::Request | cmif::CommandType::RequestWithContext => {
                                match cmif::server::read_request_command_from_ipc_buffer(&mut ctx) {
                                    Ok((request_id, domain_command_type, domain_object_id)) => {
                                        let mut base_info = server_info;
                                        if server_info.is_domain() {
                                            // This is a domain request
                                            base_info.domain_object_id = domain_object_id;
                                            base_info.owns_handle = server_info.domain_object_id == domain_object_id;
                                        }
                                        ctx.object_info = base_info;
                                        domain_cmd_type = domain_command_type;
                                        rq_id = request_id;
                                        domain_table = server_holder.domain_table.clone();
                                    },
                                    Err(rc) => return Err(rc)
                                };
                            },
                            cmif::CommandType::Control | cmif::CommandType::ControlWithContext => {
                                match cmif::server::read_control_command_from_ipc_buffer(&mut ctx) {
                                    Ok(control_rq_id) => {
                                        rq_id = control_rq_id as u32;
                                    },
                                    Err(rc) => return Err(rc),
                                };
                            },
                            _ => return Err(results::lib::ipc::ResultInvalidCommandType::make())
                        }
                    },
                    WaitHandleType::Server => {
                        let new_handle = svc::accept_session(handle)?;

                        if server_holder.is_mitm_service {
                            let sm = service::new_named_port_object::<sm::UserInterface>()?;
                            let (info, session_handle) = sm.get().atmosphere_acknowledge_mitm_session(server_holder.service_name)?;
                            new_sessions.push(server_holder.make_new_mitm_session(new_handle, session_handle.handle, info)?);
                            sm.get().detach_client(sf::ProcessId::new())?;
                        }
                        else {
                            new_sessions.push(server_holder.make_new_session(new_handle)?);
                        }
                    }
                };
                break;
            }
            index += 1;
        }

        let reply_impl = || -> Result<()> {
            match svc::reply_and_receive(&handle, 0, handle, 0) {
                Err(rc) => {
                    if results::os::ResultTimeout::matches(rc) || results::os::ResultSessionClosed::matches(rc) {
                        Ok(())
                    }
                    else {
                        Err(rc)
                    }
                },
                _ => Ok(())
            }
        };

        match command_type {
            cmif::CommandType::Request | cmif::CommandType::RequestWithContext => {
                self.handle_request_command(&mut ctx, rq_id, command_type, domain_cmd_type, &ipc_buf_backup, domain_table)?;
                reply_impl()?;
            },
            cmif::CommandType::Control | cmif::CommandType::ControlWithContext => {
                self.handle_control_command(&mut ctx, rq_id, command_type)?;
                reply_impl()?;
            },
            cmif::CommandType::Close => {
                cmif::server::write_close_command_response_on_ipc_buffer(&mut ctx);
                reply_impl()?;
                should_close_session = true;
            }
            _ => {
                // Do nothing, since it might not be set at all without having failed (for instance, if a new session was accepted)
                // If this actually failed and it reached this point server_found will be false, which is handled below
            }
        };

        if should_close_session {
            self.server_holders.remove(index);
        }

        self.server_holders.append(&mut new_sessions);

        match server_found {
            true => Ok(()),
            false => Err(results::lib::ipc::ResultSignaledServerNotFound::make())
        }
    }
    
    pub fn register_server<S: IServerObject + 'static>(&mut self, handle: svc::Handle, service_name: sm::ServiceName) {
        self.server_holders.push(ServerHolder::new_server::<S>(handle, service_name));
    }

    pub fn register_mitm_server<S: IMitmServerObject + 'static>(&mut self, handle: svc::Handle, service_name: sm::ServiceName) {
        self.server_holders.push(ServerHolder::new_mitm_server::<S>(handle, service_name));
    }
    
    pub fn register_session<S: IServerObject + 'static>(&mut self, handle: svc::Handle) {
        self.server_holders.push(ServerHolder::new_server_session::<S>(handle));
    }
    
    pub fn register_service_server<S: IService + 'static>(&mut self) -> Result<()> {
        let service_name = sm::ServiceName::new(S::get_name());
        
        let sm = service::new_named_port_object::<sm::UserInterface>()?;
        let service_handle = sm.get().register_service(service_name, false, S::get_max_sesssions())?;
        self.register_server::<S>(service_handle.handle, service_name);
        sm.get().detach_client(sf::ProcessId::new())?;
        Ok(())
    }
    
    pub fn register_mitm_service_server<S: IMitmService + 'static>(&mut self) -> Result<()> {
        let service_name = sm::ServiceName::new(S::get_name());

        let sm = service::new_named_port_object::<sm::UserInterface>()?;
        let (mitm_handle, query_handle) = sm.get().atmosphere_install_mitm(service_name)?;

        self.register_mitm_server::<S>(mitm_handle.handle, service_name);
        self.register_session::<MitmQueryServer<S>>(query_handle.handle);

        sm.get().atmosphere_clear_future_mitm(service_name)?;
        sm.get().detach_client(sf::ProcessId::new())?;
        Ok(())
    }

    pub fn register_named_port_server<S: INamedPort + 'static>(&mut self) -> Result<()> {
        let port_handle = svc::manage_named_port(S::get_port_name().as_ptr(), S::get_max_sesssions())?;

        self.register_server::<S>(port_handle, sm::ServiceName::empty());
        Ok(())
    }

    pub fn process(&mut self) -> Result<()> {
        let handles = self.prepare_wait_handles();
        let index = wait::wait_handles(handles, -1)?;

        let signaled_handle = self.wait_handles[index];
        self.process_signaled_handle(signaled_handle)?;

        Ok(())
    }

    pub fn loop_process(&mut self) -> Result<()> {
        loop {
            match self.process() {
                Err(rc) => {
                    // TODO: handle results properly here
                    if results::os::ResultOperationCanceled::matches(rc) {
                        break;
                    }
                    return Err(rc);
                },
                _ => {}
            }
        }

        Ok(())
    }
}