#![macro_use]

#[macro_export]
macro_rules! ipc_client_send_request_command {
    ([$session:expr; $rq_id:expr] ( $( $in_param:expr ),* ) => ( $( $out_param:ident: $out_param_type:ty ),* )) => {{
        let rc: $crate::result::Result<_> = {
            let mut ctx = $crate::ipc::CommandContext::new_client($session);

            let mut walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
            $(
                {
                    let in_v = &$in_param;
                    $crate::ipc::client::RequestCommandParameter::before_request_write(in_v, &mut walker, &mut ctx)?;
                }
            )*
            ctx.in_params.data_size = walker.get_offset() as u32;
            
            match $session.protocol {
                $crate::ipc::CommandProtocol::Cmif => $crate::ipc::cmif::client::write_request_command_on_msg_buffer(&mut ctx, Some($rq_id), $crate::ipc::cmif::DomainCommandType::SendMessage),
                $crate::ipc::CommandProtocol::Tipc => $crate::ipc::tipc::client::write_request_command_on_msg_buffer(&mut ctx, $rq_id)
            };

            walker.reset_with(ctx.in_params.data_offset);
            $(
                {
                    let in_v = &$in_param;
                    $crate::ipc::client::RequestCommandParameter::before_send_sync_request(in_v, &mut walker, &mut ctx)?;
                }
            )*

            $crate::svc::send_sync_request($session.handle)?;

            match $session.protocol {
                $crate::ipc::CommandProtocol::Cmif => $crate::ipc::cmif::client::read_request_command_response_from_msg_buffer(&mut ctx)?,
                $crate::ipc::CommandProtocol::Tipc => $crate::ipc::tipc::client::read_request_command_response_from_msg_buffer(&mut ctx)?
            };

            walker.reset_with(ctx.out_params.data_offset);
            $( let $out_param = <$out_param_type as $crate::ipc::client::ResponseCommandParameter<_>>::after_response_read(&mut walker, &mut ctx)?; )*

            Ok(( $( $out_param as _ ),* ))
        };
        rc
    }};
}

#[macro_export]
macro_rules! ipc_client_send_control_command {
    ([$session:expr; $control_rq_id:expr] ( $( $in_param:expr ),* ) => ( $( $out_param:ident: $out_param_type:ty ),* )) => {{
        let rc: $crate::result::Result<_> = {
            if $session.uses_tipc_protocol() {
                /* Err */
            }

            let mut ctx = $crate::ipc::CommandContext::new_client($session);

            let mut walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
            $(
                {
                    let in_v = &$in_param;
                    $crate::ipc::client::RequestCommandParameter::before_request_write(in_v, &mut walker, &mut ctx)?;
                }
            )*
            ctx.in_params.data_size = walker.get_offset() as u32;
            
            $crate::ipc::cmif::client::write_control_command_on_msg_buffer(&mut ctx, $control_rq_id);

            walker.reset_with(ctx.in_params.data_offset);
            $(
                {
                    let in_v = &$in_param;
                    $crate::ipc::client::RequestCommandParameter::before_send_sync_request(in_v, &mut walker, &mut ctx)?;
                }
            )*

            $crate::svc::send_sync_request($session.handle)?;

            $crate::ipc::cmif::client::read_control_command_response_from_msg_buffer(&mut ctx)?;

            walker.reset_with(ctx.out_params.data_offset);
            $( let $out_param = <$out_param_type as $crate::ipc::client::ResponseCommandParameter<_>>::after_response_read(&mut walker, &mut ctx)?; )*

            Ok(( $( $out_param ),* ))
        };
        rc
    }};
}