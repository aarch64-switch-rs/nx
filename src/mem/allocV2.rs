use crate::result::*;
use crate::util::PointerAndSize;
use crate::sync;
use core::ptr;
use core::mem;

extern crate alloc;
use alloc::alloc::GlobalAlloc;
use alloc::alloc::Layout;

pub const PAGE_ALIGNMENT: usize = 0x1000;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct MemoryRegion {
    pub address: usize,
    pub size: usize
}

impl MemoryRegion {
    pub const fn empty() -> Self {
        Self { address: 0, size: 0 }
    }
    
    pub const fn from_addr(address: usize, size: usize) -> Self {
        Self { address: address, size: size }
    }
    
    pub fn from_ptr(address: *mut u8, size: usize) -> Self {
        Self { address: address as usize, size: size }
    }

    pub const fn is_valid(&self) -> bool {
        (self.address != 0) && (self.size != 0)
    }

    pub const fn get_start(&self) -> usize {
        self.address
    }

    pub const fn get_end(&self) -> usize {
        self.get_start() + self.size
    }
    
    pub fn get_ptr(&self) -> *mut u8 {
        self.address as *mut u8
    }

    pub const fn is_address_in(&self, addr: usize) -> bool {
        (self.get_start() <= addr) && (addr < self.get_end())
    }

    pub const fn is_in_region(&self, region: MemoryRegion) -> bool {
        ((region.get_start() <= self.get_start()) && (self.get_end() < region.get_end())) || self.is_address_in(region.get_start())
    }
    
    pub fn overlaps_with(&self, region: MemoryRegion) -> bool {
        for i in 0..self.size {
            let addr = self.get_start() + i;
            if region.is_address_in(addr) {
                return true;
            }
        }
        
        return false;
    }
}

pub trait Allocator {
    fn allocate(&self, align: usize, size: usize) -> Result<*mut u8>;
    fn free(&self, addr: *mut u8);

    fn new<T: Sized>(&self) -> Result<*mut T> {
        self.allocate(mem::align_of::<T>(), mem::size_of::<T>()).map(|ptr| ptr as *mut T)
    }

    fn delete<T: Sized>(&self, t: *mut T) {
        self.free(t as *mut u8);
    }
}

pub struct PoolAllocator {
    region: MemoryRegion
}

const fn compute_padding(base_addr: usize, align: usize) -> usize {
    let multiplier = (base_addr / align) + 1;
    let aligned_addr = multiplier * align;
    aligned_addr - base_addr
}

const unsafe fn is_end_pool(pool: *mut MemoryRegion) -> bool {
    ((*pool).address == usize::MAX) && ((*pool).size == usize::MAX)
}

const fn create_end_pool() -> MemoryRegion {
    MemoryRegion::from_addr(usize::MAX, usize::MAX)
}

impl PoolAllocator {
    unsafe fn get_pool_table_start(&self) -> *mut MemoryRegion {
        self.region.get_ptr().offset((self.region.size - mem::size_of::<MemoryRegion>()) as isize) as *mut MemoryRegion
    }

    unsafe fn is_pool_valid(&self, pool: *mut MemoryRegion) -> bool {
        (*pool).is_valid() && (*pool).is_in_region(self.region)
    }

    unsafe fn find_empty_pool(&self) -> Option<*mut MemoryRegion> {
        let mut cur_pool = self.get_pool_table_start();
        loop {
            if !self.region.is_address_in(cur_pool as usize) {
                return None;
            }

            if !self.is_pool_valid(cur_pool) || is_end_pool(cur_pool) {
                return Some(cur_pool);
            }

            cur_pool = cur_pool.offset(-1);
        }
    }

