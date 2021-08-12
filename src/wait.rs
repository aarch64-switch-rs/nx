use crate::result::*;
use crate::results;
use crate::svc;
use crate::arm;

pub struct RemoteEvent {
    pub handle: svc::Handle
}

impl RemoteEvent {
    pub const fn empty() -> Self {
        Self { handle: svc::INVALID_HANDLE }
    }
    
    pub const fn new(handle: svc::Handle) -> Self {
        Self { handle: handle }
    }

    pub fn reset(&self) -> Result<()> {
        svc::reset_signal(self.handle)
    }

    pub fn wait(&self, timeout: i64) -> Result<()> {
        wait_handles(&[self.handle], timeout)?;
        self.reset()
    }
}

impl Drop for RemoteEvent {
    fn drop(&mut self) {
        let _ = svc::close_handle(self.handle);
    }
}

pub struct SystemEvent {
    pub server_handle: svc::Handle,
    pub client_handle: svc::Handle
}

impl SystemEvent {
    pub const fn empty() -> Self {
        Self { server_handle: 0, client_handle: 0 }
    }
    
    pub fn new() -> Result<Self> {
        let (server_handle, client_handle) = svc::create_event()?;
        Ok(Self { server_handle: server_handle, client_handle: client_handle })
    }

    pub fn signal(&self) -> Result<()> {
        svc::signal_event(self.server_handle)
    }
}

impl Drop for SystemEvent {
    fn drop(&mut self) {
        let _ = svc::close_handle(self.client_handle);
        let _ = svc::close_handle(self.server_handle);
    }
}

pub enum WaiterType {
    Handle,
    HandleWithClear
}

pub const MAX_OBJECT_COUNT: u32 = 0x40;

#[allow(dead_code)]
pub struct Waiter {
    handle: svc::Handle,
    wait_type: WaiterType
}

impl Waiter {
    pub const fn from(handle: svc::Handle, wait_type: WaiterType) -> Self {
        Self { handle: handle, wait_type: wait_type }
    }
    
    pub const fn from_handle(handle: svc::Handle) -> Self {
        Self::from(handle, WaiterType::Handle)
    }

    pub const fn from_handle_with_clear(handle: svc::Handle) -> Self {
        Self::from(handle, WaiterType::HandleWithClear)
    }
}

type WaitFn<W> = fn(&[W], i64) -> Result<usize>;

fn handles_wait_fn(handles: &[svc::Handle], timeout: i64) -> Result<usize> {
    Ok(svc::wait_synchronization(handles.as_ptr(), handles.len() as u32, timeout)? as usize)
}

fn waiters_wait_fn(_waiters: &[Waiter], _timeout: i64) -> Result<usize> {
    todo!();
}

fn wait_impl<W>(wait_objects: &[W], timeout: i64, wait_fn: WaitFn<W>) -> Result<usize> {
    let has_timeout = timeout != -1;
    let mut deadline: u64 = 0;
    if has_timeout {
        deadline = arm::get_system_tick() - arm::nanoseconds_to_ticks(timeout as u64);
    }

    loop {
        let this_timeout = match has_timeout {
            true => {
                let remaining = deadline - arm::get_system_tick();
                arm::ticks_to_nanoseconds(remaining) as i64
            },
            false => -1
        };
        match (wait_fn)(wait_objects, this_timeout) {
            Ok(index) => return Ok(index),
            Err(rc) => {
                if results::os::ResultTimeout::matches(rc) {
                    if has_timeout {
                        return Err(rc);
                    }
                }
                else if !results::os::ResultOperationCanceled::matches(rc) {
                    return Err(rc);
                }
            }
        }
    }
}

pub fn wait(waiters: &[Waiter], timeout: i64) -> Result<usize> {
    wait_impl(waiters, timeout, waiters_wait_fn)
}

pub fn wait_handles(handles: &[svc::Handle], timeout: i64) -> Result<usize> {
    wait_impl(handles, timeout, handles_wait_fn)
}