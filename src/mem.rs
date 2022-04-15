extern crate alloc as core_alloc;
use core_alloc::boxed::Box;
use core::ops;
use core::ptr;
use core::mem;
use core::marker;

use crate::util;

pub mod alloc;

#[derive(Copy, Clone)]
struct Refcount {
    holder: *mut i64
}

impl Refcount {
    pub const fn new() -> Self {
        Self { holder: ptr::null_mut() }
    }
    
    pub fn use_count(&self) -> i64 {
        if self.holder.is_null() {
            0
        }
        else {
            unsafe { *self.holder }
        }
    }
    
    pub fn acquire<U: ?Sized>(&mut self, ptr: *mut U) {
        if !ptr.is_null() {
            unsafe {
                if self.holder.is_null() {
                    self.holder = alloc::new::<i64>().unwrap();
                    *self.holder = 1;
                }
                else {
                    *self.holder += 1;
                }
            }
        }
    }
    
    pub fn release<U: ?Sized>(&mut self, ptr: *mut U) {
        if !self.holder.is_null() {
            unsafe {
                *self.holder -= 1;
                if *self.holder == 0 {
                    // We created the variable as a Box, so we destroy it the same way
                    mem::drop(Box::from_raw(ptr));
                    alloc::delete(self.holder);
                    self.holder = ptr::null_mut();
                }
            }
        }
    }
}

pub struct Shared<T: ?Sized> {
    object: *mut T,
    refcount: Refcount
}

impl<T> Shared<T> {
    pub fn new(var: T) -> Self {
        // This is done instead of just &var to avoid dropping the variable inside this function
        let object = Box::into_raw(Box::new(var));
        let mut shared = Self { object, refcount: Refcount::new() };
        shared.refcount.acquire(object);
        shared
    }

    // TODO: custom allocator support?

    pub const fn empty() -> Self {
        Self { object: ptr::null_mut(), refcount: Refcount::new() }
    }
}

impl<T: ?Sized> Shared<T> {
    fn release(&mut self) {
        self.refcount.release(self.object);
    }
    
    fn acquire(&mut self, object: *mut T) {
        self.refcount.acquire(object);
        self.object = object;
    }

    pub fn is_null(&self) -> bool {
        self.object.is_null()
    }

    pub fn is_valid(&self) -> bool {
        !self.is_null()
    }

    pub fn use_count(&self) -> i64 {
        self.refcount.use_count()
    }

    pub fn to<U: ?Sized>(&self) -> Shared<U> {
        let mut new_shared = Shared::<U> { object: util::raw_transmute(self.object), refcount: self.refcount };
        new_shared.acquire(new_shared.object);
        new_shared
    }
    
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.object }
    }

    pub fn reset(&mut self) {
        self.release();
    }

    pub fn copy(&self) -> Self {
        let mut new_shared = Self { object: self.object, refcount: self.refcount };
        new_shared.acquire(new_shared.object);
        new_shared
    }
}

impl<T: marker::Unsize<U> + ?Sized, U: ?Sized> ops::CoerceUnsized<Shared<U>> for Shared<T> {}

impl<T: ?Sized> Drop for Shared<T> {
    fn drop(&mut self) {
        self.release();
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    fn clone(&self) -> Self {
        self.copy()
    }
}

impl<T> ops::Deref for Shared<T> {
    type Target = T;
    
    fn deref(&self) -> &T {
        unsafe { &*self.object }
    }
}

impl<T> ops::DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.object }
    }
}

impl<T: ?Sized> PartialEq for Shared<T> {
    fn eq(&self, other: &Self) -> bool {
        self.object == other.object
    }
}

impl<T: ?Sized> Eq for Shared<T> {}

#[inline(always)]
pub fn flush_data_cache(address: *mut u8, size: usize) {
    extern "C" {
        fn __nx_mem_flush_data_cache(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_mem_flush_data_cache(address, size);
    }
}

pub const fn align_up(value: usize, align: usize) -> usize {
    let inv_mask = align - 1;
    (value + inv_mask) & !inv_mask
}

pub const fn align_down(value: usize, align: usize) -> usize {
    let inv_mask = align - 1;
    value & !inv_mask
}