use crate::result::*;
use crate::ipc::sf;
use crate::service;

pub use crate::ipc::sf::dispdrv::*;

pub struct HOSBinderDriver {
    session: sf::Session
}

impl sf::IObject for HOSBinderDriver {
    fn get_session(&mut self) -> &mut sf::Session {
        &mut self.session
    }

    fn get_command_table(&self) -> sf::CommandMetadataTable {
        ipc_server_make_command_table! {
            transact_parcel: 0,
            adjust_refcount: 1,
            get_native_handle: 2,
            transact_parcel_auto: 3
        }
    }
}

impl service::IClientObject for HOSBinderDriver {
    fn new(session: sf::Session) -> Self {
        Self { session: session }
    }
}

impl IHOSBinderDriver for HOSBinderDriver {
    fn transact_parcel(&mut self, binder_handle: BinderHandle, transaction_id: ParcelTransactionId, flags: u32, in_parcel: sf::InMapAliasBuffer, out_parcel: sf::OutMapAliasBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (binder_handle, transaction_id, flags, in_parcel, out_parcel) => ())
    }

    fn adjust_refcount(&mut self, binder_handle: BinderHandle, add_value: i32, refcount_type: RefcountType) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (binder_handle, add_value, refcount_type) => ())
    }

    fn get_native_handle(&mut self, binder_handle: BinderHandle, handle_type: NativeHandleType) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 2] (binder_handle, handle_type) => (native_handle: sf::CopyHandle))
    }

    fn transact_parcel_auto(&mut self, binder_handle: BinderHandle, transaction_id: ParcelTransactionId, flags: u32, in_parcel: sf::InAutoSelectBuffer, out_parcel: sf::OutAutoSelectBuffer) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (binder_handle, transaction_id, flags, in_parcel, out_parcel) => ())
    }
}

impl service::IService for HOSBinderDriver {
    fn get_name() -> &'static str {
        nul!("dispdrv")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}