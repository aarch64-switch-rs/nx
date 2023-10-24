//! Allocator implementation and definitions

use crate::result::*;
use crate::util::PointerAndSize;
use crate::sync;
use core::ptr;
use core::mem;

extern crate alloc;
use alloc::alloc::GlobalAlloc;
pub use alloc::alloc::Layout;

pub const PAGE_ALIGNMENT: usize = 0x1000;

pub mod rc;

// TODO: be able to change the global allocator?

/// Represents a heap allocator for this library
pub trait Allocator {
    /// Allocates memory
    /// 
    /// # Arguments
    /// 
    /// * `layout`: The memory layout
    fn allocate(&mut self, layout: Layout) -> Result<*mut u8>;

    /// Releases memory
    /// 
    /// # Arguments
    /// 
    /// * `addr`: The memory address
    /// * `layout`: The memory layout
    fn release(&mut self, addr: *mut u8, layout: Layout);

    /// Allocates a new heap value
    fn new<T>(&mut self) -> Result<*mut T> {
        let layout = Layout::new::<T>();
        self.allocate(layout).map(|ptr| ptr as *mut T)
    }

    /// Releases a heap value
    /// 
    /// The value must have been created using [`Allocator::new`]
    /// 
    /// # Arguments
    /// 
    /// * `t`: Heap value address
    fn delete<T>(&mut self, t: *mut T) {
        let layout = Layout::new::<T>();
        self.release(t as *mut u8, layout);
    }
}

extern crate linked_list_allocator;
use linked_list_allocator::Heap as LinkedListAllocator;

impl Allocator for LinkedListAllocator {
    fn allocate(&mut self, layout: Layout) -> Result<*mut u8> {
        match self.allocate_first_fit(layout) {
            Ok(non_null_addr) => Ok(non_null_addr.as_ptr()),
            Err(_) => rc::ResultOutOfMemory::make_err()
        }
    }

    fn release(&mut self, addr: *mut u8, layout: Layout) {
        if !addr.is_null() {
            unsafe {
                self.deallocate(ptr::NonNull::new_unchecked(addr), layout);
            }
        }
    }
}

unsafe impl<A: Allocator> GlobalAlloc for sync::Locked<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.get().allocate(layout).unwrap()
    }

    unsafe fn dealloc(&self, addr: *mut u8, layout: Layout) {
        self.get().release(addr, layout);
    }
}

#[global_allocator]
static mut G_ALLOCATOR_HOLDER: sync::Locked<LinkedListAllocator> = sync::Locked::new(false, LinkedListAllocator::empty());
static mut G_ALLOCATOR_ENABLED: bool = false;

/// Initializes the global allocator with the given address and size
/// 
/// # Arguments
/// 
/// * `heap`: The heap address and size
pub fn initialize(heap: PointerAndSize) {
    unsafe {
        G_ALLOCATOR_HOLDER.get().init(heap.address, heap.size);
        G_ALLOCATOR_ENABLED = true;
    }
}

/*
pub(crate) fn set_enabled(enabled: bool) {
    unsafe {
        G_ALLOCATOR_ENABLED = enabled;
    }
}
*/

/// Gets whether heap allocations are enabled
/// 
/// The library may internally disable them in exceptional cases: for instance, to avoid exception handlers to allocate heap memory if the exception cause is actually an OOM situation
pub fn is_enabled() -> bool {
    unsafe {
        G_ALLOCATOR_ENABLED
    }
}

/// Allocates heap memory using the global allocator
/// 
/// # Arguments
/// 
/// * `align`: The memory alignment
/// * `size`: The memory size
pub fn allocate(align: usize, size: usize) -> Result<*mut u8> {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        G_ALLOCATOR_HOLDER.get().allocate(layout)
    }
}

/// Releases allocated memory
/// 
/// # Arguments
/// 
/// * `addr`: The memory address
/// * `align`: The memory alignment
/// * `size`: The memory size
pub fn release(addr: *mut u8, align: usize, size: usize) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        G_ALLOCATOR_HOLDER.get().release(addr, layout);
    }
}

/// Creates a new heap value using the global allocator
pub fn new<T>() -> Result<*mut T> {
    unsafe {
        G_ALLOCATOR_HOLDER.get().new::<T>()
    }
}

/// Deletes a heap value
/// 
/// The value must have been created using the global allocator
/// 
/// # Arguments
/// 
/// * `t`: The heap value
pub fn delete<T>(t: *mut T) {
    unsafe {
        G_ALLOCATOR_HOLDER.get().delete(t);
    }
}

/// Represents a wrapped and manually managed heap value
/// 
/// Note that a [`Buffer`] is able to hold both a single value or an array of values of the provided type
pub struct Buffer<T> {
    /// The actual heap value
    pub ptr: *mut T,
    /// The memory's layout
    pub layout: Layout
}

impl<T> Buffer<T> {
    /// Creates a new, invalid [`Buffer`]
    #[inline]
    pub const fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            layout: Layout::new::<u8>() // Dummy value
        }
    }

    /// Gets whether this [`Buffer`] is valid
    #[inline]
    pub fn is_valid(&self) -> bool {
        !self.ptr.is_null()
    }

    /// Creates a new [`Buffer`] using the global allocator
    /// 
    /// # Arguments
    /// 
    /// * `align`: The align to use
    /// * `count`: The count of values to allocate
    pub fn new(align: usize, count: usize) -> Result<Self> {
        let size = count * mem::size_of::<T>();
        let ptr = allocate(align, size)? as *mut T;
        Ok(Self {
            ptr,
            layout: unsafe {
                Layout::from_size_align_unchecked(size, align)
            }
        })
    }

    /// Creates a new [`Buffer`] using a given allocator
    /// 
    /// # Arguments
    /// 
    /// * `align`: The align to use
    /// * `count`: The count of values to allocate
    /// * `allocator`: The allocator
    pub fn new_alloc<A: Allocator>(align: usize, count: usize, allocator: &mut A) -> Result<Self> {
        let size = count * mem::size_of::<T>();
        let layout = unsafe {
            Layout::from_size_align_unchecked(size, align)
        };
        let ptr = allocator.allocate(layout)? as *mut T;

        Ok(Self {
            ptr,
            layout
        })
    }

    /// Releases the [`Buffer`]
    /// 
    /// The [`Buffer`] becomes invalid after this
    pub fn release(&mut self) {
        release(self.ptr as *mut u8, self.layout.align(), self.layout.size());
        *self = Self::empty();
    }
}