    unsafe fn find_pool(&self, addr: usize) -> Option<*mut MemoryRegion> {
        let mut cur_pool = self.get_pool_table_start();
        loop {
            if !self.region.is_address_in(cur_pool as usize) {
                return None;
            }
            if is_end_pool(cur_pool) {
                return None;
            }
            
            if self.is_pool_valid(cur_pool) && (*cur_pool).is_address_in(addr) {
                return Some(cur_pool);
            }

            cur_pool = cur_pool.offset(-1);
        }
    }

    unsafe fn is_region_allocated(&self, addr: usize, size: usize) -> Option<*mut MemoryRegion> {
        let mut cur_pool = self.get_pool_table_start();
        let check_region = MemoryRegion::from_addr(addr, size);
        loop {
            if !self.region.is_address_in(cur_pool as usize) {
                return None;
            }
            if is_end_pool(cur_pool) {
                return None;
            }

            // log_str("Alokt");
            
            if self.is_pool_valid(cur_pool) && (*cur_pool).overlaps_with(check_region) {
                return Some(cur_pool);
            }

            cur_pool = cur_pool.offset(-1);
        }
    }

    unsafe fn is_pool_in_region(&self, addr: usize, size: usize, pool: *mut MemoryRegion) -> bool {
        let search_region = MemoryRegion::from_addr(addr, size);
        let pool_region = MemoryRegion::from_ptr(pool as *mut u8, mem::size_of::<MemoryRegion>());

        pool_region.overlaps_with(search_region)
    }
    
    unsafe fn is_any_pool_in_region(&self, addr: usize, size: usize) -> bool {
        let mut cur_pool = self.get_pool_table_start();
        loop {
            if !self.region.is_address_in(cur_pool as usize) {
                return false;
            }

            if is_end_pool(cur_pool) {
                return self.is_pool_in_region(addr, size, cur_pool);
            }
            else if self.is_pool_valid(cur_pool) && self.is_pool_in_region(addr, size, cur_pool) {
                return true;
            }

            cur_pool = cur_pool.offset(-1);
        }
    }

    pub const fn empty() -> Self {
        Self {
            region: MemoryRegion::empty()
        }
    }

    pub fn new(region: MemoryRegion) -> Self {
        unsafe {
            ptr::write_bytes(region.get_ptr(), 0, region.size);
        }

        let allocator = Self {
            region: region
        };

        unsafe {
            let start_pool = allocator.get_pool_table_start();
            *start_pool = create_end_pool();
        }

        allocator
    }

    pub fn is_valid(&self) -> bool {
        self.region.is_valid()
    }

    pub fn is_allocated(&self, addr: *mut u8) -> bool {
        unsafe {
            self.find_pool(addr as usize).is_some()
        }
    }
}

impl Allocator for PoolAllocator {
    fn allocate(&self, align: usize, size: usize) -> Result<*mut u8> {
        result_return_unless!(self.is_valid(), 0xBABA);
        result_return_unless!(size > 0, 0xBEBE);

        let mut cur_addr = self.region.get_start();

        unsafe {
            if let Some(empty_pool) = self.find_empty_pool() {
                loop {
                    if let Some(alloc_pool) = self.is_region_allocated(cur_addr, size) {
                        // Already allocated, continue search
                        cur_addr = (*alloc_pool).get_end();
                    }
                    else {
                        if (cur_addr % align) != 0 {
                            // Address needs to be aligned
                            let pad = compute_padding(cur_addr, align);
                            cur_addr += pad;
                        }
    
                        let cur_addr_end = cur_addr + size;
                        result_return_unless!(cur_addr_end < self.region.get_end(), 0xB0B1);
                        result_return_unless!(!self.is_any_pool_in_region(cur_addr, size), 0xB0B2);
                        result_return_unless!(!self.is_pool_in_region(cur_addr, size, empty_pool), 0xB0B3);

                        if is_end_pool(empty_pool) {
                            let next_pool = empty_pool.offset(-1);
                            if self.region.is_address_in(next_pool as usize) {
                                *next_pool = create_end_pool();
                            }
                        }
                        (*empty_pool).address = cur_addr;
                        (*empty_pool).size = size;

                        return Ok(cur_addr as *mut u8);
                    }
                }
            }
            else {
                return Err(ResultCode::new(0xBFBF));
            }
        }
    }

