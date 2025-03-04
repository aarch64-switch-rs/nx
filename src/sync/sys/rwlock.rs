use core::sync::atomic::Ordering::{Acquire, Release};
use core::sync::atomic::{AtomicBool, AtomicU32};

use super::get_current_thread_handle;
use super::mutex::Mutex;

use crate::diag::abort;
use crate::svc;

#[repr(transparent)]
struct CondVar(u32);

impl CondVar {
    const fn new() -> Self {
        Self(0)
    }

    fn wait(&self, mutex: &Mutex) {
        if let Err(rc) = unsafe {
            svc::wait_process_wide_key_atomic(
                mutex as *const _ as _,
                self as *const Self as _,
                get_current_thread_handle(),
                -1,
            )
        } {
            abort::abort(abort::AbortLevel::Panic(), rc);
        }
    }

    fn signal_one(&self) {
        if let Err(rc) = unsafe { svc::signal_process_wide_key(self as *const Self as _, 1) } {
            abort::abort(abort::AbortLevel::Panic(), rc);
        }
    }

    fn signal_all(&self) {
        if let Err(rc) = unsafe { svc::signal_process_wide_key(self as *const Self as _, -1) } {
            abort::abort(abort::AbortLevel::Panic(), rc);
        }
    }
}

// we're going to do it like libnx to start, then maybe we can write our own.

pub struct RwLock {
    lock: Mutex,
    reader_waiting: CondVar,
    writer_waiting: CondVar,
    reader_lock_count: AtomicU32,
    writer_locked: AtomicBool,
    read_waiter_count: AtomicU32,
    write_waiter_count: AtomicU32,
}

impl RwLock {
    pub const fn new() -> Self {
        Self {
            lock: Mutex::new(),
            reader_waiting: CondVar::new(),
            writer_waiting: CondVar::new(),
            reader_lock_count: AtomicU32::new(0),
            writer_locked: AtomicBool::new(false),
            read_waiter_count: AtomicU32::new(0),
            write_waiter_count: AtomicU32::new(0),
        }
    }

    pub fn read(&self) {
        self.lock.lock();

        while self.writer_locked.load(Acquire) {
            assert!(
                self.read_waiter_count.fetch_add(1, Release) < u32::MAX,
                "About to overflow read waiter count."
            );
            self.reader_waiting.wait(&self.lock);
            self.read_waiter_count.fetch_sub(1, Release);
        }

        assert!(
            self.reader_lock_count.fetch_add(1, Release) < u32::MAX,
            "About to overflow reader lock count."
        );

        unsafe { self.lock.unlock() };
    }

    pub fn try_read(&self) -> bool {
        if !self.lock.try_lock() {
            return false;
        }

        let no_write_lock = !self.writer_locked.load(Acquire);
        if no_write_lock {
            assert!(
                self.reader_lock_count.fetch_add(1, Release) < u32::MAX,
                "About to overflow reader lock count."
            );
        }

        unsafe { self.lock.unlock() };

        no_write_lock
    }

    pub unsafe fn read_unlock(&self) {
        self.lock.lock();

        if self.reader_lock_count.fetch_sub(1, Release) == 1
            && self.write_waiter_count.load(Acquire) > 0
        {
            // we have just reduced the reader count to 0, signal a waiting writer
            self.writer_waiting.signal_one();
        }

        unsafe { self.lock.unlock() };
    }

    pub fn write(&self) {
        self.lock.lock();

        while self.reader_lock_count.load(Acquire) > 0 {
            assert!(
                self.write_waiter_count.fetch_add(1, Release) < u32::MAX,
                "About to overflow read waiter count."
            );
            self.writer_waiting.wait(&self.lock);
            self.write_waiter_count.fetch_sub(1, Release);
        }

        self.writer_locked.store(true, Release);

        unsafe { self.lock.unlock() };
    }

    pub fn try_write(&self) -> bool {
        if !self.lock.try_lock() {
            return false;
        }

        if self.writer_locked.load(Acquire) || self.reader_lock_count.load(Acquire) > 0 {
            unsafe { self.lock.unlock() };
            return false;
        }

        self.writer_locked.store(true, Release);

        unsafe { self.lock.unlock() };

        true
    }

    pub unsafe fn write_unlock(&self) {
        self.lock.lock();

        self.writer_locked.store(false, Release);

        if self.write_waiter_count.load(Acquire) > 0 {
            self.writer_waiting.signal_one();
        } else if self.read_waiter_count.load(Acquire) > 0 {
            self.reader_waiting.signal_all();
        }

        unsafe { self.lock.unlock() };
    }
}

impl Default for RwLock {
    fn default() -> Self {
        Self::new()
    }
}
