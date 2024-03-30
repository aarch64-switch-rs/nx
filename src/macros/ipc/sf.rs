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
///     trait IExampleInterface {
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
                $name:ident [$rq_id:expr, $ver_intv:expr]: ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )
            );* $(;)* // Note: trick to allow last trailing ';' for proper styling
        }
    ) => {
        paste::paste! {
            pub trait $intf: $crate::ipc::sf::IObject {
                $(
                    #[allow(unused_parens)]
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

                fn get_sf_command_metadata_table(&self) -> $crate::ipc::sf::CommandMetadataTable {
                    vec! [
                        $(
                            $crate::ipc::sf::CommandMetadata::new($rq_id, unsafe { core::mem::transmute(Self::[<sf_server_impl_ $name>] as fn(&mut Self, $crate::ipc::CommandProtocol, &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()>) }, $ver_intv)
                        ),*
                    ]
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

                fn get_sf_command_metadata_table(&self) -> $crate::ipc::sf::CommandMetadataTable {
                    vec! [
                        $(
                            $crate::ipc::sf::CommandMetadata::new($rq_id, unsafe { core::mem::transmute(Self::[<sf_server_impl_ $name>] as fn(&mut Self, $crate::ipc::CommandProtocol, &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()>) }, $ver_intv)
                        ),*
                    ]
                }
            }
        }
    };
}

/// Macro to simplify defining an IPC interface impl command metadata
/// 
/// This is meant to only be used inside [`IObject`][`crate::ipc::sf::IObject`] impls!
/// 
/// Note that this only has to be manually used for non-client-IPC interface types - for client-IPC interface types, see [`ipc_client_define_object_default`]
/// 
/// # Example
/// ```
/// use nx::ipc::sf::{Session, IObject};
/// 
/// // Let's assume an IPC interface named "IExampleInterface" exists
/// // Let's then create a custom implementation of that interface
/// pub struct ExampleInterface {
///     // Required, only effectively used on client IPC interfaces
///     dummy_session: Session
/// }
/// 
/// impl IObject for ExampleInterface {
///     ipc_sf_object_impl_default_command_metadata!();
/// 
///     fn get_session(&mut self) -> &mut Session {
///         &mut self.dummy_session
///     }
/// }
/// 
/// impl IExampleInterface for ExampleInterface {
///     (...)
/// }
/// ```
#[macro_export]
macro_rules! ipc_sf_object_impl_default_command_metadata {
    () => {
        fn get_command_metadata_table(&self) -> $crate::ipc::sf::CommandMetadataTable {
            // Provided by the interface being implemented by this object
            self.get_sf_command_metadata_table()
        }
    };
}

// TODO: better system than using ipc_sf_object_impl_default_command_metadata!(), enforce command version when invoking it (only on client implementations, etc.), more