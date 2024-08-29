//! Synchronization support and utils

use crate::diag::abort;
use crate::svc;
use crate::thread;
use core::cell::UnsafeCell;
use core::arch::asm;

const HANDLE_WAIT_MASK: u32 = 0x40000000;

#[inline(always)]
fn get_current_thread_handle() -> u32 {
    thread::get_current_thread().get_handle()
}

#[inline(always)]
unsafe fn load_exclusive(ptr: *mut u32) -> u32 {
    let value: u32;
    unsafe {
        asm!(
            "ldaxr {0:w}, [{1:x}]",
            out(reg) value,
            in(reg) ptr
        );
    }
    value
}

#[inline(always)]
unsafe fn store_exclusive(ptr: *mut u32, value: u32) -> i32 {
    let res: i32;
    unsafe {
        asm!(
            "stlxr {0:w}, {1:w}, [{2:x}]",
            out(reg) res,
            in(reg) value,
            in(reg) ptr
        );
    }
    res
}

#[inline(always)]
fn clear_exclusive() {
    unsafe {
        asm!("clrex");
    }
}

unsafe fn lock_impl(handle_ref: *mut u32) {
    let thr_handle = get_current_thread_handle();
    
    let mut value = load_exclusive(handle_ref);
    loop {
        if value == svc::INVALID_HANDLE {
            if store_exclusive(handle_ref, thr_handle) != 0 {
                value = load_exclusive(handle_ref);
                continue;
            }
            break;
        }
        if (value & HANDLE_WAIT_MASK) == 0 && store_exclusive(handle_ref, value | HANDLE_WAIT_MASK) != 0 {
            value = load_exclusive(handle_ref);
            continue;
        }

        if let Err(rc) = svc::arbitrate_lock(value & !HANDLE_WAIT_MASK, handle_ref as *mut u8, thr_handle) {
            abort::abort(abort::AbortLevel::SvcBreak(), rc);
        }

        value = load_exclusive(handle_ref);
        if (value & !HANDLE_WAIT_MASK) == thr_handle {
            clear_exclusive();
            break;
        }
    }
}

unsafe fn unlock_impl(handle_ref: *mut u32) {
    let thr_handle = get_current_thread_handle();
    
    let mut value = load_exclusive(handle_ref);
    loop {
        if value != thr_handle {
            clear_exclusive();
            break;
        }

        if store_exclusive(handle_ref, 0) == 0 {
            break;
        }

        value = load_exclusive(handle_ref);
    }

    if (value & HANDLE_WAIT_MASK) != 0 {
        // TODO: handle this instead of unwrapping it?
        svc::arbitrate_unlock(handle_ref as *mut u8).unwrap();
    }
}

unsafe fn try_lock_impl(handle_ref: *mut u32) -> bool {
    let thr_handle = get_current_thread_handle();

    loop {
        let value = load_exclusive(handle_ref);
        if value != svc::INVALID_HANDLE {
            break;
        }

        if store_exclusive(handle_ref, thr_handle) == 0 {
            return true;
        }
    }

    clear_exclusive();

    false
}

/// Represents a locking/unlocking type
pub struct Mutex {
    tag: u32,
    _thread_handle: u32,
}

impl Mutex {
    /// Creates a new [`Mutex`]
    #[inline]
    pub const fn new() -> Self {
        Self { tag: 0, _thread_handle: 0 }
    }

    /// Locks the [`Mutex`]
    pub fn lock(&mut self) {
        // SAFETY: We know this is OK as we are passing a valid &mut u32
        unsafe {lock_impl(&mut self.tag)};
    }

    /// Checks whether the [`Mutex`] is locked by the current thread.
    pub fn is_locked_by_current_thread(&self) -> bool {
        // If the lock is held by the current thread, then this thread write it with an exlusive lock
        // therefore it should never require an exclusive read to observe our own handle.
        self.tag & !HANDLE_WAIT_MASK == get_current_thread_handle()
    } 

    /// Unlocks the [`Mutex`]
    pub fn unlock(&mut self) {
        // SAFETY: We know this is OK as we are passing a valid &mut u32
        unsafe {unlock_impl(&mut self.tag)};
    }

