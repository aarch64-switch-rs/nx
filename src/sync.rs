use crate::diag::abort;
use crate::svc;
use crate::thread;
use core::cell::UnsafeCell;

#[cfg(target_pointer_width = "64")]
use core::arch::asm;

#[cfg(target_pointer_width = "32")]
use core::arch::arm;

const HANDLE_WAIT_MASK: u32 = 0x40000000;

// TODO: 32-bit support is still broken...

#[inline(always)]
fn get_current_thread_handle() -> u32 {
    thread::get_current_thread().get_handle()
}

#[inline(always)]
fn load_exclusive(ptr: *const u32) -> u32 {
    let value: u32;

    #[cfg(target_pointer_width = "64")]
    unsafe {
        asm!(
            "ldaxr {0:w}, [{1:x}]",
            out(reg) value,
            in(reg) ptr
        );
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        value = arm::__ldrex(ptr);
    }

    value
}

#[inline(always)]
fn store_exclusive(ptr: *mut u32, value: u32) -> u32 {
    let res: u32;

    #[cfg(target_pointer_width = "64")]
    unsafe {
        asm!(
            "stlxr {0:w}, {1:w}, [{2:x}]",
            out(reg) res,
            in(reg) value,
            in(reg) ptr
        );
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        res = arm::__strex(value, ptr);
    }

    res
}

#[inline(always)]
fn clear_exclusive() {
    #[cfg(target_pointer_width = "64")]
    unsafe {
        asm!("clrex");
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        arm::__clrex();
    }
}

fn lock_impl(handle_ref: *mut u32) {
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

        match svc::arbitrate_lock(value & !HANDLE_WAIT_MASK, handle_ref as *mut u8, thr_handle) {
            Err(rc) => abort::abort(abort::AbortLevel::SvcBreak(), rc),
            _ => {}
        };

        value = load_exclusive(handle_ref);
        if (value & !HANDLE_WAIT_MASK) == thr_handle {
            clear_exclusive();
            break;
        }
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        arm::__dmb(arm::SY);
    }
}

fn unlock_impl(handle_ref: *mut u32) {
    let thr_handle = get_current_thread_handle();
    
    let mut value = load_exclusive(handle_ref);
    loop {
        if value != thr_handle {
            clear_exclusive();
            break;
        }

        #[cfg(target_pointer_width = "32")]
        unsafe {
            arm::__dmb(arm::SY);
        }

        if store_exclusive(handle_ref, 0) == 0 {
            break;
        }

        value = load_exclusive(handle_ref);
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        arm::__dmb(arm::SY);
    }

    if (value & HANDLE_WAIT_MASK) != 0 {
        // TODO: handle this instead of unwrapping it?
        svc::arbitrate_unlock(handle_ref as *mut u8).unwrap();
    }
}

fn try_lock_impl(handle_ref: *mut u32) -> bool {
    let thr_handle = get_current_thread_handle();

    loop {
        let value = load_exclusive(handle_ref);
        if value != svc::INVALID_HANDLE {
            break;
        }

        #[cfg(target_pointer_width = "32")]
        unsafe {
            arm::__dmb(arm::SY);
        }

        if store_exclusive(handle_ref, thr_handle) == 0 {
            return true;
        }
    }

    clear_exclusive();

    #[cfg(target_pointer_width = "32")]
    unsafe {
        arm::__dmb(arm::SY);
    }

    false
}

pub struct Mutex {
    value: u32,
    is_recursive: bool,
    counter: u32,
    thread_handle: u32,
}

impl Mutex {
    pub const fn new(recursive: bool) -> Self {
        Self { value: 0, is_recursive: recursive, counter: 0, thread_handle: 0 }
    }

    pub fn lock(&mut self) {
        let mut do_lock = true;
        if self.is_recursive {
            do_lock = false;
            let thr_handle = get_current_thread_handle();
            if self.thread_handle != thr_handle {
                do_lock = true;
                self.thread_handle = thr_handle;
            }
            self.counter += 1;
        }

        if do_lock {
            lock_impl(&mut self.value);
        }
    }

    pub fn unlock(&mut self) {
        let mut do_unlock = true;
        if self.is_recursive {
            do_unlock = false;
            self.counter -= 1;
            if self.counter == 0 {
                self.thread_handle = 0;
                do_unlock = true;
            }
        }

        if do_unlock {
            unlock_impl(&mut self.value);
        }
    }

    pub fn try_lock(&mut self) -> bool {
        if self.is_recursive {
            let thr_handle = get_current_thread_handle();
            if self.thread_handle != thr_handle {
                if !try_lock_impl(&mut self.value) {
                    return false;
                }
                self.thread_handle = thr_handle;
            }
            self.counter += 1;
            true
        }
        else {
            try_lock_impl(&mut self.value)
        }
    }
}

pub struct ScopedLock<'a> {
    lock: &'a mut Mutex,
}

impl<'a> ScopedLock<'a> {
    pub fn new(lock: &'a mut Mutex) -> Self {
        lock.lock();
        Self { lock }
    }
}

impl<'a> Drop for ScopedLock<'a> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

pub struct Locked<T> {
    lock_cell: UnsafeCell<Mutex>,
    object_cell: UnsafeCell<T>,
}

impl<T> Locked<T> {
    pub const fn new(recursive: bool, t: T) -> Self {
        Self { lock_cell: UnsafeCell::new(Mutex::new(recursive)), object_cell: UnsafeCell::new(t) }
    }

    pub const fn get_lock(&self) -> &mut Mutex {
        unsafe {
            &mut *self.lock_cell.get()
        }
    }

    pub fn get(&self) -> &mut T {
        self.get_lock().lock();
        let obj_ref = unsafe {
            &mut *self.object_cell.get()
        };
        self.get_lock().unlock();
        obj_ref
    }

    pub fn set(&mut self, t: T) {
        self.get_lock().lock();
        self.object_cell = UnsafeCell::new(t);
        self.get_lock().unlock();
    }
}

impl<T: Copy> Locked<T> {
    pub fn get_val(&self) -> T {
        self.get_lock().lock();
        let obj_copy = unsafe {
            *self.object_cell.get()
        };
        self.get_lock().unlock();
        obj_copy
    }
}