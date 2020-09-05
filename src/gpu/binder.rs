use crate::result::*;
use crate::results;
use crate::ipc::sf;
use crate::gpu::parcel;
use crate::service::dispdrv;
use crate::service::dispdrv::IHOSBinderDriver;
use crate::mem;
use core::mem as cmem;
use super::*;

pub const INTERFACE_TOKEN: &str = "android.gui.IGraphicBufferProducer";

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(i32)]
pub enum ErrorCode {
    Success = 0,
    PermissionDenied = -1,
    NameNotFound = -2,
    WouldBlock = -11,
    NoMemory = -12,
    AlreadyExists = -17,
    NoInit = -19,
    BadValue = -22,
    DeadObject = -32,
    InvalidOperation = -38,
    NotEnoughData = -61,
    UnknownTransaction = -74,
    BadIndex = -75,
    TimeOut = -110,
    FdsNotAllowed = -2147483641,
    FailedTransaction = -2147483646,
    BadType = -2147483647,
}

#[allow(unreachable_patterns)]
pub fn convert_error_code(err: ErrorCode) -> Result<()> {
    match err {
        ErrorCode::Success => Ok(()),
        ErrorCode::PermissionDenied => Err(results::lib::gpu::ResultBinderErrorCodePermissionDenied::make()),
        ErrorCode::NameNotFound => Err(results::lib::gpu::ResultBinderErrorCodeNameNotFound::make()),
        ErrorCode::WouldBlock => Err(results::lib::gpu::ResultBinderErrorCodeWouldBlock::make()),
        ErrorCode::NoMemory => Err(results::lib::gpu::ResultBinderErrorCodeNoMemory::make()),
        ErrorCode::AlreadyExists => Err(results::lib::gpu::ResultBinderErrorCodeAlreadyExists::make()),
        ErrorCode::NoInit => Err(results::lib::gpu::ResultBinderErrorCodeNoInit::make()),
        ErrorCode::BadValue => Err(results::lib::gpu::ResultBinderErrorCodeBadValue::make()),
        ErrorCode::DeadObject => Err(results::lib::gpu::ResultBinderErrorCodeDeadObject::make()),
        ErrorCode::InvalidOperation => Err(results::lib::gpu::ResultBinderErrorCodeInvalidOperation::make()),
        ErrorCode::NotEnoughData => Err(results::lib::gpu::ResultBinderErrorCodeNotEnoughData::make()),
        ErrorCode::UnknownTransaction => Err(results::lib::gpu::ResultBinderErrorCodeUnknownTransaction::make()),
        ErrorCode::BadIndex => Err(results::lib::gpu::ResultBinderErrorCodeBadIndex::make()),
        ErrorCode::TimeOut => Err(results::lib::gpu::ResultBinderErrorCodeTimeOut::make()),
        ErrorCode::FdsNotAllowed => Err(results::lib::gpu::ResultBinderErrorCodeFdsNotAllowed::make()),
        ErrorCode::FailedTransaction => Err(results::lib::gpu::ResultBinderErrorCodeFailedTransaction::make()),
        ErrorCode::BadType => Err(results::lib::gpu::ResultBinderErrorCodeBadType::make()),
        _ => Err(results::lib::gpu::ResultBinderErrorCodeInvalid::make()),
    }
}

pub struct Binder {
    handle: dispdrv::BinderHandle,
    hos_binder_driver: mem::Shared<dispdrv::HOSBinderDriver>,
}

impl Binder {
    pub fn new(handle: dispdrv::BinderHandle, hos_binder_driver: mem::Shared<dispdrv::HOSBinderDriver>) -> Result<Self> {
        Ok(Self { handle: handle, hos_binder_driver: hos_binder_driver })
    }

    fn transact_parcel_begin(&self, parcel: &mut parcel::Parcel) -> Result<()> {
        parcel.write_interface_token(INTERFACE_TOKEN)
    }

    fn transact_parcel_check_err(&mut self, parcel: &mut parcel::Parcel) -> Result<()> {
        let err: ErrorCode = parcel.read()?;
        convert_error_code(err)?;
        Ok(())
    }

    fn transact_parcel_impl(&mut self, transaction_id: dispdrv::ParcelTransactionId, payload: parcel::ParcelPayload) -> Result<parcel::Parcel> {
        let response_payload = parcel::ParcelPayload::new();
        self.hos_binder_driver.get().transact_parcel(self.handle, transaction_id, 0, sf::Buffer::from_var(&payload), sf::Buffer::from_var(&response_payload))?;
        
        let mut parcel = parcel::Parcel::new();
        parcel.load_from(response_payload);
        Ok(parcel)
    }

