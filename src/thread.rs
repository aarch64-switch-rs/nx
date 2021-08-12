extern crate alloc;

use crate::result::*;
use crate::svc;
use crate::util;
use crate::mem;
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
        if let Some(entry) = (*thread_ref).entry {
            let entry_arg = (*thread_ref).entry_arg;
            (entry)(entry_arg);
        }
    }

    svc::exit_thread();
}

pub const INVALID_PRIORITY: i32 = -1;

#[repr(C)]
pub struct Thread {
    pub self_ref: *mut Thread,
    pub state: ThreadState,
    pub owns_stack: bool,
    pub pad: [u8; 2],
    pub handle: svc::Handle,
    pub stack: *mut u8,
    pub stack_size: usize,
    pub entry: Option<fn(*mut u8)>,
    pub entry_arg: *mut u8,
    pub tls_slots: [*mut u8; 0x20],
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
            entry_arg: ptr::null_mut(),
            tls_slots: [ptr::null_mut(); 0x20],
            reserved: [0; 0x54],
            name_len: 0,
            name: ThreadName::new(),
            name_addr: ptr::null_mut(),
            reserved_2: [0; 0x20],
        }
    }

    pub fn existing(handle: svc::Handle, name: &str, stack: *mut u8, stack_size: usize, owns_stack: bool, entry: Option<fn(*mut u8)>, entry_arg: *mut u8) -> Result<Self> {
        let mut thread = Self {
            self_ref: ptr::null_mut(),
            state: ThreadState::Started,
            owns_stack: owns_stack,
            pad: [0; 2],
            handle: handle,
            stack: stack,
            stack_size: stack_size,
            entry: entry,
            entry_arg: entry_arg,
            tls_slots: [ptr::null_mut(); 0x20],
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

    pub fn new(entry: fn(*mut u8), entry_arg: *mut u8, stack: *mut u8, stack_size: usize, name: &str) -> Result<Self> {
        let mut stack_value = stack;
        let mut owns_stack = false;
        if stack_value.is_null() {
            unsafe {
                let stack_layout = alloc::alloc::Layout::from_size_align_unchecked(stack_size, mem::PAGE_ALIGNMENT);
                stack_value = alloc::alloc::alloc(stack_layout);
                owns_stack = true;
            }
        }

        Self::existing(0, name, stack_value, stack_size, owns_stack, Some(entry), entry_arg)
    }

    pub fn create(&mut self, priority: i32, cpu_id: i32) -> Result<()> {
        let mut priority_value = priority;
        if priority_value == INVALID_PRIORITY {
            priority_value = get_current_thread().get_priority()?;
        }

        self.handle = svc::create_thread(thread_entry_impl, self as *mut _ as *mut u8, (self.stack as usize + self.stack_size) as *const u8, priority_value, cpu_id)?;
        Ok(())
    }

    pub fn get_handle(&self) -> svc::Handle {
        self.handle
    }

    pub fn get_priority(&self) -> Result<i32> {
        svc::get_thread_priority(self.handle)
    }

    pub fn get_id(&self) -> Result<u64> {
        svc::get_thread_id(self.handle)
    }

    pub fn start(&self) -> Result<()> {
        svc::start_thread(self.handle)
    }

    pub fn create_and_start(&mut self, priority: i32, cpu_id: i32) -> Result<()> {
        self.create(priority, cpu_id)?;
        self.start()
    }

    pub fn join(&self) -> Result<()> {
        svc::wait_synchronization(&self.handle, 1, -1)?;
        Ok(())
    }
}

impl Drop for Thread {
    fn drop(&mut self) {
        if self.owns_stack {
            unsafe {
                let stack_layout = alloc::alloc::Layout::from_size_align_unchecked(self.stack_size, mem::PAGE_ALIGNMENT);
                alloc::alloc::dealloc(self.stack, stack_layout);
            }
        }

        // If a thread is not created (like the main thread) the entry field will have nothing (Thread::empty), and we want to avoid closing threads we did not create :P
        if self.entry.is_some() {
            let _ = svc::close_handle(self.handle);
        }
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