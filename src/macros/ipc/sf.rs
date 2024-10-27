#![macro_use]

/// Defines a trait meant to be used for IPC interfaces
/// 
/// # Examples
/// 
/// ```
/// use nx::version::{Version, VersionInterval};
/// 
/// // Define commands with their request ID, allowed version interval and in/out parameters
/// ipc_sf_define_interface_trait! {
///     trait ExampleInterface {
///         command_1 [1, VersionInterval::all()]: (in_32: u32) => (out_16: u16);
///         command_2 [20, VersionInterval::all()]: (in_8: u8) => ();
///     }
/// }
/// 
/// // You can impl "IExampleInterface" to create a custom object implementing the commands
/// ```
#[macro_export]
macro_rules! ipc_sf_define_interface_trait {
    (
        trait $intf:ident {
            $(
                $name:ident [$rq_id:expr, $ver_intv:expr $(, $noalias:tt)?]: ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )
            );* $(;)* // Note: trick to allow last trailing ';' for proper styling
        }
    ) => {
        paste::paste! {
            pub trait [<I$intf>]: $crate::ipc::sf::IObject {
                $(
                    #[allow(unused_parens)]
                    #[allow(clippy::too_many_arguments)]
                    fn $name(& $($noalias)? self, $( $in_param_name: $in_param_type ),* ) -> $crate::result::Result<( $( $out_param_type ),* )> {
                        let mut ctx = $crate::ipc::CommandContext::new_client(self.get_session().object_info);

                        let mut walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
                        $( $crate::ipc::client::RequestCommandParameter::before_request_write(&$in_param_name, &mut walker, &mut ctx)?; )*
                        ctx.in_params.data_size = walker.get_offset() as u32;
                        
                        match self.get_session().object_info.protocol {
                            $crate::ipc::CommandProtocol::Cmif => $crate::ipc::cmif::client::write_request_command_on_msg_buffer(&mut ctx, Some($rq_id), $crate::ipc::cmif::DomainCommandType::SendMessage),
                            $crate::ipc::CommandProtocol::Tipc => $crate::ipc::tipc::client::write_request_command_on_msg_buffer(&mut ctx, $rq_id)
                        };

                        walker.reset_with(ctx.in_params.data_offset);
                        $( $crate::ipc::client::RequestCommandParameter::before_send_sync_request(&$in_param_name, &mut walker, &mut ctx)?; )*

                        $crate::svc::send_sync_request(self.get_session().object_info.handle)?;

                        match self.get_session().object_info.protocol {
                            $crate::ipc::CommandProtocol::Cmif => $crate::ipc::cmif::client::read_request_command_response_from_msg_buffer(&mut ctx)?,
                            $crate::ipc::CommandProtocol::Tipc => $crate::ipc::tipc::client::read_request_command_response_from_msg_buffer(&mut ctx)?
                        };

                        walker.reset_with(ctx.out_params.data_offset);
                        $( let $out_param_name = <$out_param_type as $crate::ipc::client::ResponseCommandParameter<_>>::after_response_read(&mut walker, &mut ctx)?; )*

                        Ok(( $( $out_param_name as _ ),* ))
                    }
                )*
            }

            pub trait [<I $intf Server>]: $crate::ipc::server::ISessionObject {
                $(
                    #[allow(unused_parens)]
                    #[allow(clippy::too_many_arguments)]
                    fn $name(&mut self, $( $in_param_name: $in_param_type ),* ) -> $crate::result::Result<( $( $out_param_type ),* )>;
        
                    #[allow(unused_assignments)]
                    #[allow(unused_parens)]
                    #[allow(unused_mut)]
                    fn [<sf_server_impl_ $name>](&mut self, protocol: $crate::ipc::CommandProtocol, mut ctx: &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()> {
                        ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.in_params.data_offset);
                        $( let $in_param_name = <$in_param_type as $crate::ipc::server::RequestCommandParameter<_>>::after_request_read(&mut ctx)?; )*
        
                        let ( $( $out_param_name ),* ) = self.$name( $( $in_param_name ),* )?;
        
                        ctx.raw_data_walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
                        $( $crate::ipc::server::ResponseCommandParameter::before_response_write(&$out_param_name, &mut ctx)?; )*
                        ctx.ctx.out_params.data_size = ctx.raw_data_walker.get_offset() as u32;
        
                        match protocol {
                            $crate::ipc::CommandProtocol::Cmif => {
                                $crate::ipc::cmif::server::write_request_command_response_on_msg_buffer(&mut ctx.ctx, $crate::result::ResultSuccess::make(), $crate::ipc::cmif::CommandType::Request);
                            },
                            $crate::ipc::CommandProtocol::Tipc => {
                                $crate::ipc::tipc::server::write_request_command_response_on_msg_buffer(&mut ctx.ctx, $crate::result::ResultSuccess::make(), 16); // TODO: is this command type actually read/used/relevant?
                            }
                        };
        
                        ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.out_params.data_offset);
                        $( $crate::ipc::server::ResponseCommandParameter::after_response_write(&$out_param_name, &mut ctx)?; )*
        
                        Ok(())
                    }
                )*

                fn try_handle_request_by_id(&mut self, req_id: u32, protocol: $crate::ipc::CommandProtocol, ctx: &mut $crate::ipc::server::ServerContext) -> Option<$crate::result::Result<()>> {
                    match req_id {
                        $(
                            $rq_id if $ver_intv.contains($crate::version::get_version()) => {
                                Some(self.[<sf_server_impl_ $name>](protocol, ctx))
                            }
                        ),*
                        _ => None
                    }
                }
            }
        }
    };
}