    fn transact_parcel(&mut self, transaction_id: dispdrv::ParcelTransactionId, parcel: &mut parcel::Parcel) -> Result<parcel::Parcel> {
        let (payload, _payload_size) = parcel.end_write()?;
        self.transact_parcel_impl(transaction_id, payload)
    }

    pub fn get_handle(&self) -> i32 {
        self.handle
    }

    pub fn get_hos_binder_driver(&mut self) -> mem::Shared<dispdrv::HOSBinderDriver> {
        self.hos_binder_driver.clone()
    }

    pub fn increase_refcounts(&mut self) -> Result<()> {
        self.hos_binder_driver.get().adjust_refcount(self.handle, 1, dispdrv::RefcountType::Weak)?;
        self.hos_binder_driver.get().adjust_refcount(self.handle, 1, dispdrv::RefcountType::Strong)
    }

    pub fn decrease_refcounts(&mut self) -> Result<()> {
        self.hos_binder_driver.get().adjust_refcount(self.handle, -1, dispdrv::RefcountType::Weak)?;
        self.hos_binder_driver.get().adjust_refcount(self.handle, -1, dispdrv::RefcountType::Strong)
    }

    pub fn connect(&mut self, api: ConnectionApi, producer_controlled_by_app: bool) -> Result<QueueBufferOutput> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        let producer_listener: u32 = 0;
        parcel.write(producer_listener)?;
        parcel.write(api)?;
        parcel.write(producer_controlled_by_app as u32)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::Connect, &mut parcel)?;
        let qbo: QueueBufferOutput = response_parcel.read()?;

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok(qbo)
    }

    pub fn disconnect(&mut self, api: ConnectionApi, mode: DisconnectMode) -> Result<()> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(api)?;
        parcel.write(mode)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::Disconnect, &mut parcel)?;

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok(())
    }

    pub fn set_preallocated_buffer(&mut self, slot: i32, buf: GraphicBuffer) -> Result<()> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(slot)?;
        let has_input = true;
        parcel.write(has_input as u32)?;
        if has_input {
            parcel.write_sized(buf)?;
        }

        self.transact_parcel(dispdrv::ParcelTransactionId::SetPreallocatedBuffer, &mut parcel)?;
        Ok(())
    }
    
    pub fn request_buffer(&mut self, slot: i32) -> Result<(bool, GraphicBuffer)> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(slot)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::RequestBuffer, &mut parcel)?;
        let non_null_v: u32 = response_parcel.read()?;
        let non_null = non_null_v != 0;
        let mut gfx_buf: GraphicBuffer = unsafe { cmem::zeroed() };
        if non_null {
            gfx_buf = response_parcel.read_sized()?;
        }

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok((non_null, gfx_buf))
    }

    pub fn dequeue_buffer(&mut self, is_async: bool, width: u32, height: u32, get_frame_timestamps: bool, usage: GraphicsAllocatorUsage) -> Result<(i32, bool, MultiFence)> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(is_async as u32)?;
        parcel.write(width)?;
        parcel.write(height)?;
        parcel.write(get_frame_timestamps as u32)?;
        parcel.write(usage)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::DequeueBuffer, &mut parcel)?;

        let slot: i32 = response_parcel.read()?;
        let has_fences_v: u32 = response_parcel.read()?;
        let has_fences = has_fences_v != 0;
        let mut fences: MultiFence = unsafe { cmem::zeroed() };
        if has_fences {
            fences = response_parcel.read_sized()?;
        }

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok((slot, has_fences, fences))
    }

    pub fn queue_buffer(&mut self, slot: i32, qbi: QueueBufferInput) -> Result<QueueBufferOutput> {
        let mut parcel = parcel::Parcel::new();
        self.transact_parcel_begin(&mut parcel)?;

        parcel.write(slot)?;
        parcel.write_sized(qbi)?;

        let mut response_parcel = self.transact_parcel(dispdrv::ParcelTransactionId::QueueBuffer, &mut parcel)?;

        let qbo = response_parcel.read()?;

        self.transact_parcel_check_err(&mut response_parcel)?;
        Ok(qbo)
    }

    pub fn get_native_handle(&mut self, handle_type: dispdrv::NativeHandleType) -> Result<sf::CopyHandle> {
        self.hos_binder_driver.get().get_native_handle(self.handle, handle_type)
    }
}