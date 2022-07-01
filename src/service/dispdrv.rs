use crate::result::*;
use crate::ipc::sf::{self, sm};
use crate::service;

pub use crate::ipc::sf::dispdrv::*;

ipc_client_define_object_default!(HOSBinderDriver);

impl IHOSBinderDriver for HOSBinderDriver {
    fn transact_parcel(&mut self, binder_handle: BinderHandle, transaction_id: ParcelTransactionId, flags: u32, in_parcel: sf::InMapAliasBuffer<u8>, out_parcel: sf::OutMapAliasBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 0] (binder_handle, transaction_id, flags, in_parcel, out_parcel) => ())
    }

    fn adjust_refcount(&mut self, binder_handle: BinderHandle, add_value: i32, refcount_type: RefcountType) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 1] (binder_handle, add_value, refcount_type) => ())
    }

    fn get_native_handle(&mut self, binder_handle: BinderHandle, handle_type: NativeHandleType) -> Result<sf::CopyHandle> {
        ipc_client_send_request_command!([self.session.object_info; 2] (binder_handle, handle_type) => (native_handle: sf::CopyHandle))
    }

    fn transact_parcel_auto(&mut self, binder_handle: BinderHandle, transaction_id: ParcelTransactionId, flags: u32, in_parcel: sf::InAutoSelectBuffer<u8>, out_parcel: sf::OutAutoSelectBuffer<u8>) -> Result<()> {
        ipc_client_send_request_command!([self.session.object_info; 3] (binder_handle, transaction_id, flags, in_parcel, out_parcel) => ())
    }
}

impl service::IService for HOSBinderDriver {
    fn get_name() -> sm::ServiceName {
        sm::ServiceName::new("dispdrv")
    }

    fn as_domain() -> bool {
        false
    }

    fn post_initialize(&mut self) -> Result<()> {
        Ok(())
    }
}