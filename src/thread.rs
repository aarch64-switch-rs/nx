extern crate alloc as core_alloc;
use core_alloc::boxed::Box;

use crate::result::*;
use crate::results;
use crate::svc;
use crate::mem::alloc;
use crate::wait;
use crate::util;
use core::ptr;

pub type ThreadName = util::CString<0x20>;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(u8)]
pub enum ThreadState {
    #[default]
    NotInitialized = 0,
    Initialized = 1,
    DestroyedBeforeStarted = 2,
    Started = 3,
    Terminated = 4
}

extern fn thread_entry_impl(thread_arg: *mut u8) -> ! {
    let thread_ref = thread_arg as *mut Thread;
    set_current_thread(thread_ref);

    unsafe {
        if let Some(entry) = (*thread_ref).entry.as_ref() {
            entry.call(());
        }
    }

    exit();
}

pub const PRIORITY_AUTO: i32 = -1;

// Note: our thread type attempts to mimic the official nn::os::ThreadType struct so that the thread name is accessible from TLS by, for instance, creport
// TODO: TLS slots

#[repr(C)]
pub struct Thread {
    pub self_ref: *mut Thread,
    pub state: ThreadState,
    pub owns_stack: bool,
    pub pad: [u8; 2],
    pub handle: svc::Handle,
    pub stack: *mut u8,
    pub stack_size: usize,
    pub entry: Option<Box<dyn Fn()>>, // Note: size of this field is 0x10 bytes, same thing as entry and entry_arg ptrs on the official ThreadType struct
    pub unused_tls_slots: [*mut u8; 0x20],
    pub reserved: [u8; 0x54],
    pub name_len: u32,
    pub name: ThreadName,
    pub name_addr: *mut u8,
    pub reserved_2: [u8; 0x20],
}

impl Thread {
    pub const fn empty() -> Self {
        Self {
            self_ref: ptr::null_mut(),
            state: ThreadState::NotInitialized,
            owns_stack: false,
            pad: [0; 2],
            handle: 0,
            stack: ptr::null_mut(),
            stack_size: 0,
            entry: None,
            unused_tls_slots: [ptr::null_mut(); 0x20],
            reserved: [0; 0x54],
            name_len: 0,
            name: ThreadName::new(),
            name_addr: ptr::null_mut(),
            reserved_2: [0; 0x20],
        }
    }

    fn new_impl(handle: svc::Handle, state: ThreadState, name: &str, stack: *mut u8, stack_size: usize, owns_stack: bool, entry: Option<Box<dyn Fn()>>) -> Result<Self> {
        let mut thread = Self {
            self_ref: ptr::null_mut(),
            state: state,
            owns_stack: owns_stack,
            pad: [0; 2],
            handle: handle,
            stack: stack,
            stack_size: stack_size,
            entry: entry,
            unused_tls_slots: [ptr::null_mut(); 0x20],
            reserved: [0; 0x54],
            name_len: 0,
            name: util::CString::new(),
            name_addr: ptr::null_mut(),
            reserved_2: [0; 0x20],
        };
        thread.self_ref = &mut thread;
        thread.name_addr = &mut thread.name as *mut ThreadName as *mut u8;
        thread.name.set_str(name)?;
        Ok(thread)
    }

    pub fn new_remote(handle: svc::Handle, name: &str, stack: *mut u8, stack_size: usize) -> Result<Self> {
        Self::new_impl(handle, ThreadState::Started, name, stack, stack_size, false, None)
    }
    
    pub fn new_with_stack<F: 'static + Fn()>(entry: F, name: &str, stack: *mut u8, stack_size: usize) -> Result<Self> {
        result_return_unless!(!stack.is_null(), results::lib::thread::ResultInvalidStack);
        // TODO: also check alignment