/// Identical to [`ipc_sf_define_interface_trait`] but for "Control" IPC interfaces (inner trait functionality differs)
/// 
/// This shouldn't really be used unless you really know what you're doing
#[macro_export]
macro_rules! ipc_sf_define_control_interface_trait {
    (
        trait $intf:ident {
            $(
                $name:ident [$rq_id:expr, $ver_intv:expr]: ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )
            );* $(;)* // Same as above
        }
    ) => {
        paste::paste! {
            pub trait $intf: $crate::ipc::sf::IObject {
                $(
                    #[allow(unused_parens)]
                    fn $name(&mut self, $( $in_param_name: $in_param_type ),* ) -> $crate::result::Result<( $( $out_param_type ),* )>;
        
                    #[allow(unused_assignments)]
                    #[allow(unused_parens)]
                    fn [<sf_server_impl_ $name>](&mut self, _protocol: $crate::ipc::CommandProtocol, mut ctx: &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()> {
                        // TODO: tipc support, for now force cmif
                        $crate::result_return_if!(ctx.ctx.object_info.uses_tipc_protocol(), $crate::ipc::rc::ResultInvalidProtocol);

                        ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.in_params.data_offset);
                        $( let $in_param_name = <$in_param_type as $crate::ipc::server::RequestCommandParameter<_>>::after_request_read(&mut ctx)?; )*

                        let ( $( $out_param_name ),* ) = self.$name( $( $in_param_name ),* )?;

                        ctx.raw_data_walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
                        $( $crate::ipc::server::ResponseCommandParameter::before_response_write(&$out_param_name, &mut ctx)?; )*
                        ctx.ctx.out_params.data_size = ctx.raw_data_walker.get_offset() as u32;

                        $crate::ipc::cmif::server::write_control_command_response_on_msg_buffer(&mut ctx.ctx, $crate::result::ResultSuccess::make(), $crate::ipc::cmif::CommandType::Control);

                        ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.out_params.data_offset);
                        $( $crate::ipc::server::ResponseCommandParameter::after_response_write(&$out_param_name, &mut ctx)?; )*

                        Ok(())
                    }
                )*

                fn try_handle_request_by_id(&mut self, rq_id: u32, protocol: $crate::ipc::CommandProtocol, ctx: &mut $crate::ipc::server::ServerContext) -> Option<$crate::result::Result<()>> {
                    match rq_id {
                        $(
                            $rq_id if $ver_intv.contains($crate::version::get_version()) => {
                                Some(self.[<sf_server_impl_ $name>](protocol, ctx))
                            }
                        ),*
                        _ => None
                    }
                }
            }
        }
    };
}

