use crate::diag::assert;
use crate::result::*;
use crate::util::PointerAndSize;
use crate::sync;
use core::ptr;

extern crate alloc;
use alloc::alloc::GlobalAlloc;
pub use alloc::alloc::Layout;

pub const PAGE_ALIGNMENT: usize = 0x1000;

pub mod rc;

// TODO: be able to change the global allocator?

pub trait Allocator {
    fn allocate(&mut self, layout: Layout) -> Result<*mut u8>;
    fn release(&mut self, addr: *mut u8, layout: Layout);

    fn new<T>(&mut self) -> Result<*mut T> {
        let layout = Layout::new::<T>();
        self.allocate(layout).map(|ptr| ptr as *mut T)
    }

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
            Err(_) => Err(rc::ResultOutOfMemory::make())
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

pub fn initialize(heap: PointerAndSize) {
    unsafe {
        G_ALLOCATOR_HOLDER.get().init(heap.address as usize, heap.size);
        G_ALLOCATOR_ENABLED = true;
    }
}

pub(crate) fn set_enabled(enabled: bool) {
    unsafe {
        G_ALLOCATOR_ENABLED = enabled;
    }
}

pub fn is_enabled() -> bool {
    unsafe {
        G_ALLOCATOR_ENABLED
    }
}

pub fn allocate(align: usize, size: usize) -> Result<*mut u8> {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        G_ALLOCATOR_HOLDER.get().allocate(layout)
    }
}

pub fn release(addr: *mut u8, align: usize, size: usize) {
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        G_ALLOCATOR_HOLDER.get().release(addr, layout);
    }
}

pub fn new<T>() -> Result<*mut T> {
    unsafe {
        G_ALLOCATOR_HOLDER.get().new::<T>()
    }
}

pub fn delete<T>(t: *mut T) {
    unsafe {
        G_ALLOCATOR_HOLDER.get().delete(t);
    }
}

pub struct Buffer<T> {
    pub ptr: *mut T,
    pub layout: Layout
}

impl<T> Buffer<T> {
    pub const fn empty() -> Self {
        Self {
            ptr: ptr::null_mut(),
            layout: Layout::new::<u8>() // Dummy value
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.ptr.is_null()
    }

    pub fn new(align: usize, size: usize) -> Result<Self> {
        let ptr = allocate(align, size)? as *mut T;
        Ok(Self {
            ptr,
            layout: unsafe {
                Layout::from_size_align_unchecked(size, align)
            }
        })
    }

    pub fn new_alloc<A: Allocator>(align: usize, size: usize, allocator: &mut A) -> Result<Self> {
        let layout = unsafe {
            Layout::from_size_align_unchecked(size, align)
        };
        let ptr = allocator.allocate(layout)? as *mut T;

        Ok(Self {
            ptr,
            layout
        })
    }

    pub fn release(&self) {
        release(self.ptr as *mut u8, self.layout.align(), self.layout.size());
    }
}

#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    // Disable memory allocation, this will avoid assertion levels which would need to allocate memory
    set_enabled(false);

    // Using SvcBreak by default since this is the safest level that can be used by any context, regardless of available mem/etc.
    // TODO: default aborting system to invoke here?
    assert::assert(assert::AssertLevel::SvcBreak(), rc::ResultOutOfMemory::make());

    // This should never be reached (TODO: better way to handle this really-unlikely situation than an infinite loop?)
    loop {}
}