    /// Attempts to lock the [`Mutex`], returning whether it was successful
    pub fn try_lock(&mut self) -> bool {
        unsafe {try_lock_impl(&mut self.tag)}
    }
}

/// Represents a type which will lock a given [`Mutex`] on creation and unlock it on destruction, effectively guarding it
pub struct ScopedLock<'a> {
    lock: &'a mut Mutex,
}

impl<'a> ScopedLock<'a> {
    /// Creates a new [`ScopedLock`] for a given [`Mutex`]
    /// 
    /// # Arguments
    /// 
    /// * `lock`: The [`Mutex`] to guard
    pub fn new(lock: &'a mut Mutex) -> Self {
        lock.lock();
        Self { lock }
    }
}

impl<'a> Drop for ScopedLock<'a> {
    /// Unlocks the [`Mutex`] as the [`ScopedLock`] is destroyed (likely out of scope)
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

/// Represents a value whose access is controlled by an inner [`Mutex`]
pub struct Locked<T: ?Sized> {
    pub (self) lock_cell: UnsafeCell<Mutex>,
    pub(self) object_cell: UnsafeCell<T>,
}

impl<T> Locked<T> {
    pub fn is_locked(&self) -> bool {
        unsafe { (*self.lock_cell.get()).tag != 0}
    }
    /// Creates a new [`Locked`] with a value
    /// 
    /// # Arguments
    /// 
    /// * `is_recursive`: Whether the inner [`Mutex`] is recursive
    /// * `t`: The value to store
    #[inline]
    pub const fn new(t: T) -> Self {
        Self { lock_cell: UnsafeCell::new(Mutex::new()), object_cell: UnsafeCell::new(t) }
    }

    /*/// Gets a reference to the inner [`Mutex`]
    #[inline]
    const fn get_lock(&self) -> &mut Mutex {
        unsafe {
            &mut *self.lock_cell.get()
        }
    }*/

    /*
    /// Gets a reference of the value, doing a lock-unlock operation in the process
    pub fn get(&self) -> &mut T {
        unsafe {
            (&mut *self.lock_cell.get()).lock();
            let obj_ref = &mut *self.object_cell.get();
            (&mut *self.lock_cell.get()).unlock();
            obj_ref
        }
    }
    */

    /// Sets a value, doing a lock-unlock operation in the process
    pub fn set(&mut self, t: T) {
        unsafe {
            (&mut *self.lock_cell.get()).lock();
            self.object_cell = UnsafeCell::new(t);
            (&mut *self.lock_cell.get()).unlock();
        }
    }

    /// Locks the Mutex and returns a guarded reference to the inner value
    pub fn lock(&self) -> LockGuard<'_, T>{
        unsafe {&mut *self.lock_cell.get()}.lock();
        LockGuard { lock: self }
    }
}


impl<T: Copy> Locked<T> {
    /// Gets a copy of the value, doing a lock-unlock operation in the process
    pub fn get_val(&self) -> T {
        unsafe {
            (&mut *self.lock_cell.get()).lock();
            let obj_copy = *self.object_cell.get();
            (&mut *self.lock_cell.get()).unlock();
            obj_copy
        }
    }
}
unsafe impl<T: ?Sized + Send> Sync for Locked<T> {}
unsafe impl<T: ?Sized + Send> Send for Locked<T> {}

pub struct LockGuard<'borrow, T:?Sized> {
    pub(self) lock: &'borrow Locked<T>
}

unsafe impl<'a, T: ?Sized + Sync> Sync for LockGuard<'a, T> {}

impl<'borrow, T: ?Sized> LockGuard<'borrow, T> {
    pub fn new(lock: &'borrow Locked<T>) -> Self {
        unsafe {&mut *lock.lock_cell.get()}.lock();
        Self {
            lock
        }
    }
}

impl<'borrow, T> core::ops::Deref for LockGuard<'borrow, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.lock.object_cell.get() }
    }
}

impl<'borrow, T> core::ops::DerefMut for LockGuard<'borrow, T> {

    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.object_cell.get() }
    }
}

impl<T: ?Sized> Drop for LockGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {&mut *self.lock.lock_cell.get()}.unlock();
    }
}