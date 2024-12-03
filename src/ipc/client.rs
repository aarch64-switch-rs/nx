use sf::hipc;

use super::*;
use crate::mem;

pub trait RequestCommandParameter {
    fn before_request_write(var: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()>;
    fn before_send_sync_request(var: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()>;
}

pub trait ResponseCommandParameter<O> {
    fn after_response_read(walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<O>;
}

/*
impl<T: Copy> RequestCommandParameter for T {
    default fn before_request_write(_raw: &Self, walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        walker.advance::<Self>();
        Ok(())
    }

    default fn before_send_sync_request(raw: &Self, walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        walker.advance_set(*raw);
        Ok(())
    }
} 


impl<T: Copy> ResponseCommandParameter<T> for T {
    default fn after_response_read(walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<Self> {
        Ok(walker.advance_get())
    }
}*/

impl<const A: BufferAttribute, T> RequestCommandParameter for sf::Buffer<A, T> {
    fn before_request_write(buffer: &Self, _walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.add_buffer(buffer)
    }

    fn before_send_sync_request(_buffer: &Self, _walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        Ok(())
    }
}
     

impl<const A: BufferAttribute, T> !ResponseCommandParameter<sf::Buffer<A, T>> for sf::Buffer<A, T> {}

impl<const M: HandleMode> RequestCommandParameter for sf::Handle<M> {
    fn before_request_write(handle: &Self, _walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.in_params.add_handle(handle.clone())
    }

    fn before_send_sync_request(_handle: &Self, _walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        Ok(())
    }
}
 


impl<const M: HandleMode> ResponseCommandParameter<sf::Handle<M>> for sf::Handle<M> {
    fn after_response_read(_walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<Self> {
        ctx.out_params.pop_handle()
    }
}


impl RequestCommandParameter for sf::ProcessId {
    fn before_request_write(_process_id: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        // signal to the kernel that we need a PID injected into the request
        ctx.in_params.send_process_id = true;
        if ctx.object_info.uses_cmif_protocol(){
            // TIPC doesn't set this placeholder space for process IDs
            walker.advance::<u64>();
        }
        Ok(())
    }

    fn before_send_sync_request(process_id: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        // Same as above
        if ctx.object_info.uses_cmif_protocol() {
            walker.advance_set(process_id.process_id);
        }
        Ok(())
    }
}

impl !ResponseCommandParameter<sf::ProcessId> for sf::ProcessId {}

impl RequestCommandParameter for sf::AppletResourceUserId {
    fn before_request_write(_aruid: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        result_return_unless!(ctx.object_info.uses_cmif_protocol(), hipc::rc::ResultUnsupportedOperation);

        // signal to the kernel that we need a PID injected into the request
        ctx.in_params.send_process_id = true;
        walker.advance::<u64>();
        Ok(())
    }

    fn before_send_sync_request(aruid: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        result_return_unless!(ctx.object_info.uses_cmif_protocol(), hipc::rc::ResultUnsupportedOperation);
        // write the aruid into the slot
        walker.advance_set(aruid.aruid);
        Ok(())
    }
}

impl !ResponseCommandParameter<sf::AppletResourceUserId> for sf::AppletResourceUserId {}

impl<S: IClientObject + ?Sized> RequestCommandParameter for mem::Shared<S> {
    fn before_request_write(session: &Self, _walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.in_params.add_object(session.lock().get_session().object_info)
    }

    fn before_send_sync_request(_session: &Self, _walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        Ok(())
    }
}

impl<S: IClientObject + 'static + Sized> ResponseCommandParameter<mem::Shared<S>> for mem::Shared<S> {
    fn after_response_read(_walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<Self> {
        let object_info = ctx.pop_object()?;
        Ok(mem::Shared::new(S::new(sf::Session::from(object_info))))
    }
}

pub trait IClientObject {
    fn new(session: sf::Session) -> Self where Self: Sized;

    fn get_session(&self) -> &sf::Session;

    fn get_session_mut(&mut self) -> &mut sf::Session;

    fn get_info(&self) -> ObjectInfo {
        self.get_session().object_info
    }

    fn set_info(&mut self, info: ObjectInfo) {
        self.get_session_mut().set_info(info);
    }

    fn convert_to_domain(&mut self) -> Result<()> {
        self.get_session_mut().convert_to_domain()
    }

    fn query_own_pointer_buffer_size(&mut self) -> Result<u16> {
        self.get_info().query_pointer_buffer_size()
    }

    fn close_session(&mut self) {
        self.get_session_mut().close()
    }

    fn is_valid(&mut self) -> bool {
        self.get_info().is_valid()
    }
    
    fn is_domain(&mut self) -> bool {
        self.get_info().is_domain()
    }
}