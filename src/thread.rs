//! Threading support and wrappers

use crate::result::*;
use crate::svc;
use crate::mem::alloc;
use crate::wait;
use crate::util;
use core::ptr;
use core::arch::asm;

pub mod rc;

/// Represents a [`Thread`]'s name, a 32-byte `CString`
pub type ThreadName = util::CString<0x20>;

/// Represents the state of a [`Thread`]
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

/// Represents the entrypoint information of a [`Thread`]
pub struct ThreadEntry {
    pub entry_impl: svc::ThreadEntrypointFn,
    pub raw_entry: *const u8,
    pub raw_args: *const u8
}

impl ThreadEntry {
    pub fn new<T: Copy, F: 'static + Fn(&T)>(entry_impl: svc::ThreadEntrypointFn, entry: F, args: &T) -> Self {
        Self {
            entry_impl,
            raw_entry: &entry as *const _ as *const u8,
            raw_args: args as *const _ as *const u8
        }
    }

    pub fn run<T: Copy, F: 'static + Fn(&T)>(&self) {
        unsafe {
            let entry = self.raw_entry as *const F;
            let args = &*(self.raw_args as *const T);
            (*entry)(args);
        }
    }
}

extern fn thread_entry_impl<T: Copy, F: 'static + Fn(&T)>(thread_ref_v: *mut u8) -> ! {
    let thread_ref = thread_ref_v as *mut Thread;
    set_current_thread(thread_ref);

    unsafe {
        if let Some(entry_ref) = (*thread_ref).entry.as_ref() {
            entry_ref.run::<T, F>();
        }
    }

    exit()
}

/// Represents a meta-value for the priority of a new [`Thread`] to be determined by the current [`Thread`]'s priority
pub const PRIORITY_AUTO: i32 = -1;

// Note: our thread type attempts to kind-of mimic the official nn::os::ThreadType struct, at least so that the thread name is properly accessible from TLS by, for instance, creport -- thus all the reserved fields
// We act like nn::os::ThreadType version 1

const CURRENT_THREAD_VERSION: u16 = 1;

/// Represents a thread
#[repr(C)]
pub struct Thread {
    pub self_ref: *mut Thread,
    pub state: ThreadState,
    pub owns_stack: bool,
    pub pad: [u8; 2],
    pub handle: svc::Handle,
    pub stack: *mut u8,
    pub stack_size: usize,
    pub reserved: [u8; 0x26],
    pub version: u16,
    pub reserved_2: [u8; 0xF8],
    pub entry: Option<ThreadEntry>,
    pub reserved_3: [u8; 0x28],
    pub name: ThreadName,
    pub name_addr: *mut u8,
    pub reserved_4: [u8; 0x20]
}

impl Thread {
    /// Creates an empty, thus invalid [`Thread`]
    pub const fn empty() -> Self {
        Self {
            self_ref: ptr::null_mut(),
            state: ThreadState::NotInitialized,
            owns_stack: false,
            pad: [0; 2],
            handle: 0,
            stack: ptr::null_mut(),
            stack_size: 0,
            reserved: [0; 0x26],
            version: CURRENT_THREAD_VERSION,
            reserved_2: [0; 0xF8],
            entry: None,
            reserved_3: [0; 0x28],
            name: ThreadName::new(),
            name_addr: ptr::null_mut(),
            reserved_4: [0; 0x20]
        }
    }

    fn new_impl(handle: svc::Handle, state: ThreadState, name: &str, stack: *mut u8, stack_size: usize, owns_stack: bool, entry: Option<ThreadEntry>) -> Result<Self> {
        let mut thread = Self {
            self_ref: ptr::null_mut(),
            state,
            owns_stack,
            pad: [0; 2],
            handle,
            stack,
            stack_size,
            reserved: [0; 0x26],
            version: CURRENT_THREAD_VERSION,
            reserved_2: [0; 0xF8],
            entry,
            reserved_3: [0; 0x28],
            name: ThreadName::new(),
            name_addr: ptr::null_mut(),
            reserved_4: [0; 0x20]
        };
        thread.self_ref = &mut thread;
        thread.name_addr = &mut thread.name as *mut ThreadName as *mut u8;
        thread.name.set_str(name);
        Ok(thread)
    }

