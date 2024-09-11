//! Threading support and wrappers

use crate::result::*;
use crate::svc;
use crate::mem::alloc;
use crate::wait;
use crate::util;
use core::alloc::Allocator;
use core::alloc::Layout;
use core::ops::Deref;
use core::pin::Pin;
use core::ptr;
use core::arch::asm;
use core::ptr::null;
use core::ptr::NonNull;

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

/// Represents what core the (thread)[`Thread`] should start on
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(i32)]
pub enum ThreadStartCore {
    #[default]
    Default = -2,
    Core0 = 0,
    Core1 = 1,
    Core2 = 2,
    Core3 = 3
}

/// Represents the priority of the (thread)[`Thread`] that we are spawning
#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
#[repr(i32)]
pub enum ThreadPriority {
    /// Inherit the thread priority from the current thread
    #[default]
    Inherit,
    /// Use the default priority for the main thread (0x2C)
    Default,
    /// Mark the thread for pre-emptive multitasking. Threads of this priority will be swapped out
    /// by the kernel every 10ms
    Preempt,
    /// Set and explicit thread priority value (0..=0x3F)
    Set(i32)
}

impl ThreadPriority {
    pub fn set(value: i32) -> Result<Self> {
        Self::try_from(value)
    }

    /// Returns the effective thread priority as the raw type (`i32`)
    pub fn to_raw(self, thread_core: ThreadStartCore) -> i32 {
        match self {
            Self::Inherit => {
                unsafe {get_current_thread()}
                .map(|thread_ref|svc::get_thread_priority(thread_ref.handle).unwrap() )
                .unwrap_or(0x2C)
            },
            Self::Default => 0x2C,
            Self::Preempt => {
                if thread_core == ThreadStartCore::Core3 || svc::get_current_processor_number() == (ThreadStartCore::Core3 as u32) {
                    0x3F // special value for preemption on Core 3
                } else {
                    0x3B  // special value for preemption on Core 0..=2
                }
            },
            Self::Set(v) => {
                v
            }
        }
    }
}

impl TryFrom<i32> for ThreadPriority {
    type Error = ResultCode;
    fn try_from(value: i32) -> Result<Self> {
        if ( 0i32 ..= 0x3F ).contains(&value) {
            Ok(Self::Set(value))
        } else {
            Err(rc::ResultInvalidPriority::make())
        }
    }
}

/// Represents the entrypoint information of a [`Thread`]
#[repr(C)]
struct ThreadEntryArgs {
    // we can't drop this at the end, as this is a self reference to the a Thread object that this struct is a member of.
    thread_object_pointer: *const Thread,
    entry: fn(),
    _args: usize,
    _reent: usize,
    _tls: usize,
    _padding: usize
}

impl ThreadEntryArgs {
    pub fn new(entry: fn()) -> Self {
        Self {
            thread_object_pointer: null(),
            entry,
            _args: 0,
            _reent: 0,
            _tls: 0,
            _padding: 0
        }
    }

    pub (self) fn set_thread_pointer(&mut self, thread_ref: *const Thread) {
        self.thread_object_pointer = thread_ref;
    }

    /// # SAFETY: The thread entry point must be set before calling this function
    unsafe fn run(&self) {
            (self.entry)();
    }
}

unsafe extern "C" fn thread_entry_impl(entry_args_ref: *mut u8) -> ! {
    let entry_args_ref = (entry_args_ref as *mut ThreadEntryArgs)
        .as_mut()
        .expect("We should never be sent a null pointer, or the thread context will be corrupted");
    
    
    let tlr = get_thread_local_region();
    (*tlr).thread_vars.thread = core::mem::transmute(entry_args_ref.thread_object_pointer);
    (*tlr).thread_vars._reent = (*entry_args_ref)._reent;
    (*tlr).thread_vars._tls_tp = 0; //we're not supporting thread local storage right now
    (*tlr).thread_vars.handle = (*entry_args_ref.thread_object_pointer).handle;
    (*tlr).thread_vars.magic = ThreadVars::MAGIC;

    entry_args_ref.run();
        
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
    pub state: ThreadState,
    pub owns_stack: bool,
    _pad: [u8; 2],
    pub handle: svc::Handle,
    pub stack: *mut u8,
    pub stack_size: usize,
    pub reserved: [u8; 0x26],
    pub version: u16,
    _reserved_2: [u8; 0xF8],
    entry: Option<ThreadEntryArgs>,
    pub reserved_3: [u8; 0x28],
    pub name: ThreadName,
    pub name_addr: *mut u8,
     _reserved_4: [u8; 0x20],
}

impl Thread { 
    /// Creates an empty, thus invalid [`Thread`]
    pub const fn new_invalid() -> Self {
        Self {
            state: ThreadState::NotInitialized,
            owns_stack: false,
            _pad: [0; 2],
            handle: 0,
            stack: ptr::null_mut(),
            stack_size: 0,
            reserved: [0; 0x26],
            version: CURRENT_THREAD_VERSION,
            _reserved_2: [0; 0xF8],
            entry: None,
            reserved_3: [0; 0x28],
            name: ThreadName::new(),
            name_addr: ptr::null_mut(),
            _reserved_4: [0; 0x20],
        }
    }