// TODO: better system than using ipc_sf_object_impl_default_command_metadata!(), enforce command version when invoking it (only on client implementations, etc.), more
/*
#[macro_export]
macro_rules! server_mark_request_command_parameters_types_as_copy {
    ($($t:ty),*) => {
        $(
        //const_assert!($t::is_pod());
        impl $crate::ipc::server::RequestCommandParameter<$t> for $t {
            default fn after_request_read(ctx: &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<Self> {
                Ok(ctx.raw_data_walker.advance_get())
            }
        }
        
        impl $crate::ipc::server::ResponseCommandParameter for $t {
            default fn before_response_write(_raw: &Self, ctx: &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()> {
                ctx.raw_data_walker.advance::<Self>();
                Ok(())
            }
        
            default fn after_response_write(raw: &Self, ctx: &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()> {
                ctx.raw_data_walker.advance_set(*raw);
                Ok(())
            }
        }
        )*
    };
}

#[macro_export]
macro_rules! api_mark_request_command_parameters_types_as_copy {
    ($($t:ty),*) => {
        $(
        server_mark_request_command_parameters_types_as_copy!($t);
        client_mark_request_command_parameters_types_as_copy!($t);
        )*
    };
}*/
/*
#[macro_export]
macro_rules! ipc_impl_dyn_trait_as_server_param {
    ($t:trait) => {
        impl $crate::ipc::server::RequestCommandParameter for $crate::mem::Shared<dyn $t> {
            default fn after_request_read(_ctx: &mut ServerContext) -> Result<S> {
                // TODO: implement this (added this placeholder impl for interfaces to actually be valid)
                sf::hipc::rc::ResultUnsupportedOperation::make_err()
            }
        }
        
        impl<S: sf::IObject + ?Sized> RequestCommandParameter<mem::Shared<dyn $t>> for mem::Shared<dyn $t> {
            fn after_request_read(_ctx: &mut ServerContext) -> Result<Self> {
                // TODO: implement this (added this placeholder impl for interfaces to actually be valid)
                sf::hipc::rc::ResultUnsupportedOperation::make_err()
            }
        }

        impl ResponseCommandParameter for mem::Shared<dyn ISessionObject> {
            fn before_response_write(session: &Self, ctx: &mut ServerContext) -> Result<()> {
                let session_copy: mem::Shared<dyn ISessionObject> = session.clone();
        
                //let session_copy: mem::Shared<dyn ISessionObject> = unsafe {core::mem::transmute::<mem::Shared<S>, mem::Shared<dyn ISe>>(session_copy)};
                if ctx.ctx.object_info.is_domain() {
                    let domain_table = ctx.domain_table.clone().ok_or(rc::ResultDomainNotFound::make())?;
                    let domain_object_id = domain_table.lock().allocate_id()?;
                    ctx.ctx.out_params.push_domain_object(domain_object_id)?;
                    domain_table.lock().domains.push(ServerHolder::new_domain_session(0, domain_object_id, session_copy));
                    Ok(())
                }
                else {
                    let (server_handle, client_handle) = svc::create_session(false, 0)?;
                    ctx.ctx.out_params.push_handle(sf::MoveHandle::from(client_handle))?;
                    ctx.new_sessions.push(ServerHolder::new_session(server_handle, session_copy));
                    Ok(())
                }
            }
        
            fn after_response_write(_session: &Self, _ctx: &mut ServerContext) -> Result<()> {
                Ok(())
            }
        }
    }
}
 */
/*api_mark_request_command_parameters_types_as_copy!(bool, u8, i8, u16, i16, u32, i32, u64, i64, usize, isize, u128, i128, f32, f64);

impl<T: Copy, const N: usize>  crate::ipc::server::RequestCommandParameter<[T;N]> for [T;N] {
    default fn after_request_read(ctx: &mut crate::ipc::server::ServerContext) -> crate::result::Result<Self> {
        Ok(ctx.raw_data_walker.advance_get())
    }
}

impl<T: Copy, const N: usize> crate::ipc::server::ResponseCommandParameter for [T;N] {
    default fn before_response_write(_raw: &Self, ctx: &mut crate::ipc::server::ServerContext) -> crate::result::Result<()> {
        ctx.raw_data_walker.advance::<Self>();
        Ok(())
    }

    default fn after_response_write(raw: &Self, ctx: &mut crate::ipc::server::ServerContext) -> crate::result::Result<()> {
        ctx.raw_data_walker.advance_set(*raw);
        Ok(())
    }
}*/