    /// Creates a [`Thread`] from an existing thread handle, a name, and stack
    /// 
    /// # Arguments
    /// 
    /// * `handle`: The remote thread handle
    /// * `name`: The custom thread name
    /// * `stack`: The remote stack address
    /// * `stack_size`: The remote stack size
    #[inline]
    pub fn new_remote(handle: svc::Handle, name: &str, stack: *mut u8, stack_size: usize) -> Result<Self> {
        Self::new_impl(handle, ThreadState::Started, name, stack, stack_size, false, None)
    }
    
    /// Creates a new [`Thread`] with an entrypoint + args, name and stack
    /// 
    /// Note that it needs to be initialized ([`Thread::initialize`]) before being started ([`Thread::start`])
    /// 
    /// # Arguments
    /// 
    /// * `entry`: The entrypoint function, taking args
    /// * `args`: The entrypoint arguments
    /// * `name`: The desired thread name
    /// * `stack`: The stack address
    /// * `stack_size`: The stack size
    pub fn new_with_stack<T: Copy, F: 'static + Fn(&T)>(entry: F, args: &T, name: &str, stack: *mut u8, stack_size: usize) -> Result<Self> {
        result_return_unless!(!stack.is_null(), rc::ResultInvalidStack);
        // TODO: also check alignment

        let thread_entry = ThreadEntry::new(thread_entry_impl::<T, F>, entry, args);
        Self::new_impl(svc::INVALID_HANDLE, ThreadState::NotInitialized, name, stack, stack_size, false, Some(thread_entry))
    }
    
    /// Creates a new [`Thread`] with an entrypoint + args, name and stack
    /// 
    /// Same as calling [`Thread::new_with_stack`] but with the stack being automatically allocated from heap
    /// 
    /// Note that it needs to be initialized ([`Thread::initialize()`]) before being started ([`Thread::start()`])
    /// 
    /// # Arguments
    /// 
    /// * `entry`: The entrypoint function, taking args
    /// * `args`: The entrypoint arguments
    /// * `name`: The desired thread name
    /// * `stack_size`: The desired stack size
    pub fn new<T: Copy, F: 'static + Fn(&T)>(entry: F, args: &T, name: &str, stack_size: usize) -> Result<Self> {
        let stack = alloc::allocate(alloc::PAGE_ALIGNMENT, stack_size)?;

        let thread_entry = ThreadEntry::new(thread_entry_impl::<T, F>, entry, args);
        Self::new_impl(svc::INVALID_HANDLE, ThreadState::NotInitialized, name, stack, stack_size, true, Some(thread_entry))
    }

    /// Initializes a [`Thread`]
    /// 
    /// Technically, this actually "creates" it using [`svc::create_thread`]
    /// 
    /// It must be in [`ThreadState::NotInitialized`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// # Arguments
    /// 
    /// * `priority`: The desired priority, or [`PRIORITY_AUTO`] to use the current thread value
    /// * `processor_id`: The desired processor ID, in `[0-3]` range or [`svc::DEFAULT_PROCESS_PROCESSOR_ID`] for the process's default value
    pub fn initialize(&mut self, priority: i32, processor_id: i32) -> Result<()> {
        result_return_unless!(self.state == ThreadState::NotInitialized, rc::ResultInvalidState);

        let mut priority_value = priority;
        if priority_value == PRIORITY_AUTO {
            priority_value = get_current_thread().get_priority()?;
        }

        self.handle = svc::create_thread(self.entry.as_ref().unwrap().entry_impl, self as *mut _ as *mut u8, (self.stack as usize + self.stack_size) as *const u8, priority_value, processor_id)?;
        
        self.state = ThreadState::Initialized;
        Ok(())
    }

    /// Starts the [`Thread`]
    /// 
    /// It must be in [`ThreadState::Initialized`] or [`ThreadState::Terminated`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially uses [`svc::start_thread`]
    pub fn start(&mut self) -> Result<()> {
        result_return_unless!((self.state == ThreadState::Initialized) || (self.state == ThreadState::Terminated), rc::ResultInvalidState);

        svc::start_thread(self.handle)?;

        self.state = ThreadState::Started;
        Ok(())
    }

    /// Joins the [`Thread`]
    /// 
    /// It must be in [`ThreadState::Started`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially waits for the thread handle, which will signal when it finishes
    pub fn join(&mut self) -> Result<()> {
        result_return_unless!(self.state == ThreadState::Started, rc::ResultInvalidState);
        
        wait::wait_handles(&[self.handle], -1)?;

        self.state = ThreadState::Terminated;
        Ok(())
    }

    /// Gets whether this [`Thread`] is remote
    #[inline]
    pub fn is_remote(&self) -> bool {
        self.entry.is_none()
    }

    /// Gets this [`Thread`]'s handle
    #[inline]
    pub fn get_handle(&self) -> svc::Handle {
        self.handle
    }

    /// Gets this [`Thread`]'s priority
    /// 
    /// It must be in any state but [`ThreadState::NotInitialized`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially uses [`svc::get_thread_priority`]
    pub fn get_priority(&self) -> Result<i32> {
        result_return_unless!(self.state != ThreadState::NotInitialized, rc::ResultInvalidState);

        svc::get_thread_priority(self.handle)
    }

    /// Gets this [`Thread`]'s ID
    /// 
    /// It must be in any state but [`ThreadState::NotInitialized`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially uses [`svc::get_thread_id`]
    pub fn get_id(&self) -> Result<u64> {
        result_return_unless!(self.state != ThreadState::NotInitialized, rc::ResultInvalidState);
        
        svc::get_thread_id(self.handle)
    } 
}