    fn new_impl(handle: svc::Handle, state: ThreadState, name: &str, stack: *mut u8, stack_size: usize, owns_stack: bool, entry: Option<ThreadEntryArgs>) -> Result<Self> {
        let mut thread = Self {
            state,
            owns_stack,
            _pad: [0; 2],
            handle,
            stack,
            stack_size,
            reserved: [0; 0x26],
            version: CURRENT_THREAD_VERSION,
            _reserved_2: [0; 0xF8],
            entry,
            reserved_3: [0; 0x28],
            name: ThreadName::new(),
            name_addr: ptr::null_mut(),
            _reserved_4: [0; 0x20],
        };
        //thread.name_addr = &mut thread.name as *mut ThreadName as *mut u8;
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
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_remote(handle: svc::Handle, name: &str, stack: *mut u8, stack_size: usize) -> Result<Self> {
        //result_return_unless!(!stack.is_null(), rc::ResultInvalidStack);
        //result_return_unless!(!stack.is_aligned_to(alloc::PAGE_ALIGNMENT), rc::ResultInvalidStack);

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
    /// * `stack`: The stack address. SAFETY: Must live as long as the thread is running
    /// * `stack_size`: The stack size
    #[allow(clippy::not_unsafe_ptr_arg_deref)]
    pub fn new_with_stack(entry: fn(), name: &str, stack: *mut u8, stack_size: usize) -> Result<Self> {
        result_return_unless!(!stack.is_null(), rc::ResultInvalidStack);
        result_return_unless!(!stack.is_aligned_to(alloc::PAGE_ALIGNMENT), rc::ResultInvalidStack);

        let thread_entry: ThreadEntryArgs = ThreadEntryArgs::new(entry);
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
    pub fn new(entry: fn(), name: &str, stack_size: usize) -> Result<Self> {
        let stack = unsafe {::alloc::alloc::Global.allocate(Layout::from_size_align_unchecked(stack_size, alloc::PAGE_ALIGNMENT))}?;

        let thread_entry: ThreadEntryArgs = ThreadEntryArgs::new(entry);
        Self::new_impl(svc::INVALID_HANDLE, ThreadState::NotInitialized, name, stack.as_mut_ptr().cast(), stack_size, true, Some(thread_entry))
    }
    /// Creates a new [`Thread`] with an entrypoint + args, name and stack
    /// 
    /// Same as calling [`Thread::new_with_stack`] but with the stack a pre-allocated slice
    /// 
    /// Note that it needs to be initialized ([`Thread::initialize()`]) before being started ([`Thread::start()`])
    /// 
    /// # Arguments
    /// 
    /// * `entry`: The entrypoint function, taking args
    /// * `args`: The entrypoint arguments
    /// * `name`: The desired thread name
    /// * `stack`: The pre-allocated stack memory for the thread
    pub fn new_with_buffer(entry: fn(), name: &str, stack: &mut [u8]) -> Result<Self> {
        let thread_entry: ThreadEntryArgs = ThreadEntryArgs::new(entry);
        Self::new_impl(svc::INVALID_HANDLE, ThreadState::NotInitialized, name, stack.as_mut_ptr(), stack.len(), false, Some(thread_entry))
    }

    /// Initializes a [`Thread`]
    /// 
    /// Technically, this actually "creates" it using [`svc::create_thread`]
    /// 
    /// It must be in [`ThreadState::NotInitialized`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// # Arguments
    /// 
    /// * `priority`: The desired priority
    /// * `processor_id`: The desired processor ID to start the thread on
    pub fn initialize(self: &mut Pin<&mut Self>, priority: ThreadPriority, processor_id: ThreadStartCore) -> Result<()> {
        result_return_unless!(self.state == ThreadState::NotInitialized, rc::ResultInvalidState);
        result_return_unless!(self.entry.is_some(), rc::ResultInvalidState);

        let raw_self_ptr: *const Self = self.as_mut().deref();

        let stack_top_addr = unsafe {self.stack.add(self.stack_size)};
        let priority = priority.to_raw(processor_id);
        let entry_mut = self.entry.as_mut();
        let entry_mut = entry_mut.ok_or(rc::ResultInvalidState::make())?;
        entry_mut.set_thread_pointer(raw_self_ptr);

        match unsafe {svc::create_thread(thread_entry_impl, entry_mut as *mut _ as *mut u8, stack_top_addr, priority, processor_id as i32)} {
            Ok(handle) => {
                self.handle = handle;
            },
            Err(e) => {
                return Err(e);
            }
        }        
        self.state = ThreadState::Initialized;
        Ok(())
    }

    /// Starts the [`Thread`]
    /// 
    /// It must be in [`ThreadState::Initialized`] or [`ThreadState::Terminated`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially uses [`svc::start_thread`]
    pub fn start(self: &mut Pin<&mut Self>) -> Result<()> {
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

    /// Joins the [`Thread`], with a timout
    /// 
    /// It must be in [`ThreadState::Started`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially waits for the thread handle, which will signal when it finishes
    pub fn join_wait(self: &mut Pin<&mut Self>, timeout: i64) -> Result<()> {
        result_return_unless!(self.state == ThreadState::Started, rc::ResultInvalidState);
        
        wait::wait_handles(&[self.handle], timeout)?;

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
    pub fn handle(&self) -> svc::Handle {
        self.handle
    }

    /// Gets this [`Thread`]'s priority
    /// 
    /// It must be in any state but [`ThreadState::NotInitialized`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially uses [`svc::get_thread_priority`]
    pub fn priority(&self) -> Result<i32> {
        result_return_unless!(self.state != ThreadState::NotInitialized, rc::ResultInvalidState);

        svc::get_thread_priority(self.handle)
    }

    /// Gets this [`Thread`]'s ID
    /// 
    /// It must be in any state but [`ThreadState::NotInitialized`], otherwise this will fail with [`ResultInvalidState`][`rc::ResultInvalidState`]
    /// 
    /// Essentially uses [`svc::get_thread_id`]
    pub fn id(&self) -> Result<u64> {
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
            // we know that this was created with a matching call to the Global allocator in Self::new
            // as that is the only way to get an owned stack.
            unsafe {::alloc::alloc::Global.deallocate( ptr::NonNull::new_unchecked(self.stack), Layout::from_size_align_unchecked(self.stack_size, alloc::PAGE_ALIGNMENT)) };
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
    _reserved_1: [u8; 0x4],
    _reserved_2: [u8; 0x78],
    _tls: [u8; 0x50],
    _locale_ptr: usize,
    _errno: i64,
    /*
    pub thread_data: [u8; 0x8],
    pub eh_globals: [u8; 0x8],
    pub thread_ptr: *mut u8,
    /// The region we (and Nintendo) use to store the current [`Thread`] reference
    pub thread_ref: *mut Thread<'t, 'e>,
    */

    /// we diverge here to implement a libnx-like thread structure so that we know the kernel won't freak out during crash reporting/debugging.
    pub(crate) thread_vars: ThreadVars
}
const_assert!(core::mem::size_of::<ThreadLocalRegion>() == 0x200);

/// Libnx's thread information struct, including a self reference to the thread's descriptor object
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct ThreadVars {
    /// magic value '!TV$'
    magic: u32,
    
    // Thread handle, for mutexes
    handle: svc::Handle,

    // Pointer to the current thread (if exists)
    pub (crate) thread: *mut Thread,

    // Pointer to this thread's newlib state - unusued
    _reent: usize,
    
    // Pointer to this thread's thread-local segment - unused
    _tls_tp: usize

}

impl ThreadVars {
    const MAGIC: u32 = 0x21545624; // !TV$
}

/// Gets the current thread's [`ThreadLocalRegion`] address
/// # SAFETY: This should never be sent over thread boundaries
#[inline(always)]
pub unsafe fn get_thread_local_region() -> *mut ThreadLocalRegion {
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
unsafe fn set_current_thread(thread_ref: NonNull<Thread>) {
        let tlr = get_thread_local_region();
        (*tlr).thread_vars.thread = thread_ref.as_ptr();
}

/// Get's the current [`Thread`] reference
/// 
/// This is done using the stored reference in the current [`ThreadLocalRegion`]
/// # SAFETY: This unconditionally returns a reference to the current thread's `Thread` object (if set). This must only be called once in a thread or it will break Rust's mutable aliasing rule.
#[inline]
pub unsafe fn get_current_thread() -> Option<&'static mut Thread> {
    unsafe {
        let tlr = get_thread_local_region();
        debug_assert!(!tlr.is_null());
        (*tlr).thread_vars.thread.as_mut()
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

/// Represents the thread yielding types for cooperative multitasking
#[repr(i64)]
pub enum YieldType{ 
    /// Yields to another thread on the same core
    WithoutCoreMigration = 0,
    /// Yields to another thread (possibly on a different core)
    WithCoreMigration = -1,
    /// Yields and performs forced load-balancing
    ToAnyThread  = -2
}

#[inline]
pub fn r#yield(yield_type: YieldType) -> Result<()> {
    svc::sleep_thread(yield_type as i64)
}

/// Exits the current thread
/// 
/// Essentially a wrapper for [`svc::exit_thread`]
#[inline]
pub fn exit() -> ! {
    svc::exit_thread()
}

pub mod builder {

    use super::*;

    pub struct Builder;
    impl Builder {
        fn new(entry: fn(), name:&str, stack_size: usize) -> Result<UninitThread> {
            Ok(UninitThread(Thread::new(entry, name, stack_size)?))
        }
    }
    struct UninitThread(Thread);
}