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
                    $crate::ipc::client::CommandParameter::<_>::before_request_write(in_v, &mut walker, &mut ctx)?;
                }
            )*
            ctx.in_params.data_size = walker.get_offset() as u32;
            
            $crate::ipc::client::write_request_command_on_ipc_buffer(&mut ctx, Some($rq_id), $crate::ipc::DomainCommandType::SendMessage);

            walker.reset_with(ctx.in_params.data_offset);
            $(
                {
                    let in_v = &$in_param;
                    $crate::ipc::client::CommandParameter::<_>::before_send_sync_request(in_v, &mut walker, &mut ctx)?;
                }
            )*

            $crate::svc::send_sync_request($session.handle)?;

            $crate::ipc::client::read_request_command_response_from_ipc_buffer(&mut ctx)?;

            walker.reset_with(ctx.out_params.data_offset);
            $( let $out_param = <$out_param_type as $crate::ipc::client::CommandParameter<_>>::after_response_read(&mut walker, &mut ctx)?; )*

            Ok(( $( $out_param ),* ))
        };
        rc
    }};
}

#[macro_export]
macro_rules! ipc_client_send_control_command {
    ([$session:expr; $control_rq_id:expr] ( $( $in_param:expr ),* ) => ( $( $out_param:ident: $out_param_type:ty ),* )) => {{
        let rc: $crate::result::Result<_> = {
            let mut ctx = $crate::ipc::CommandContext::new_client($session);

            let mut walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
            $(
                {
                    let in_v = &$in_param;
                    $crate::ipc::client::CommandParameter::<_>::before_request_write(in_v, &mut walker, &mut ctx)?;
                }
            )*
            ctx.in_params.data_size = walker.get_offset() as u32;
            
            $crate::ipc::client::write_control_command_on_ipc_buffer(&mut ctx, $control_rq_id);

            walker.reset_with(ctx.in_params.data_offset);
            $(
                {
                    let in_v = &$in_param;
                    $crate::ipc::client::CommandParameter::<_>::before_send_sync_request(in_v, &mut walker, &mut ctx)?;
                }
            )*

            $crate::svc::send_sync_request($session.handle)?;

            $crate::ipc::client::read_control_command_response_from_ipc_buffer(&mut ctx)?;

            walker.reset_with(ctx.out_params.data_offset);
            $( let $out_param = <$out_param_type as $crate::ipc::client::CommandParameter<_>>::after_response_read(&mut walker, &mut ctx)?; )*

            Ok(( $( $out_param ),* ))
        };
        rc
    }};
}