use crate::result::*;
use crate::ipc::sf;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum RefcountType {
    Weak,
    Strong,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum ParcelTransactionId {
    RequestBuffer = 1,
    SetBufferCount = 2,
    DequeueBuffer = 3,
    DetachBuffer = 4,
    DetachNextBuffer = 5,
    AttachBuffer = 6,
    QueueBuffer = 7,
    CancelBuffer = 8,
    Query = 9,
    Connect = 10,
    Disconnect = 11,
    SetSidebandStream = 12,
    AllocateBuffers = 13,
    SetPreallocatedBuffer = 14,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum NativeHandleType {
    BufferEvent = 0xF
}

pub type BinderHandle = i32;

pub trait IHOSBinderDriver {
    ipc_interface_define_command!(transact_parcel: (binder_handle: BinderHandle, transaction_id: ParcelTransactionId, flags: u32, in_parcel: sf::InMapAliasBuffer, out_parcel: sf::OutMapAliasBuffer) => ());
    ipc_interface_define_command!(adjust_refcount: (binder_handle: BinderHandle, add_value: i32, refcount_type: RefcountType) => ());
    ipc_interface_define_command!(get_native_handle: (binder_handle: BinderHandle, handle_type: NativeHandleType) => (native_handle: sf::CopyHandle));
    ipc_interface_define_command!(transact_parcel_auto: (binder_handle: BinderHandle, transaction_id: ParcelTransactionId, flags: u32, in_parcel: sf::InAutoSelectBuffer, out_parcel: sf::OutAutoSelectBuffer) => ());
}