        Self::new_impl(svc::INVALID_HANDLE, ThreadState::NotInitialized, name, stack, stack_size, false, Some(Box::new(entry)))
    }
    
    pub fn new<F: 'static + Fn()>(entry: F, name: &str, stack_size: usize) -> Result<Self> {
        let stack = alloc::allocate(alloc::PAGE_ALIGNMENT, stack_size)?;

        Self::new_impl(svc::INVALID_HANDLE, ThreadState::NotInitialized, name, stack, stack_size, true, Some(Box::new(entry)))
    }

    pub fn initialize(&mut self, priority: i32, processor_id: i32) -> Result<()> {
        result_return_unless!(self.state == ThreadState::NotInitialized, results::lib::thread::ResultInvalidState);

        let mut priority_value = priority;
        if priority_value == PRIORITY_AUTO {
            priority_value = get_current_thread().get_priority()?;
        }

        self.handle = svc::create_thread(thread_entry_impl, self as *mut _ as *mut u8, (self.stack as usize + self.stack_size) as *const u8, priority_value, processor_id)?;
        
        self.state = ThreadState::Initialized;
        Ok(())
    }

    pub fn start(&mut self) -> Result<()> {
        result_return_unless!((self.state == ThreadState::Initialized) || (self.state == ThreadState::Terminated), results::lib::thread::ResultInvalidState);

        svc::start_thread(self.handle)?;

        self.state = ThreadState::Started;
        Ok(())
    }

    pub fn join(&mut self) -> Result<()> {
        result_return_unless!(self.state == ThreadState::Started, results::lib::thread::ResultInvalidState);
        
        wait::wait_handles(&[self.handle], -1)?;

        self.state = ThreadState::Terminated;
        Ok(())
    }

    pub fn is_remote(&self) -> bool {
        self.entry.is_none()
    }

    pub fn get_handle(&self) -> svc::Handle {
        self.handle
    }

    pub fn get_priority(&self) -> Result<i32> {
        result_return_unless!(self.state != ThreadState::NotInitialized, results::lib::thread::ResultInvalidState);

        svc::get_thread_priority(self.handle)
    }

    pub fn get_id(&self) -> Result<u64> {
        result_return_unless!(self.state != ThreadState::NotInitialized, results::lib::thread::ResultInvalidState);
        
        svc::get_thread_id(self.handle)
    } 
}

impl Drop for Thread {
    fn drop(&mut self) {
        // If it's still active, finalize it
        if self.state == ThreadState::Started {
            let _ = self.join();
        }

        if self.owns_stack {
            alloc::release(self.stack, alloc::PAGE_ALIGNMENT, self.stack_size);
        }

        // If a thread is not created (like the main thread) the entry field will have nothing, and we definitely should not close threads we did not create...
        if !self.is_remote() {
            let _ = svc::close_handle(self.handle);
        }

        self.state = ThreadState::NotInitialized;
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct Tls {
    pub ipc_buffer: [u8; 0x100],
    pub preemption_state: u32,
    pub unk: [u8; 0xF4],
    pub thread_ref: *mut Thread,
}

#[inline(always)]
pub fn get_thread_local_storage() -> *mut Tls {
    let tls: *mut Tls;
    unsafe {
        llvm_asm!("mrs x0, tpidrro_el0" : "={x0}"(tls) ::: "volatile");
    }
    tls
}

pub fn set_current_thread(thread_ref: *mut Thread) {
    unsafe {
        (*thread_ref).self_ref = thread_ref;
        (*thread_ref).name_addr = &mut (*thread_ref).name as *mut _ as *mut u8;

        let tls = get_thread_local_storage();
        (*tls).thread_ref = thread_ref;
    }
}

pub fn get_current_thread() -> &'static mut Thread {
    unsafe {
        let tls = get_thread_local_storage();
        &mut *(*tls).thread_ref
    }
}

pub fn sleep(timeout: i64) -> Result<()> {
    svc::sleep_thread(timeout)
}

pub fn exit() -> ! {
    svc::exit_thread()
}