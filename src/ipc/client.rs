use super::*;
use crate::results;
use crate::ipc::sf;
use crate::service;
use crate::mem;

pub trait CommandParameter<O> {
    fn before_request_write(var: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()>;
    fn before_send_sync_request(var: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()>;
    fn after_response_read(walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<O>;
}

impl<T: Copy> CommandParameter<T> for T {
    default fn before_request_write(_raw: &Self, walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        walker.advance::<Self>();
        Ok(())
    }

    default fn before_send_sync_request(raw: &Self, walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        walker.advance_set(*raw);
        Ok(())
    }

    default fn after_response_read(walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<Self> {
        Ok(walker.advance_get())
    }
}

impl<const A: BufferAttribute, const S: usize> CommandParameter<sf::Buffer<A, S>> for sf::Buffer<A, S> {
    fn before_request_write(buffer: &Self, _walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.add_buffer(buffer.clone())
    }

    fn before_send_sync_request(_buffer: &Self, _walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        Ok(())
    }

    fn after_response_read(_walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<Self> {
        // Buffers aren't returned as output variables - the buffer sent as input (with Out attribute) will contain the output data
        Err(results::hipc::ResultUnsupportedOperation::make())
    }
}

impl<const M: HandleMode> CommandParameter<sf::Handle<M>> for sf::Handle<M> {
    fn before_request_write(handle: &Self, _walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.in_params.add_handle(handle.clone())
    }

    fn before_send_sync_request(_handle: &Self, _walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        Ok(())
    }

    fn after_response_read(_walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<Self> {
        ctx.out_params.pop_handle()
    }
}

impl CommandParameter<sf::ProcessId> for sf::ProcessId {
    fn before_request_write(_process_id: &Self, walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.in_params.send_process_id = true;
        if ctx.object_info.uses_cmif_protocol() {
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

    fn after_response_read(_walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<Self> {
        // TODO: is this actually valid/used?
        Err(results::hipc::ResultUnsupportedOperation::make())
    }
}

impl CommandParameter<mem::Shared<dyn sf::IObject>> for mem::Shared<dyn sf::IObject> {
    fn before_request_write(session: &Self, _walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.in_params.add_object(session.get().get_info())
    }

    fn before_send_sync_request(_session: &Self, _walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        Ok(())
    }

    fn after_response_read(_walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<Self> {
        // Only supported when the IObject type is known (see the generic implementation below)
        Err(results::hipc::ResultUnsupportedOperation::make())
    }
}

impl<S:service::IClientObject + 'static> CommandParameter<mem::Shared<dyn sf::IObject>> for mem::Shared<S> {
    fn before_request_write(session: &Self, _walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<()> {
        ctx.in_params.add_object(session.get().get_info())
    }

    fn before_send_sync_request(_session: &Self, _walker: &mut DataWalker, _ctx: &mut CommandContext) -> Result<()> {
        Ok(())
    }

    fn after_response_read(_walker: &mut DataWalker, ctx: &mut CommandContext) -> Result<mem::Shared<dyn sf::IObject>> {
        let object_info = ctx.pop_object()?;
        Ok(mem::Shared::new(S::new(sf::Session::from(object_info))))
    }
}