    fn free(&self, addr: *mut u8) {
        if self.is_valid() {
            unsafe {
                if let Some(alloc_pool) = self.find_pool(addr as usize) {
                    if is_end_pool(alloc_pool) {
                        let prev_pool = alloc_pool.offset(1);
                        if self.region.is_address_in(prev_pool as usize) && !(*prev_pool).is_valid() {
                            *prev_pool = create_end_pool();
                            *alloc_pool = MemoryRegion::empty();
                        }
                        else {
                            *alloc_pool = create_end_pool();
                        }
                    }
                    else {
                        let next_pool = alloc_pool.offset(-1);
                        if self.region.is_address_in(next_pool as usize) && is_end_pool(next_pool) {
                            *next_pool = MemoryRegion::empty();
                            *alloc_pool = create_end_pool();
                        }
                        else {
                            *alloc_pool = MemoryRegion::empty();
                        }
                    }
                }
            }
        }
    }
}

// NOTE: there wasn't a better way to do this since sync::Mutex's (un)locking must be mutable but alloc's GlobalAlloc trait's fns are not mutable (thanks, Rust)
// TODO: be able to change global allocator?

static mut G_ALLOCATOR_LOCK: sync::Mutex = sync::Mutex::new(false);

pub struct GlobalAllocatorHolder<A: Allocator> {
    pub allocator: A
}

impl<A: Allocator> GlobalAllocatorHolder<A> {
    pub const fn new(allocator: A) -> Self {
        Self { allocator: allocator }
    }
}

unsafe impl<A: Allocator> GlobalAlloc for GlobalAllocatorHolder<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let _ = sync::ScopedLock::new(&mut G_ALLOCATOR_LOCK);
        self.allocator.allocate(layout.align(), layout.size()).unwrap()
    }

    unsafe fn dealloc(&self, addr: *mut u8, _layout: Layout) {
        let _ = sync::ScopedLock::new(&mut G_ALLOCATOR_LOCK);
        self.allocator.free(addr)
    }
}

#[global_allocator]
static mut G_ALLOCATOR_HOLDER: GlobalAllocatorHolder<PoolAllocator> = GlobalAllocatorHolder::new(PoolAllocator::empty());

pub fn initialize(heap: PointerAndSize) {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_ALLOCATOR_LOCK);
        G_ALLOCATOR_HOLDER = GlobalAllocatorHolder::new(PoolAllocator::new(MemoryRegion::from_ptr(heap.address, heap.size)));
    }
}

pub fn allocate(align: usize, size: usize) -> Result<*mut u8> {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_ALLOCATOR_LOCK);
        G_ALLOCATOR_HOLDER.allocator.allocate(align, size)
    }
}

pub fn free(addr: *mut u8) {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_ALLOCATOR_LOCK);
        G_ALLOCATOR_HOLDER.allocator.free(addr)
    }
}

pub fn new<T: Sized>() -> Result<*mut T> {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_ALLOCATOR_LOCK);
        G_ALLOCATOR_HOLDER.allocator.new::<T>()
    }
}

pub fn delete<T: Sized>(t: *mut T) {
    unsafe {
        let _ = sync::ScopedLock::new(&mut G_ALLOCATOR_LOCK);
        G_ALLOCATOR_HOLDER.allocator.delete(t);
    }
}

use crate::svc;

#[alloc_error_handler]
fn alloc_error_handler(_layout: core::alloc::Layout) -> ! {
    // TODO: maybe abort using break? that sounds feasible since no allocations will be made there...
    let mut rc: u32 = 0xBEEF;
    svc::break_(svc::BreakReason::Panic, &mut rc as *mut _ as *mut u8, core::mem::size_of::<u32>());
    loop {}
}