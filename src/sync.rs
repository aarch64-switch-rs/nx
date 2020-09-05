use crate::svc;
use crate::thread;

pub struct Mutex {
    value: u32,
    is_recursive: bool,
    counter: u32,
    thread_handle: u32,
}

const HANDLE_WAIT_MASK: u32 = 0x40000000;

fn get_current_thread_handle() -> u32 {
    thread::get_current_thread().get_handle()
}

fn load_exclusive(ptr: *mut u32) -> u32 {
    let value: u32;
    unsafe {
        llvm_asm!("ldaxr w0, [x1]" : "={w0}"(value) : "{x1}"(ptr) : "memory" : "volatile");
    }
    value
}

fn store_exclusive(ptr: *mut u32, value: u32) -> i32 {
    let res: i32;
    unsafe {
        llvm_asm!("stlxr w0, w1, [x2]" : "={w0}"(res) : "{w1}"(value), "{x2}"(ptr) : "memory" : "volatile");
    }
    res
}

fn clear_exclusive() {
    unsafe {
        llvm_asm!("clrex" ::: "memory" : "volatile");
    }
}

fn lock_impl(handle_ref: *mut u32) {
    let thr_handle = get_current_thread_handle();
    
    let mut value = load_exclusive(handle_ref);
    loop {
        if value == 0 {
            if store_exclusive(handle_ref, thr_handle) != 0 {
                value = load_exclusive(handle_ref);
                continue;
            }
            break;
        }
        if (value & HANDLE_WAIT_MASK) == 0 {
            if store_exclusive(handle_ref, value | HANDLE_WAIT_MASK) != 0 {
                value = load_exclusive(handle_ref);
                continue;
            }
        }

        // TODO: handle this instead of unwrapping it?
        svc::arbitrate_lock(value & !HANDLE_WAIT_MASK, handle_ref as *mut u8, thr_handle).unwrap();

        value = load_exclusive(handle_ref);
        if (value & !HANDLE_WAIT_MASK) == thr_handle {
            clear_exclusive();
            break;
        }
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

fn try_lock_impl(handle_ref: *mut u32) -> bool {
    let thr_handle = get_current_thread_handle();

    loop {
        let value = load_exclusive(handle_ref);
        if value != 0 {
            break;
        }
        if store_exclusive(handle_ref, thr_handle) == 0 {
            return true;
        }
    }

    clear_exclusive();
    false
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
        Self { lock: lock }
    }
}

impl<'a> Drop for ScopedLock<'a> {
    fn drop(&mut self) {
        self.lock.unlock();
    }
}

pub struct Locked<T> {
    lock: Mutex,
    object: T,
}

impl<T> Locked<T> {
    pub const fn new(recursive: bool, t: T) -> Self {
        Self { lock: Mutex::new(recursive), object: t }
    }

    pub fn get(&mut self) -> &mut T {
        self.lock.lock();
        let obj_ref = &mut self.object;
        self.lock.unlock();
        obj_ref
    }

    pub fn set(&mut self, t: T) {
        self.lock.lock();
        self.object = t;
        self.lock.unlock();
    }

    pub fn get_lock(&self) -> &Mutex {
        &self.lock
    }
}