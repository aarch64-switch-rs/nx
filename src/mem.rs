//! Memory (heap) support and utils

extern crate alloc as core_alloc;
use core_alloc::boxed::Box;
use core::ops;
use core::ptr;
use core::mem;
use core::marker;

use crate::util;

pub mod alloc;

#[derive(Copy, Clone)]
struct ReferenceCount {
    holder: *mut u64
}

impl ReferenceCount {
    #[inline]
    pub const fn new() -> Self {
        Self { holder: ptr::null_mut() }
    }
    
    #[inline]
    pub fn use_count(&self) -> u64 {
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
                    self.holder = alloc::new::<u64>().unwrap();
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

/// Represents a shared object, similar to C++'s `std::shared_ptr`
pub struct Shared<T: ?Sized> {
    object: *mut T,
    ref_count: ReferenceCount
}

impl<T> Shared<T> {
    /// Creates a new [`Shared`] from the given variable
    /// 
    /// # Argument
    /// 
    /// * `var`: Variable to box into a [`Shared`]
    pub fn new(var: T) -> Self {
        // This is done instead of just &var to avoid dropping the variable inside this function
        let object = Box::into_raw(Box::new(var));
        let mut shared = Self { object, ref_count: ReferenceCount::new() };
        shared.ref_count.acquire(object);
        shared
    }
}

impl<T: ?Sized> Shared<T> {
    fn release(&mut self) {
        self.ref_count.release(self.object);
    }
    
    fn acquire(&mut self, object: *mut T) {
        self.ref_count.acquire(object);
        self.object = object;
    }

    /// Returns the number of existing [`Shared`] instances pointing to this instance's variable
    #[inline]
    pub fn use_count(&self) -> u64 {
        self.ref_count.use_count()
    }

    /// Performs a potentiallt unsafe conversion to a different [`Shared`]
    /// 
    /// Note that this is used in very, very limited cases over the library where it's tested to work as expected, and probably shouldn't be used otherwise
    pub unsafe fn to<U: ?Sized>(&self) -> Shared<U> {
        let mut new_shared = Shared::<U> { object: util::raw_transmute(self.object), ref_count: self.ref_count };
        new_shared.acquire(new_shared.object);
        new_shared
    }
    
    /// Accesses the value inside the [`Shared`] object
    /// 
    /// Note that the value is guaranteed to be valid since the [`Shared`] object must be created with a valid value
    #[inline]
    pub fn get(&self) -> &mut T {
        unsafe { &mut *self.object }
    }

    // TODO: rename get() to get_mut() and make a get() fn returning a &T ref?
}

impl<T: marker::Unsize<U> + ?Sized, U: ?Sized> ops::CoerceUnsized<Shared<U>> for Shared<T> {}

impl<T: ?Sized> Drop for Shared<T> {
    /// Drops this [`Shared`] instance
    /// 
    /// This won't drop the inner variable unless this is the last existing instance pointing to it
    fn drop(&mut self) {
        self.release();
    }
}

impl<T: ?Sized> Clone for Shared<T> {
    /// Creates a new [`Shared`] instance pointing to the same variable
    fn clone(&self) -> Self {
        let mut new_shared = Self { object: self.object, ref_count: self.ref_count };
        new_shared.acquire(new_shared.object);
        new_shared
    }
}

impl<T: ?Sized> PartialEq for Shared<T> {
    /// Gets whether both [`Shared`] instances point to the same variable
    fn eq(&self, other: &Self) -> bool {
        ptr::addr_eq(self.object, other.object)
    }
}

impl<T: ?Sized> Eq for Shared<T> {}

/// Flushes data cache at a certain memory region
/// 
/// # Arguments
/// 
/// * `address`: Memory region address
/// * `size`: Memory region size
#[inline(always)]
pub fn flush_data_cache(address: *mut u8, size: usize) {
    extern "C" {
        fn __nx_mem_flush_data_cache(address: *mut u8, size: usize);
    }

    unsafe {
        __nx_mem_flush_data_cache(address, size);
    }
}

/// Aligns up a given value with respect to a certain alignment
/// 
/// # Arguments
/// 
/// * `value`: Value to align
/// * `align`: Alignment
#[inline]
pub const fn align_up(value: usize, align: usize) -> usize {
    let inv_mask = align - 1;
    (value + inv_mask) & !inv_mask
}

/// Aligns down a given value with respect to a certain alignment
/// 
/// # Arguments
/// 
/// * `value`: Value to align
/// * `align`: Alignment
#[inline]
pub const fn align_down(value: usize, align: usize) -> usize {
    let inv_mask = align - 1;
    value & !inv_mask
}