impl Drop for Thread {
    /// Destroys the [`Thread`], doing the following:
    /// * Waits for it to finish (see [`Thread::join`]) if it's running
    /// * Frees the stack memory if it was automatically allocated on creation
    /// * Closes the thread handle if it isn't remote, effectively closing the thread
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

// Note: https://switchbrew.org/wiki/Thread_Local_Region

/// Represents the console's Thread Local Region layout
#[derive(Copy, Clone)]
#[repr(C)]
pub struct ThreadLocalRegion {
    /// The region used for IPC messages
    pub msg_buffer: [u8; 0x100],
    /// The disabled counter
    pub disable_counter: u16,
    /// The interrupt flag
    pub interrupt_flag: u16,
    pub reserved_1: [u8; 0x4],
    pub reserved_2: [u8; 0x78],
    pub tls: [u8; 0x50],
    pub locale_ptr: *mut u8,
    pub errno_val: i64,
    pub thread_data: [u8; 0x8],
    pub eh_globals: [u8; 0x8],
    pub thread_ptr: *mut u8,
    /// The region we (and Nintendo) use to store the current [`Thread`] reference
    pub thread_ref: *mut Thread,
}
const_assert!(core::mem::size_of::<ThreadLocalRegion>() == 0x200);

/// Gets the current thread's [`ThreadLocalRegion`] address
#[inline(always)]
pub fn get_thread_local_region() -> *mut ThreadLocalRegion {
    let tlr: *mut ThreadLocalRegion;
    unsafe {
        asm!(
            "mrs {}, tpidrro_el0",
            out(reg) tlr
        );
    }
    tlr
}

/// Sets the current [`Thread`] reference on the current [`ThreadLocalRegion`]
/// 
/// This is internally used when launching a [`Thread`], and probably shouldn't be used manually
/// 
/// # Arguments
/// 
/// * `thread_ref`: The [`Thread`] address to set
pub fn set_current_thread(thread_ref: *mut Thread) {
    unsafe {
        (*thread_ref).self_ref = thread_ref;
        (*thread_ref).name_addr = &mut (*thread_ref).name as *mut _ as *mut u8;

        let tlr = get_thread_local_region();
        (*tlr).thread_ref = thread_ref;
    }
}

/// Get's the current [`Thread`] reference
/// 
/// This is done using the stored reference in the current [`ThreadLocalRegion`]
#[inline]
pub fn get_current_thread() -> &'static mut Thread {
    unsafe {
        let tlr = get_thread_local_region();
        &mut *(*tlr).thread_ref
    }
}

/// Sleeps for the given timeout
/// 
/// Essentially a wrapper for [`svc::sleep_thread`]
/// 
/// # Arguments
/// 
/// * `timeout`: Sleep timeout in nanoseconds, where `0` can be used for yielding without core migration, `-1` for yielding with core migration and `-2` for yielding to any other thread
#[inline]
pub fn sleep(timeout: i64) -> Result<()> {
    svc::sleep_thread(timeout)
}

/// Exits the current thread
/// 
/// Essentially a wrapper for [`svc::exit_thread`]
#[inline]
pub fn exit() -> ! {
    svc::exit_thread()
}