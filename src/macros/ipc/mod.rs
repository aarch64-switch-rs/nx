#![macro_use]

pub mod client;

#[macro_export]
macro_rules! ipc_interface_define_command {
    ($name:ident: ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )) => {
        #[allow(unused_parens)]
        fn $name(&mut self, $( $in_param_name: $in_param_type ),* ) -> $crate::result::Result<( $( $out_param_type ),* )>;

        paste::paste! {
            #[allow(unused_assignments)]
            #[allow(unused_parens)]
            fn [<$name _impl>](&mut self, mut ctx: &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()> {
                ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.in_params.data_offset);
                $( let $in_param_name = <$in_param_type as $crate::ipc::server::CommandParameter<_>>::after_request_read(&mut ctx)?; )*

                let ( $( $out_param_name ),* ) = self.$name( $( $in_param_name ),* )?;

                ctx.raw_data_walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
                $( $crate::ipc::server::CommandParameter::<_>::before_response_write(&$out_param_name, &mut ctx)?; )*
                ctx.ctx.out_params.data_size = ctx.raw_data_walker.get_offset() as u32;

                $crate::ipc::server::write_request_command_response_on_ipc_buffer(&mut ctx.ctx, $crate::result::ResultSuccess::make(), $crate::ipc::CommandType::Request);

                ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.out_params.data_offset);
                $( $crate::ipc::server::CommandParameter::<_>::after_response_write(&$out_param_name, &mut ctx)?; )*

                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! ipc_control_interface_define_command {
    ($name:ident: ( $( $in_param_name:ident: $in_param_type:ty ),* ) => ( $( $out_param_name:ident: $out_param_type:ty ),* )) => {
        #[allow(unused_parens)]
        fn $name(&mut self, $( $in_param_name: $in_param_type ),* ) -> $crate::result::Result<( $( $out_param_type ),* )>;

        paste::paste! {
            #[allow(unused_assignments)]
            #[allow(unused_parens)]
            fn [<$name _impl>](&mut self, mut ctx: &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()> {
                ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.in_params.data_offset);
                $( let $in_param_name = <$in_param_type as $crate::ipc::server::CommandParameter<_>>::after_request_read(&mut ctx)?; )*

                let ( $( $out_param_name ),* ) = self.$name( $( $in_param_name ),* )?;

                ctx.raw_data_walker = $crate::ipc::DataWalker::new(core::ptr::null_mut());
                $( $crate::ipc::server::CommandParameter::<_>::before_response_write(&$out_param_name, &mut ctx)?; )*
                ctx.ctx.out_params.data_size = ctx.raw_data_walker.get_offset() as u32;

                $crate::ipc::server::write_control_command_response_on_ipc_buffer(&mut ctx.ctx, $crate::result::ResultSuccess::make(), $crate::ipc::CommandType::Control);

                ctx.raw_data_walker = $crate::ipc::DataWalker::new(ctx.ctx.out_params.data_offset);
                $( $crate::ipc::server::CommandParameter::<_>::after_response_write(&$out_param_name, &mut ctx)?; )*

                Ok(())
            }
        }
    };
}

#[macro_export]
macro_rules! ipc_interface_make_command_meta {
    ($name:ident: $id:expr) => {
        paste::paste! {
            $crate::ipc::sf::CommandMetadata::new($id, unsafe { core::mem::transmute(Self::[<$name _impl>] as fn(&mut Self, &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()>) }, None, None)
        }
    };
    ($name:ident: $id:expr, [($major:expr, $minor:expr, $micro:expr) =>]) => {
        paste::paste! {
            $crate::ipc::sf::CommandMetadata::new($id, unsafe { core::mem::transmute(Self::[<$name _impl>] as fn(&mut Self, &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()>) }, Some($crate::version::Version::new($major, $minor, $micro)), None)
        }
    };
    ($name:ident: $id:expr, [=> ($major:expr, $minor:expr, $micro:expr)]) => {
        paste::paste! {
            $crate::ipc::sf::CommandMetadata::new($id, unsafe { core::mem::transmute(Self::[<$name _impl>] as fn(&mut Self, &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()>) }, None, Some($crate::version::Version::new($major, $minor, $micro)))
        }
    };
    ($name:ident: $id:expr, [($major_a:expr, $minor_a:expr, $micro_a:expr) => ($major_b:expr, $minor_b:expr, $micro_b:expr)]) => {
        paste::paste! {
            $crate::ipc::sf::CommandMetadata::new($id, unsafe { core::mem::transmute(Self::[<$name _impl>] as fn(&mut Self, &mut $crate::ipc::server::ServerContext) -> $crate::result::Result<()>) }, Some($crate::version::Version::new($major_a, $minor_a, $micro_a)), Some($crate::version::Version::new($major_b, $minor_b, $micro_b)))
        }
    };
}