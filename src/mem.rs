//! Memory (heap) support and utils

extern crate alloc as core_alloc;
use ::alloc::alloc::handle_alloc_error;
use ::alloc::alloc::Global;
use ::alloc::borrow::Cow;
use ::alloc::borrow::ToOwned;
use ::alloc::string::String;
use ::alloc::vec::Vec;
use core_alloc::boxed::Box;
use core::alloc::AllocError;
use core::alloc::Allocator;
use core::alloc::Layout;
use core::any::Any;
use core::cell::UnsafeCell;
use core::fmt;
use core::hash::{Hash, Hasher};
use core::iter;
use core::marker::PhantomData;
use core::marker::Unsize;
use core::mem::ManuallyDrop;
use core::ops::CoerceUnsized;
use core::ptr;
use core::mem;

use core::ptr::NonNull;
use core::sync::atomic;
use core::sync::atomic:: Ordering::*;
use core::u32;

use crate::acquire;
use crate::diag::abort::abort;
use crate::diag::abort::AbortLevel;
use crate::result::ResultBase;
//use crate::result::{Result, ResultBase};
use crate::svc;
use crate::sync::sys::rwlock::RwLock as RawRwLock;
use crate::util;

pub mod alloc;

/// Flushes data cache at a certain memory region
/// 
/// # Arguments
/// 
/// * `address`: Memory region address
/// * `size`: Memory region size
/// # SAFETY: Null pointers are OK as we are just doing cache invalidation, not accessing the pointer.
#[inline(always)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
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

/// Blocks thread until the memory region specified has the permission passed
/// 
/// # Arguments
/// 
/// * `address`: The address to query for memory permissions
/// * `permissions`: The memory permission to wait on
/// 
/// Note that if multiple permissions are specified (e.g. `MemoryPermission::Read | MemoryPermission::Write`), the function will return if *any* specified permission is present.
#[inline(always)]
pub fn wait_for_permission(address: svc::Address, permission: svc::MemoryPermission, timeout: Option<usize>) -> crate::result::Result<()> {
    let mut iteration: usize = 0;
    loop {
        let (memory, _) = svc::query_memory(address)?;
        if memory.permission.contains(permission) {
            return Ok(());
        }
        if let Some(timeout) = timeout && timeout <= (100_000 * iteration) {
            // The timeout has been set and has already expired
            return Err(svc::rc::ResultTimedOut::make());
        }
        iteration += 1;
        let _ = crate::thread::sleep(100_000);
    }
}

/////// Make some Sharedss
/// 


/// A soft limit on the amount of references that may be made to an `Arc`.
///
/// Going above this limit will abort your program (although not
/// necessarily) at _exactly_ `MAX_REFCOUNT + 1` references.
/// Trying to go above it might call a `panic` (if not actually going above it).
///
/// This is a global invariant, and also applies when using a compare-exchange loop.
///
/// See comment in `Arc::clone`.
const MAX_REFCOUNT: usize = (isize::MAX>>1) as usize;

/// The error in case either counter reaches above `MAX_REFCOUNT`, and we can `panic` safely.
const INTERNAL_REFCOUNT_OVERFLOW_ERROR: &str = "Shared ref counter overflow";
const INTERNAL_READCOUNT_OVERFLOW_ERROR: &str = "Shared reader counter overflow";



/// A thread-safe reference-counting pointer. 'Arc' stands for 'Atomically
/// Reference Counted'.
///
/// The type `Arc<T>` provides shared ownership of a value of type `T`,
/// allocated in the heap. Invoking [`clone`][clone] on `Arc` produces
/// a new `Arc` instance, which points to the same allocation on the heap as the
/// source `Arc`, while increasing a reference count. When the last `Arc`
/// pointer to a given allocation is destroyed, the value stored in that allocation (often
/// referred to as "inner value") is also dropped.
///
/// Shared references in Rust disallow mutation by default, and `Arc` is no
/// exception: you cannot generally obtain a mutable reference to something
/// inside an `Arc`. If you need to mutate through an `Arc`, use
/// [`Mutex`][mutex], [`RwLock`][rwlock], or one of the [`Atomic`][atomic]
/// types.
///
/// **Note**: This type is only available on platforms that support atomic
/// loads and stores of pointers, which includes all platforms that support
/// the `std` crate but not all those which only support [`alloc`](crate).
/// This may be detected at compile time using `#[cfg(target_has_atomic = "ptr")]`.
///
/// ## Thread Safety
///
/// Unlike [`Rc<T>`], `Arc<T>` uses atomic operations for its reference
/// counting. This means that it is thread-safe. The disadvantage is that
/// atomic operations are more expensive than ordinary memory accesses. If you
/// are not sharing reference-counted allocations between threads, consider using
/// [`Rc<T>`] for lower overhead. [`Rc<T>`] is a safe default, because the
/// compiler will catch any attempt to send an [`Rc<T>`] between threads.
/// However, a library might choose `Arc<T>` in order to give library consumers
/// more flexibility.
///
/// `Arc<T>` will implement [`Send`] and [`Sync`] as long as the `T` implements
/// [`Send`] and [`Sync`]. Why can't you put a non-thread-safe type `T` in an
/// `Arc<T>` to make it thread-safe? This may be a bit counter-intuitive at
/// first: after all, isn't the point of `Arc<T>` thread safety? The key is
/// this: `Arc<T>` makes it thread safe to have multiple ownership of the same
/// data, but it  doesn't add thread safety to its data. Consider
/// <code>Arc<[RefCell\<T>]></code>. [`RefCell<T>`] isn't [`Sync`], and if `Arc<T>` was always
/// [`Send`], <code>Arc<[RefCell\<T>]></code> would be as well. But then we'd have a problem:
/// [`RefCell<T>`] is not thread safe; it keeps track of the borrowing count using
/// non-atomic operations.
///
/// In the end, this means that you may need to pair `Arc<T>` with some sort of
/// [`std::sync`] type, usually [`Mutex<T>`][mutex].
///
/// ## Breaking cycles with `Weak`
///
/// The [`downgrade`][downgrade] method can be used to create a non-owning
/// [`Weak`] pointer. A [`Weak`] pointer can be [`upgrade`][upgrade]d
/// to an `Arc`, but this will return [`None`] if the value stored in the allocation has
/// already been dropped. In other words, `Weak` pointers do not keep the value
/// inside the allocation alive; however, they *do* keep the allocation
/// (the backing store for the value) alive.
///
/// A cycle between `Arc` pointers will never be deallocated. For this reason,
/// [`Weak`] is used to break cycles. For example, a tree could have
/// strong `Arc` pointers from parent nodes to children, and [`Weak`]
/// pointers from children back to their parents.
///
/// # Cloning references
///
/// Creating a new reference from an existing reference-counted pointer is done using the
/// `Clone` trait implemented for [`Arc<T>`][Arc] and [`Weak<T>`][Weak].
///
/// ```
/// use std::sync::Arc;
/// let foo = Arc::new(vec![1.0, 2.0, 3.0]);
/// // The two syntaxes below are equivalent.
/// let a = foo.clone();
/// let b = Arc::clone(&foo);
/// // a, b, and foo are all Arcs that point to the same memory location
/// ```
///
/// ## `Deref` behavior
///
/// `Arc<T>` automatically dereferences to `T` (via the [`Deref`] trait),
/// so you can call `T`'s methods on a value of type `Arc<T>`. To avoid name
/// clashes with `T`'s methods, the methods of `Arc<T>` itself are associated
/// functions, called using [fully qualified syntax]:
///
/// ```
/// use std::sync::Arc;
///
/// let my_arc = Arc::new(());
/// let my_weak = Arc::downgrade(&my_arc);
/// ```
///
/// `Arc<T>`'s implementations of traits like `Clone` may also be called using
/// fully qualified syntax. Some people prefer to use fully qualified syntax,
/// while others prefer using method-call syntax.
///
/// ```
/// use std::sync::Arc;
///
/// let arc = Arc::new(());
/// // Method-call syntax
/// let arc2 = arc.clone();
/// // Fully qualified syntax
/// let arc3 = Arc::clone(&arc);
/// ```
///
/// [`Weak<T>`][Weak] does not auto-dereference to `T`, because the inner value may have
/// already been dropped.
///
/// [`Rc<T>`]: crate::rc::Rc
/// [clone]: Clone::clone
/// [mutex]: ../../std/sync/struct.Mutex.html
/// [rwlock]: ../../std/sync/struct.RwLock.html
/// [atomic]: core::sync::atomic
/// [downgrade]: Arc::downgrade
/// [upgrade]: Weak::upgrade
/// [RefCell\<T>]: core::cell::RefCell
/// [`RefCell<T>`]: core::cell::RefCell
/// [`std::sync`]: ../../std/sync/index.html
/// [`Arc::clone(&from)`]: Arc::clone
/// [fully qualified syntax]: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#fully-qualified-syntax-for-disambiguation-calling-methods-with-the-same-name
///
/// # Examples
///
/// Sharing some immutable data between threads:
///
/// ```
/// use std::sync::Arc;
/// use std::thread;
///
/// let five = Arc::new(5);
///
/// for _ in 0..10 {
///     let five = Arc::clone(&five);
///
///     thread::spawn(move || {
///         println!("{five:?}");
///     });
/// }
/// ```
///
/// Sharing a mutable [`AtomicUsize`]:
///
/// [`AtomicUsize`]: core::sync::atomic::AtomicUsize "sync::atomic::AtomicUsize"
///
/// ```
/// use std::sync::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use std::thread;
///
/// let val = Arc::new(AtomicUsize::new(5));
///
/// for _ in 0..10 {
///     let val = Arc::clone(&val);
///
///     thread::spawn(move || {
///         let v = val.fetch_add(1, Ordering::Relaxed);
///         println!("{v:?}");
///     });
/// }
/// ```
///
/// See the [`rc` documentation][rc_examples] for more examples of reference
/// counting in general.
///
/// [rc_examples]: crate::rc#examples

pub struct Shared<
    T: ?Sized,
    A: Allocator = Global,
> {
    ptr: NonNull<ArcRwLockInner<T>>,
    phantom: PhantomData<ArcRwLockInner<T>>,
    alloc: A,
}


unsafe impl<T: ?Sized + Sync + Send, A: Allocator + Send> Send for Shared<T, A> {}

unsafe impl<T: ?Sized + Sync + Send, A: Allocator + Sync> Sync for Shared<T, A> {}


//impl<T: RefUnwindSafe + ?Sized, A: Allocator + UnwindSafe> UnwindSafe for Shared<T, A> {}

impl<T: ?Sized + Unsize<U>, U: ?Sized, A: Allocator> CoerceUnsized<Shared<U, A>> for Shared<T, A> {}


pub struct SharedReadGuard<'borrow, T: ?Sized> {
    reader: &'borrow ArcRwLockInner<T>
}
impl<'borrow, T: ?Sized> core::ops::Deref for SharedReadGuard<'borrow, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {&*self.reader.data.get() }
    }
}

/*
impl<'borrow, T> core::ops::Deref for SharedReadGuard<'borrow, [T]> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        &self.reader.data
    }
}*/

pub struct SharedWriteGuard<'borrow, T: ?Sized> {
    writer: &'borrow ArcRwLockInner<T>
}

impl<'borrow, T: ?Sized> core::ops::Deref for SharedWriteGuard<'borrow, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe {&*self.writer.data.get() }
    }
}
impl<'borrow, T: ?Sized> core::ops::DerefMut for SharedWriteGuard<'borrow, T> {
        fn deref_mut(&mut self) -> &mut T {
            unsafe {&mut *self.writer.data.get() }
    }
}

impl<T: ?Sized> Shared<T> {
    unsafe fn from_inner(ptr: NonNull<ArcRwLockInner<T>>) -> Self {
        unsafe { Self::from_inner_in(ptr, Global) }
    }

    unsafe fn from_ptr(ptr: *mut ArcRwLockInner<T>) -> Self {
        unsafe { Self::from_ptr_in(ptr, Global) }
    }
}

impl<T: ?Sized, A: Allocator> Shared<T, A> {

    pub(crate) unsafe fn to<U: ?Sized>(self) ->  Shared<U, A> {
        let (inner, alloc) = Self::into_inner_with_allocator(self);
        let inner: *mut ArcRwLockInner<U> = util::raw_transmute(inner.as_ptr());
        Shared::from_inner_in(NonNull::new_unchecked(inner), alloc)
    }

    #[inline]
    pub fn get(&self) -> SharedWriteGuard<'_, T> {self.write()}

    pub fn read(&self) -> SharedReadGuard<'_, T> {
        
        self.inner().lock.read();

        SharedReadGuard { reader: self.inner() }
    }

    pub fn try_read(&self) -> Option<SharedReadGuard<'_, T>> {
        if self.inner().lock.try_read() {
            Some(SharedReadGuard { reader: self.inner() })
        } else {
            None
        }
    }

    pub fn write(&self) -> SharedWriteGuard<'_, T> {
        self.inner().lock.write();

        SharedWriteGuard { writer: unsafe { self.ptr.as_ref()} }
    }

    pub fn try_write(&self) -> Option<SharedWriteGuard<'_, T>> {
        if self.inner().lock.try_write() {
            Some(SharedWriteGuard { writer: unsafe { self.ptr.as_ref()} })
        } else {
            None
        }
    }

    #[inline]
    fn into_inner_with_allocator(this: Self) -> (NonNull<ArcRwLockInner<T>>, A) {
        let this = mem::ManuallyDrop::new(this);
        (this.ptr, unsafe { ptr::read(&this.alloc) })
    }

    #[inline]
    unsafe fn from_inner_in(ptr: NonNull<ArcRwLockInner<T>>, alloc: A) -> Self {
        Self { ptr, phantom: PhantomData, alloc }
    }

    #[inline]
    unsafe fn from_ptr_in(ptr: *mut ArcRwLockInner<T>, alloc: A) -> Self {
        unsafe { Self::from_inner_in(NonNull::new_unchecked(ptr), alloc) }
    }
}

// This is repr(C) to future-proof against possible field-reordering, which
// would interfere with otherwise safe [into|from]_raw() of transmutable
// inner types.
#[repr(C)]
struct ArcRwLockInner<T: ?Sized> {
    ref_count: atomic::AtomicUsize,
    lock: RawRwLock,
    pub(self) data: UnsafeCell<T>,
}

/// Calculate layout for `ArcRwLockInner<T>` using the inner value's layout
fn arcinner_layout_for_value_layout(layout: Layout) -> Layout {
    // Calculate layout using the given value layout.
    // Previously, layout was calculated on the expression
    // `&*(ptr as *const ArcRwLockInner<T>)`, but this created a misaligned
    // reference (see #54908).
    Layout::new::<ArcRwLockInner<()>>().extend(layout).unwrap().0.pad_to_align()
}

unsafe impl<T: ?Sized + Sync + Send> Send for ArcRwLockInner<T> {}
unsafe impl<T: ?Sized + Sync + Send> Sync for ArcRwLockInner<T> {}

impl<T> Shared<T> {
    /// Constructs a new `Arc<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    /// ```
    #[inline]
    pub fn new(data: T) -> Shared<T> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::new(ArcRwLockInner {
            ref_count: atomic::AtomicUsize::new(1),
            lock: RawRwLock::new(),
            data: UnsafeCell::new(data),
        });
        unsafe { Self::from_inner(Box::leak(x).into()) }
    }

    /// Constructs a new `Arc` with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let mut five = Arc::<u32>::new_uninit();
    ///
    /// // Deferred initialization:
    /// Arc::get_mut(&mut five).unwrap().write(5);
    ///
    /// let five = unsafe { five.assume_init() };
    ///
    /// assert_eq!(*five, 5)
    /// ```
    
    #[inline]
    #[must_use]
    pub fn new_uninit() -> Shared<mem::MaybeUninit<T>> {
        unsafe {
            Shared::from_ptr(Shared::allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate(layout),
                <*mut u8>::cast,
            ))
        }
    }

    /// Constructs a new `Arc` with uninitialized contents, with the memory
    /// being filled with `0` bytes.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and incorrect usage
    /// of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    ///
    /// use std::sync::Arc;
    ///
    /// let zero = Arc::<u32>::new_zeroed();
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero, 0)
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    
    #[inline]
    #[must_use]
    pub fn new_zeroed() -> Shared<mem::MaybeUninit<T>> {
        unsafe {
            Shared::from_ptr(Shared::allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate_zeroed(layout),
                <*mut u8>::cast,
            ))
        }
    }

    /// Constructs a new `Arc<T>`, returning an error if allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    /// use std::sync::Arc;
    ///
    /// let five = Arc::try_new(5)?;
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn try_new(data: T) -> Result<Shared<T>, AllocError> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x: Box<_> = Box::try_new(ArcRwLockInner {
            ref_count: atomic::AtomicUsize::new(1),
            lock: RawRwLock::new(),
            data: UnsafeCell::new(data),
        })?;
        unsafe { Ok(Self::from_inner(Box::leak(x).into())) }
    }

    /// Constructs a new `Arc` with uninitialized contents, returning an error
    /// if allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit, allocator_api)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let mut five = Arc::<u32>::try_new_uninit()?;
    ///
    /// // Deferred initialization:
    /// Arc::get_mut(&mut five).unwrap().write(5);
    ///
    /// let five = unsafe { five.assume_init() };
    ///
    /// assert_eq!(*five, 5);
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    pub fn try_new_uninit() -> Result<Shared<mem::MaybeUninit<T>>, AllocError> {
        unsafe {
            Ok(Shared::from_ptr(Shared::try_allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate(layout),
                <*mut u8>::cast,
            )?))
        }
    }

    /// Constructs a new `Arc` with uninitialized contents, with the memory
    /// being filled with `0` bytes, returning an error if allocation fails.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and incorrect usage
    /// of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit, allocator_api)]
    ///
    /// use std::sync::Arc;
    ///
    /// let zero = Arc::<u32>::try_new_zeroed()?;
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero, 0);
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    pub fn try_new_zeroed() -> Result<Shared<mem::MaybeUninit<T>>, AllocError> {
        unsafe {
            Ok(Shared::from_ptr(Shared::try_allocate_for_layout(
                Layout::new::<T>(),
                |layout| Global.allocate_zeroed(layout),
                <*mut u8>::cast,
            )?))
        }
    }
}

impl<T, A: Allocator> Shared<T, A> {
    /// Constructs a new `Arc<T>` in the provided allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let five = Arc::new_in(5, System);
    /// ```
    #[inline]
    
    
    pub fn new_in(data: T, alloc: A) -> Shared<T, A> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x = Box::new_in(
            ArcRwLockInner {
                ref_count: atomic::AtomicUsize::new(1),
                lock: RawRwLock::new(),
                data: UnsafeCell::new(data),
            },
            alloc,
        );
        let (ptr, alloc) = Box::into_raw_with_allocator(x);
        unsafe { Self::from_inner_in(NonNull::new_unchecked(ptr), alloc) }
    }

    /// Constructs a new `Arc` with uninitialized contents in the provided allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let mut five = Arc::<u32, _>::new_uninit_in(System);
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     Arc::get_mut_unchecked(&mut five).as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    ///
    /// assert_eq!(*five, 5)
    /// ```
    #[inline]
    pub fn new_uninit_in(alloc: A) -> Shared<mem::MaybeUninit<T>, A> {
        unsafe {
            Shared::from_ptr_in(
                Shared::allocate_for_layout(
                    Layout::new::<T>(),
                    |layout| alloc.allocate(layout),
                    <*mut u8>::cast,
                ),
                alloc,
            )
        }
    }

    /// Constructs a new `Arc` with uninitialized contents, with the memory
    /// being filled with `0` bytes, in the provided allocator.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and incorrect usage
    /// of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let zero = Arc::<u32, _>::new_zeroed_in(System);
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero, 0)
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    #[inline]
    pub fn new_zeroed_in(alloc: A) -> Shared<mem::MaybeUninit<T>, A> {
        unsafe {
            Shared::from_ptr_in(
                Shared::allocate_for_layout(
                    Layout::new::<T>(),
                    |layout| alloc.allocate_zeroed(layout),
                    <*mut u8>::cast,
                ),
                alloc,
            )
        }
    }

    /// Constructs a new `Arc<T, A>` in the provided allocator, returning an error if allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let five = Arc::try_new_in(5, System)?;
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn try_new_in(data: T, alloc: A) -> Result<Shared<T, A>, AllocError> {
        // Start the weak pointer count as 1 which is the weak pointer that's
        // held by all the strong pointers (kinda), see std/rc.rs for more info
        let x = Box::try_new_in(
            ArcRwLockInner {
                ref_count: atomic::AtomicUsize::new(1),
                lock: RawRwLock::new(),
                data: UnsafeCell::new(data),
            },
            alloc,
        )?;
        let (ptr, alloc) = Box::into_raw_with_allocator(x);
        Ok(unsafe { Self::from_inner_in(NonNull::new_unchecked(ptr), alloc) })
    }

    /// Constructs a new `Arc` with uninitialized contents, in the provided allocator, returning an
    /// error if allocation fails.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit, allocator_api)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let mut five = Arc::<u32, _>::try_new_uninit_in(System)?;
    ///
    /// let five = unsafe {
    ///     // Deferred initialization:
    ///     Arc::get_mut_unchecked(&mut five).as_mut_ptr().write(5);
    ///
    ///     five.assume_init()
    /// };
    ///
    /// assert_eq!(*five, 5);
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    #[inline]
    pub fn try_new_uninit_in(alloc: A) -> Result<Shared<mem::MaybeUninit<T>, A>, AllocError> {
        unsafe {
            Ok(Shared::from_ptr_in(
                Shared::try_allocate_for_layout(
                    Layout::new::<T>(),
                    |layout| alloc.allocate(layout),
                    <*mut u8>::cast,
                )?,
                alloc,
            ))
        }
    }

    /// Constructs a new `Arc` with uninitialized contents, with the memory
    /// being filled with `0` bytes, in the provided allocator, returning an error if allocation
    /// fails.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and incorrect usage
    /// of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit, allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let zero = Arc::<u32, _>::try_new_zeroed_in(System)?;
    /// let zero = unsafe { zero.assume_init() };
    ///
    /// assert_eq!(*zero, 0);
    /// # Ok::<(), std::alloc::AllocError>(())
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    #[inline]
    pub fn try_new_zeroed_in(alloc: A) -> Result<Shared<mem::MaybeUninit<T>, A>, AllocError> {
        unsafe {
            Ok(Shared::from_ptr_in(
                Shared::try_allocate_for_layout(
                    Layout::new::<T>(),
                    |layout| alloc.allocate_zeroed(layout),
                    <*mut u8>::cast,
                )?,
                alloc,
            ))
        }
    }
    /// Returns the inner value, if the `Arc` has exactly one strong reference.
    ///
    /// Otherwise, an [`Err`] is returned with the same `Arc` that was
    /// passed in.
    ///
    /// This will succeed even if there are outstanding weak references.
    ///
    /// It is strongly recommended to use [`Arc::into_inner`] instead if you don't
    /// keep the `Arc` in the [`Err`] case.
    /// Immediately dropping the [`Err`]-value, as the expression
    /// `Arc::try_unwrap(this).ok()` does, can cause the strong count to
    /// drop to zero and the inner value of the `Arc` to be dropped.
    /// For instance, if two threads execute such an expression in parallel,
    /// there is a race condition without the possibility of unsafety:
    /// The threads could first both check whether they own the last instance
    /// in `Arc::try_unwrap`, determine that they both do not, and then both
    /// discard and drop their instance in the call to [`ok`][`Result::ok`].
    /// In this scenario, the value inside the `Arc` is safely destroyed
    /// by exactly one of the threads, but neither thread will ever be able
    /// to use the value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let x = Arc::new(3);
    /// assert_eq!(Arc::try_unwrap(x), Ok(3));
    ///
    /// let x = Arc::new(4);
    /// let _y = Arc::clone(&x);
    /// assert_eq!(*Arc::try_unwrap(x).unwrap_err(), 4);
    /// ```
    #[inline]
    
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        if this.inner().ref_count.compare_exchange(1, 0, Relaxed, Relaxed).is_err() {
            return Err(this);
        }

        acquire!(this.inner().ref_count);

        let mut this = ManuallyDrop::new(this);
        let elem: T = unsafe { ptr::read(this.ptr.as_mut().data.get_mut() ) };
        let _alloc: A = unsafe { ptr::read(&this.alloc) }; // copy the allocator

        Ok(elem)
    }

    /// Returns the inner value, if the `Arc` has exactly one strong reference.
    ///
    /// Otherwise, [`None`] is returned and the `Arc` is dropped.
    ///
    /// This will succeed even if there are outstanding weak references.
    ///
    /// If `Arc::into_inner` is called on every clone of this `Arc`,
    /// it is guaranteed that exactly one of the calls returns the inner value.
    /// This means in particular that the inner value is not dropped.
    ///
    /// [`Arc::try_unwrap`] is conceptually similar to `Arc::into_inner`, but it
    /// is meant for different use-cases. If used as a direct replacement
    /// for `Arc::into_inner` anyway, such as with the expression
    /// <code>[Arc::try_unwrap]\(this).[ok][Result::ok]()</code>, then it does
    /// **not** give the same guarantee as described in the previous paragraph.
    /// For more information, see the examples below and read the documentation
    /// of [`Arc::try_unwrap`].
    ///
    /// # Examples
    ///
    /// Minimal example demonstrating the guarantee that `Arc::into_inner` gives.
    /// ```
    /// use std::sync::Arc;
    ///
    /// let x = Arc::new(3);
    /// let y = Arc::clone(&x);
    ///
    /// // Two threads calling `Arc::into_inner` on both clones of an `Arc`:
    /// let x_thread = std::thread::spawn(|| Arc::into_inner(x));
    /// let y_thread = std::thread::spawn(|| Arc::into_inner(y));
    ///
    /// let x_inner_value = x_thread.join().unwrap();
    /// let y_inner_value = y_thread.join().unwrap();
    ///
    /// // One of the threads is guaranteed to receive the inner value:
    /// assert!(matches!(
    ///     (x_inner_value, y_inner_value),
    ///     (None, Some(3)) | (Some(3), None)
    /// ));
    /// // The result could also be `(None, None)` if the threads called
    /// // `Arc::try_unwrap(x).ok()` and `Arc::try_unwrap(y).ok()` instead.
    /// ```
    ///
    /// A more practical example demonstrating the need for `Arc::into_inner`:
    /// ```
    /// use std::sync::Arc;
    ///
    /// // Definition of a simple singly linked list using `Arc`:
    /// #[derive(Clone)]
    /// struct LinkedList<T>(Option<Arc<Node<T>>>);
    /// struct Node<T>(T, Option<Arc<Node<T>>>);
    ///
    /// // Dropping a long `LinkedList<T>` relying on the destructor of `Arc`
    /// // can cause a stack overflow. To prevent this, we can provide a
    /// // manual `Drop` implementation that does the destruction in a loop:
    /// impl<T> Drop for LinkedList<T> {
    ///     fn drop(&mut self) {
    ///         let mut link = self.0.take();
    ///         while let Some(arc_node) = link.take() {
    ///             if let Some(Node(_value, next)) = Arc::into_inner(arc_node) {
    ///                 link = next;
    ///             }
    ///         }
    ///     }
    /// }
    ///
    /// // Implementation of `new` and `push` omitted
    /// impl<T> LinkedList<T> {
    ///     /* ... */
    /// #   fn new() -> Self {
    /// #       LinkedList(None)
    /// #   }
    /// #   fn push(&mut self, x: T) {
    /// #       self.0 = Some(Arc::new(Node(x, self.0.take())));
    /// #   }
    /// }
    ///
    /// // The following code could have still caused a stack overflow
    /// // despite the manual `Drop` impl if that `Drop` impl had used
    /// // `Arc::try_unwrap(arc).ok()` instead of `Arc::into_inner(arc)`.
    ///
    /// // Create a long list and clone it
    /// let mut x = LinkedList::new();
    /// let size = 100000;
    /// # let size = if cfg!(miri) { 100 } else { size };
    /// for i in 0..size {
    ///     x.push(i); // Adds i to the front of x
    /// }
    /// let y = x.clone();
    ///
    /// // Drop the clones in parallel
    /// let x_thread = std::thread::spawn(|| drop(x));
    /// let y_thread = std::thread::spawn(|| drop(y));
    /// x_thread.join().unwrap();
    /// y_thread.join().unwrap();
    /// ```
    #[inline]
    
    pub fn into_inner(this: Self) -> Option<T> {
        // Make sure that the ordinary `Drop` implementation isn’t called as well
        let mut this = mem::ManuallyDrop::new(this);

        // Following the implementation of `drop` and `drop_slow`
        if this.inner().ref_count.fetch_sub(1, Release) != 1 {
            return None;
        }

        acquire!(this.inner().ref_count);

        // SAFETY: This mirrors the line
        //     unsafe { ptr::drop_in_place(Self::get_mut_unchecked(self)) };
        // in `drop_slow`. Instead of dropping the value behind the pointer,
        // it is read and eventually returned; `ptr::read` has the same
        // safety conditions as `ptr::drop_in_place`.

        let inner = unsafe { ptr::read(Self::get_mut_unchecked(&mut this)) };
        let _alloc = unsafe { ptr::read(&this.alloc) };

        Some(inner)
    }
}

impl<T> Shared<[T]> {
    /// Constructs a new atomically reference-counted slice with uninitialized contents.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let mut values = Arc::<[u32]>::new_uninit_slice(3);
    ///
    /// // Deferred initialization:
    /// let data = Arc::get_mut(&mut values).unwrap();
    /// data[0].write(1);
    /// data[1].write(2);
    /// data[2].write(3);
    ///
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values, [1, 2, 3])
    /// ```
    
    #[inline]
    
    #[must_use]
    pub fn new_uninit_slice(len: usize) -> Shared<[mem::MaybeUninit<T>]> {
        unsafe { Shared::from_ptr(Shared::allocate_for_slice(len)) }
    }

    /// Constructs a new atomically reference-counted slice with uninitialized contents, with the memory being
    /// filled with `0` bytes.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and
    /// incorrect usage of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    ///
    /// use std::sync::Arc;
    ///
    /// let values = Arc::<[u32]>::new_zeroed_slice(3);
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values, [0, 0, 0])
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    
    #[inline]
    
    #[must_use]
    pub fn new_zeroed_slice(len: usize) -> Shared<[mem::MaybeUninit<T>]> {
        unsafe {
            Shared::from_ptr(Shared::allocate_for_layout(
                Layout::array::<T>(len).unwrap(),
                |layout| Global.allocate_zeroed(layout),
                |mem| {
                    ptr::slice_from_raw_parts_mut(mem as *mut T, len)
                        as *mut ArcRwLockInner<[mem::MaybeUninit<T>]>
                },
            ))
        }
    }
}

impl<T, A: Allocator> Shared<[T], A> {
    /// Constructs a new atomically reference-counted slice with uninitialized contents in the
    /// provided allocator.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let mut values = Arc::<[u32], _>::new_uninit_slice_in(3, System);
    ///
    /// let values = unsafe {
    ///     // Deferred initialization:
    ///     Arc::get_mut_unchecked(&mut values)[0].as_mut_ptr().write(1);
    ///     Arc::get_mut_unchecked(&mut values)[1].as_mut_ptr().write(2);
    ///     Arc::get_mut_unchecked(&mut values)[2].as_mut_ptr().write(3);
    ///
    ///     values.assume_init()
    /// };
    ///
    /// assert_eq!(*values, [1, 2, 3])
    /// ```
    
    
    #[inline]
    pub fn new_uninit_slice_in(len: usize, alloc: A) -> Shared<[mem::MaybeUninit<T>], A> {
        unsafe { Shared::from_ptr_in(Shared::allocate_for_slice_in(len, &alloc), alloc) }
    }

    /// Constructs a new atomically reference-counted slice with uninitialized contents, with the memory being
    /// filled with `0` bytes, in the provided allocator.
    ///
    /// See [`MaybeUninit::zeroed`][zeroed] for examples of correct and
    /// incorrect usage of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let values = Arc::<[u32], _>::new_zeroed_slice_in(3, System);
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values, [0, 0, 0])
    /// ```
    ///
    /// [zeroed]: mem::MaybeUninit::zeroed
    
    
    #[inline]
    pub fn new_zeroed_slice_in(len: usize, alloc: A) -> Shared<[mem::MaybeUninit<T>], A> {
        unsafe {
            Shared::from_ptr_in(
                Shared::allocate_for_layout(
                    Layout::array::<T>(len).unwrap(),
                    |layout| alloc.allocate_zeroed(layout),
                    |mem| {
                        ptr::slice_from_raw_parts_mut(mem.cast::<T>(), len)
                            as *mut ArcRwLockInner<[mem::MaybeUninit<T>]>
                    },
                ),
                alloc,
            )
        }
    }
}

impl<T, A: Allocator> Shared<mem::MaybeUninit<T>, A> {
    /// Converts to `Arc<T>`.
    ///
    /// # Safety
    ///
    /// As with [`MaybeUninit::assume_init`],
    /// it is up to the caller to guarantee that the inner value
    /// really is in an initialized state.
    /// Calling this when the content is not yet fully initialized
    /// causes immediate undefined behavior.
    ///
    /// [`MaybeUninit::assume_init`]: mem::MaybeUninit::assume_init
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let mut five = Arc::<u32>::new_uninit();
    ///
    /// // Deferred initialization:
    /// Arc::get_mut(&mut five).unwrap().write(5);
    ///
    /// let five = unsafe { five.assume_init() };
    ///
    /// assert_eq!(*five, 5)
    /// ```
    
    #[must_use = "`self` will be dropped if the result is not used"]
    #[inline]
    pub unsafe fn assume_init(self) -> Shared<T, A> {
        let (ptr, alloc) = Shared::into_inner_with_allocator(self);
        unsafe { Shared::from_inner_in(ptr.cast(), alloc) }
    }
}

impl<T, A: Allocator> Shared<[mem::MaybeUninit<T>], A> {
    /// Converts to `Arc<[T]>`.
    ///
    /// # Safety
    ///
    /// As with [`MaybeUninit::assume_init`],
    /// it is up to the caller to guarantee that the inner value
    /// really is in an initialized state.
    /// Calling this when the content is not yet fully initialized
    /// causes immediate undefined behavior.
    ///
    /// [`MaybeUninit::assume_init`]: mem::MaybeUninit::assume_init
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(new_uninit)]
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let mut values = Arc::<[u32]>::new_uninit_slice(3);
    ///
    /// // Deferred initialization:
    /// let data = Arc::get_mut(&mut values).unwrap();
    /// data[0].write(1);
    /// data[1].write(2);
    /// data[2].write(3);
    ///
    /// let values = unsafe { values.assume_init() };
    ///
    /// assert_eq!(*values, [1, 2, 3])
    /// ```
    
    #[must_use = "`self` will be dropped if the result is not used"]
    #[inline]
    pub unsafe fn assume_init(self) -> Shared<[T], A> {
        let (ptr, alloc) = Shared::into_inner_with_allocator(self);
        unsafe { Shared::from_ptr_in(ptr.as_ptr() as _, alloc) }
    }
}

impl<T: ?Sized> Shared<T> {
    /// Constructs an `Arc<T>` from a raw pointer.
    ///
    /// The raw pointer must have been previously returned by a call to
    /// [`Arc<U>::into_raw`][into_raw] with the following requirements:
    ///
    /// * If `U` is sized, it must have the same size and alignment as `T`. This
    ///   is trivially true if `U` is `T`.
    /// * If `U` is unsized, its data pointer must have the same size and
    ///   alignment as `T`. This is trivially true if `Arc<U>` was constructed
    ///   through `Arc<T>` and then converted to `Arc<U>` through an [unsized
    ///   coercion].
    ///
    /// Note that if `U` or `U`'s data pointer is not `T` but has the same size
    /// and alignment, this is basically like transmuting references of
    /// different types. See [`mem::transmute`][transmute] for more information
    /// on what restrictions apply in this case.
    ///
    /// The user of `from_raw` has to make sure a specific value of `T` is only
    /// dropped once.
    ///
    /// This function is unsafe because improper use may lead to memory unsafety,
    /// even if the returned `Arc<T>` is never accessed.
    ///
    /// [into_raw]: Arc::into_raw
    /// [transmute]: core::mem::transmute
    /// [unsized coercion]: https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let x = Arc::new("hello".to_owned());
    /// let x_ptr = Arc::into_raw(x);
    ///
    /// unsafe {
    ///     // Convert back to an `Arc` to prevent leak.
    ///     let x = Arc::from_raw(x_ptr);
    ///     assert_eq!(&*x, "hello");
    ///
    ///     // Further calls to `Arc::from_raw(x_ptr)` would be memory-unsafe.
    /// }
    ///
    /// // The memory was freed when `x` went out of scope above, so `x_ptr` is now dangling!
    /// ```
    ///
    /// Convert a slice back into its original array:
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let x: Arc<[u32]> = Arc::new([1, 2, 3]);
    /// let x_ptr: *const [u32] = Arc::into_raw(x);
    ///
    /// unsafe {
    ///     let x: Arc<[u32; 3]> = Arc::from_raw(x_ptr.cast::<[u32; 3]>());
    ///     assert_eq!(&*x, &[1, 2, 3]);
    /// }
    /// ```
    #[inline]
    
    pub unsafe fn from_raw(ptr: *const T) -> Self {
        unsafe { Shared::from_raw_in(ptr, Global) }
    }

    /// Increments the strong reference count on the `Arc<T>` associated with the
    /// provided pointer by one.
    ///
    /// # Safety
    ///
    /// The pointer must have been obtained through `Arc::into_raw`, and the
    /// associated `Arc` instance must be valid (i.e. the strong count must be at
    /// least 1) for the duration of this method.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// unsafe {
    ///     let ptr = Arc::into_raw(five);
    ///     Arc::increment_ref_count(ptr);
    ///
    ///     // This assertion is deterministic because we haven't shared
    ///     // the `Arc` between threads.
    ///     let five = Arc::from_raw(ptr);
    ///     assert_eq!(2, Arc::ref_count(&five));
    /// #   // Prevent leaks for Miri.
    /// #   Arc::decrement_ref_count(ptr);
    /// }
    /// ```
    #[inline]
    
    pub unsafe fn increment_ref_count(ptr: *const T) {
        unsafe { Shared::increment_ref_count_in(ptr, Global) }
    }

    /// Decrements the strong reference count on the `Arc<T>` associated with the
    /// provided pointer by one.
    ///
    /// # Safety
    ///
    /// The pointer must have been obtained through `Arc::into_raw`, and the
    /// associated `Arc` instance must be valid (i.e. the strong count must be at
    /// least 1) when invoking this method. This method can be used to release the final
    /// `Arc` and backing storage, but **should not** be called after the final `Arc` has been
    /// released.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// unsafe {
    ///     let ptr = Arc::into_raw(five);
    ///     Arc::increment_ref_count(ptr);
    ///
    ///     // Those assertions are deterministic because we haven't shared
    ///     // the `Arc` between threads.
    ///     let five = Arc::from_raw(ptr);
    ///     assert_eq!(2, Arc::ref_count(&five));
    ///     Arc::decrement_ref_count(ptr);
    ///     assert_eq!(1, Arc::ref_count(&five));
    /// }
    /// ```
    #[inline]
    
    pub unsafe fn decrement_ref_count(ptr: *const T) {
        unsafe { Shared::decrement_ref_count_in(ptr, Global) }
    }
}

impl<T: ?Sized, A: Allocator> Shared<T, A> {
    /// Returns a reference to the underlying allocator.
    ///
    /// Note: this is an associated function, which means that you have
    /// to call it as `Arc::allocator(&a)` instead of `a.allocator()`. This
    /// is so that there is no conflict with a method on the inner type.
    #[inline]
    
    pub fn allocator(this: &Self) -> &A {
        &this.alloc
    }

    /// Consumes the `Arc`, returning the wrapped pointer.
    ///
    /// To avoid a memory leak the pointer must be converted back to an `Arc` using
    /// [`Arc::from_raw`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let x = Arc::new("hello".to_owned());
    /// let x_ptr = Arc::into_raw(x);
    /// assert_eq!(unsafe { &*x_ptr }, "hello");
    /// # // Prevent leaks for Miri.
    /// # drop(unsafe { Arc::from_raw(x_ptr) });
    /// ```
    #[must_use = "losing the pointer will leak memory"]
    pub fn into_raw(this: Self) -> *const T {
        let this = ManuallyDrop::new(this);
        Self::as_ptr(&*this)
    }

    /// Consumes the `Arc`, returning the wrapped pointer and allocator.
    ///
    /// To avoid a memory leak the pointer must be converted back to an `Arc` using
    /// [`Arc::from_raw_in`].
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let x = Arc::new_in("hello".to_owned(), System);
    /// let (ptr, alloc) = Arc::into_raw_with_allocator(x);
    /// assert_eq!(unsafe { &*ptr }, "hello");
    /// let x = unsafe { Arc::from_raw_in(ptr, alloc) };
    /// assert_eq!(&*x, "hello");
    /// ```
    #[must_use = "losing the pointer will leak memory"]
    
    pub fn into_raw_with_allocator(this: Self) -> (*const T, A) {
        let this = mem::ManuallyDrop::new(this);
        let ptr = Self::as_ptr(&this);
        // Safety: `this` is ManuallyDrop so the allocator will not be double-dropped
        let alloc = unsafe { ptr::read(&this.alloc) };
        (ptr, alloc)
    }

    /// Provides a raw pointer to the data.
    ///
    /// The counts are not affected in any way and the `Arc` is not consumed. The pointer is valid for
    /// as long as there are strong counts in the `Arc`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let x = Arc::new("hello".to_owned());
    /// let y = Arc::clone(&x);
    /// let x_ptr = Arc::as_ptr(&x);
    /// assert_eq!(x_ptr, Arc::as_ptr(&y));
    /// assert_eq!(unsafe { &*x_ptr }, "hello");
    /// ```
    #[must_use]
    fn as_ptr(this: &Self) -> *const T {
        this.inner().data.get()
    }

    /// Constructs an `Arc<T, A>` from a raw pointer.
    ///
    /// The raw pointer must have been previously returned by a call to [`Arc<U,
    /// A>::into_raw`][into_raw] with the following requirements:
    ///
    /// * If `U` is sized, it must have the same size and alignment as `T`. This
    ///   is trivially true if `U` is `T`.
    /// * If `U` is unsized, its data pointer must have the same size and
    ///   alignment as `T`. This is trivially true if `Arc<U>` was constructed
    ///   through `Arc<T>` and then converted to `Arc<U>` through an [unsized
    ///   coercion].
    ///
    /// Note that if `U` or `U`'s data pointer is not `T` but has the same size
    /// and alignment, this is basically like transmuting references of
    /// different types. See [`mem::transmute`][transmute] for more information
    /// on what restrictions apply in this case.
    ///
    /// The raw pointer must point to a block of memory allocated by `alloc`
    ///
    /// The user of `from_raw` has to make sure a specific value of `T` is only
    /// dropped once.
    ///
    /// This function is unsafe because improper use may lead to memory unsafety,
    /// even if the returned `Arc<T>` is never accessed.
    ///
    /// [into_raw]: Arc::into_raw
    /// [transmute]: core::mem::transmute
    /// [unsized coercion]: https://doc.rust-lang.org/reference/type-coercions.html#unsized-coercions
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let x = Arc::new_in("hello".to_owned(), System);
    /// let x_ptr = Arc::into_raw(x);
    ///
    /// unsafe {
    ///     // Convert back to an `Arc` to prevent leak.
    ///     let x = Arc::from_raw_in(x_ptr, System);
    ///     assert_eq!(&*x, "hello");
    ///
    ///     // Further calls to `Arc::from_raw(x_ptr)` would be memory-unsafe.
    /// }
    ///
    /// // The memory was freed when `x` went out of scope above, so `x_ptr` is now dangling!
    /// ```
    ///
    /// Convert a slice back into its original array:
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let x: Arc<[u32], _> = Arc::new_in([1, 2, 3], System);
    /// let x_ptr: *const [u32] = Arc::into_raw(x);
    ///
    /// unsafe {
    ///     let x: Arc<[u32; 3], _> = Arc::from_raw_in(x_ptr.cast::<[u32; 3]>(), System);
    ///     assert_eq!(&*x, &[1, 2, 3]);
    /// }
    /// ```
    #[inline]
    
    pub unsafe fn from_raw_in(ptr: *const T, alloc: A) -> Self {
        unsafe {
            let offset = data_offset(ptr);

            // Reverse the offset to find the original ArcRwLockInner.
            let arc_ptr = ptr.byte_sub(offset) as *mut ArcRwLockInner<T>;

            Self::from_ptr_in(arc_ptr, alloc)
        }
    }

    /// Gets the number of strong (`Arc`) pointers to this allocation.
    ///
    /// # Safety
    ///
    /// This method by itself is safe, but using it correctly requires extra care.
    /// Another thread can change the strong count at any time,
    /// including potentially between calling this method and acting on the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    /// let _also_five = Arc::clone(&five);
    ///
    /// // This assertion is deterministic because we haven't shared
    /// // the `Arc` between threads.
    /// assert_eq!(2, Arc::ref_count(&five));
    /// ```
    #[inline]
    #[must_use]
    pub fn ref_count(this: &Self) -> usize {
        this.inner().ref_count.load(Relaxed)
    }

    /// Increments the strong reference count on the `Arc<T>` associated with the
    /// provided pointer by one.
    ///
    /// # Safety
    ///
    /// The pointer must have been obtained through `Arc::into_raw`, and the
    /// associated `Arc` instance must be valid (i.e. the strong count must be at
    /// least 1) for the duration of this method,, and `ptr` must point to a block of memory
    /// allocated by `alloc`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let five = Arc::new_in(5, System);
    ///
    /// unsafe {
    ///     let ptr = Arc::into_raw(five);
    ///     Arc::increment_ref_count_in(ptr, System);
    ///
    ///     // This assertion is deterministic because we haven't shared
    ///     // the `Arc` between threads.
    ///     let five = Arc::from_raw_in(ptr, System);
    ///     assert_eq!(2, Arc::ref_count(&five));
    /// #   // Prevent leaks for Miri.
    /// #   Arc::decrement_ref_count_in(ptr, System);
    /// }
    /// ```
    #[inline]
    
    pub unsafe fn increment_ref_count_in(ptr: *const T, alloc: A)
    where
        A: Clone,
    {
        // Retain Arc, but don't touch refcount by wrapping in ManuallyDrop
        let arc = unsafe { mem::ManuallyDrop::new(Shared::from_raw_in(ptr, alloc)) };
        // Now increase refcount, but don't drop new refcount either
        let _arc_clone: mem::ManuallyDrop<_> = arc.clone();
    }

    /// Decrements the strong reference count on the `Arc<T>` associated with the
    /// provided pointer by one.
    ///
    /// # Safety
    ///
    /// The pointer must have been obtained through `Arc::into_raw`,  the
    /// associated `Arc` instance must be valid (i.e. the strong count must be at
    /// least 1) when invoking this method, and `ptr` must point to a block of memory
    /// allocated by `alloc`. This method can be used to release the final
    /// `Arc` and backing storage, but **should not** be called after the final `Arc` has been
    /// released.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(allocator_api)]
    ///
    /// use std::sync::Arc;
    /// use std::alloc::System;
    ///
    /// let five = Arc::new_in(5, System);
    ///
    /// unsafe {
    ///     let ptr = Arc::into_raw(five);
    ///     Arc::increment_ref_count_in(ptr, System);
    ///
    ///     // Those assertions are deterministic because we haven't shared
    ///     // the `Arc` between threads.
    ///     let five = Arc::from_raw_in(ptr, System);
    ///     assert_eq!(2, Arc::ref_count(&five));
    ///     Arc::decrement_ref_count_in(ptr, System);
    ///     assert_eq!(1, Arc::ref_count(&five));
    /// }
    /// ```
    #[inline]
    
    pub unsafe fn decrement_ref_count_in(ptr: *const T, alloc: A) {
        unsafe { drop(Shared::from_raw_in(ptr, alloc)) };
    }

    #[inline]
    fn inner(&self) -> &ArcRwLockInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // that the inner pointer is valid. Furthermore, we know that the
        // `ArcRwLockInner` structure itself is `Sync` because the inner data is
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // contents.
        unsafe { self.ptr.as_ref() }
    }

    // Non-inlined part of `drop`.
    #[inline(never)]
    unsafe fn drop_slow(&mut self) {
        // Destroy the data at this time, even though we must not free the box
        // allocation itself (there might still be weak pointers lying around).
        unsafe { ptr::drop_in_place(Self::get_mut_unchecked(self)) };
    }

    /// Returns `true` if the two `Arc`s point to the same allocation in a vein similar to
    /// [`ptr::eq`]. This function ignores the metadata of  `dyn Trait` pointers.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    /// let same_five = Arc::clone(&five);
    /// let other_five = Arc::new(5);
    ///
    /// assert!(Arc::ptr_eq(&five, &same_five));
    /// assert!(!Arc::ptr_eq(&five, &other_five));
    /// ```
    ///
    /// [`ptr::eq`]: core::ptr::eq "ptr::eq"
    #[inline]
    #[must_use]
    
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        ptr::addr_eq(this.ptr.as_ptr(), other.ptr.as_ptr())
    }
}

impl<T: ?Sized> Shared<T> {
    /// Allocates an `ArcRwLockInner<T>` with sufficient space for
    /// a possibly-unsized inner value where the value has the layout provided.
    ///
    /// The function `mem_to_arcinner` is called with the data pointer
    /// and must return back a (potentially fat)-pointer for the `ArcRwLockInner<T>`.
    
    unsafe fn allocate_for_layout(
        value_layout: Layout,
        allocate: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocError>,
        mem_to_arcinner: impl FnOnce(*mut u8) -> *mut ArcRwLockInner<T>,
    ) -> *mut ArcRwLockInner<T> {
        let layout = arcinner_layout_for_value_layout(value_layout);

        let ptr = allocate(layout).unwrap_or_else(|_| handle_alloc_error(layout));

        unsafe { Self::initialize_arcinner(ptr, layout, mem_to_arcinner) }
    }

    /// Allocates an `ArcRwLockInner<T>` with sufficient space for
    /// a possibly-unsized inner value where the value has the layout provided,
    /// returning an error if allocation fails.
    ///
    /// The function `mem_to_arcinner` is called with the data pointer
    /// and must return back a (potentially fat)-pointer for the `ArcRwLockInner<T>`.
    unsafe fn try_allocate_for_layout(
        value_layout: Layout,
        allocate: impl FnOnce(Layout) -> Result<NonNull<[u8]>, AllocError>,
        mem_to_arcinner: impl FnOnce(*mut u8) -> *mut ArcRwLockInner<T>,
    ) -> Result<*mut ArcRwLockInner<T>, AllocError> {
        let layout = arcinner_layout_for_value_layout(value_layout);

        let ptr = allocate(layout)?;

        let inner = unsafe { Self::initialize_arcinner(ptr, layout, mem_to_arcinner) };

        Ok(inner)
    }

    unsafe fn initialize_arcinner(
        ptr: NonNull<[u8]>,
        layout: Layout,
        mem_to_arcinner: impl FnOnce(*mut u8) -> *mut ArcRwLockInner<T>,
    ) -> *mut ArcRwLockInner<T> {
        let inner = mem_to_arcinner(ptr.as_non_null_ptr().as_ptr());
        debug_assert_eq!(unsafe { Layout::for_value_raw(inner) }, layout);

        unsafe {
            ptr::addr_of_mut!((*inner).ref_count).write(atomic::AtomicUsize::new(1));
        }

        inner
    }
}

impl<T: ?Sized, A: Allocator> Shared<T, A> {
    /// Allocates an `ArcRwLockInner<T>` with sufficient space for an unsized inner value.
    #[inline]
    
    unsafe fn allocate_for_ptr_in(ptr: *const T, alloc: &A) -> *mut ArcRwLockInner<T> {
        // Allocate for the `ArcRwLockInner<T>` using the given value.
        unsafe {
            Shared::allocate_for_layout(
                Layout::for_value_raw(ptr),
                |layout| alloc.allocate(layout),
                |mem| mem.with_metadata_of(ptr as *const ArcRwLockInner<T>),
            )
        }
    }

    
    fn from_box_in(src: Box<T, A>) -> Shared<T, A> {
        unsafe {
            let value_size = size_of_val(&*src);
            let ptr = Self::allocate_for_ptr_in(&*src, Box::allocator(&src));

            // Copy value as bytes
            ptr::copy_nonoverlapping(
                core::ptr::addr_of!(*src) as *const u8,
                ptr::addr_of_mut!((*ptr).data) as *mut u8,
                value_size,
            );

            // Free the allocation without dropping its contents
            let (bptr, alloc) = Box::into_raw_with_allocator(src);
            let src = Box::from_raw_in(bptr as *mut mem::ManuallyDrop<T>, alloc.by_ref());
            drop(src);

            Self::from_ptr_in(ptr, alloc)
        }
    }
}

impl<T> Shared<[T]> {
    /// Allocates an `ArcRwLockInner<[T]>` with the given length.
    
    unsafe fn allocate_for_slice(len: usize) -> *mut ArcRwLockInner<[T]> {
        unsafe {
            Self::allocate_for_layout(
                Layout::array::<T>(len).unwrap(),
                |layout| Global.allocate(layout),
                |mem| ptr::slice_from_raw_parts_mut(mem.cast::<T>(), len) as *mut ArcRwLockInner<[T]>,
            )
        }
    }

    /// Copy elements from slice into newly allocated `Arc<[T]>`
    ///
    /// Unsafe because the caller must either take ownership or bind `T: Copy`.
    
    unsafe fn copy_from_slice(v: &[T]) -> Shared<[T]> {
        unsafe {
            let ptr = Self::allocate_for_slice(v.len());

            ptr::copy_nonoverlapping(v.as_ptr(), ptr::addr_of_mut!((*ptr).data) as *mut T, v.len());

            Self::from_ptr(ptr)
        }
    }

    /// Constructs an `Arc<[T]>` from an iterator known to be of a certain size.
    ///
    /// Behavior is undefined should the size be wrong.
    
    unsafe fn from_iter_exact(iter: impl Iterator<Item = T>, len: usize) -> Shared<[T]> {
        // Panic guard while cloning T elements.
        // In the event of a panic, elements that have been written
        // into the new ArcRwLockInner will be dropped, then the memory freed.
        struct Guard<T> {
            mem: NonNull<u8>,
            elems: *mut T,
            layout: Layout,
            n_elems: usize,
        }

        impl<T> Drop for Guard<T> {
            fn drop(&mut self) {
                unsafe {
                    let slice = core::slice::from_raw_parts_mut(self.elems, self.n_elems);
                    ptr::drop_in_place(slice);

                    Global.deallocate(self.mem, self.layout);
                }
            }
        }

        unsafe {
            let ptr = Self::allocate_for_slice(len);

            let mem = ptr as *mut _ as *mut u8;
            let layout = Layout::for_value_raw(ptr);

            // Pointer to first element
            let elems = ptr::addr_of_mut!((*ptr).data) as *mut T;

            let mut guard = Guard { mem: NonNull::new_unchecked(mem), elems, layout, n_elems: 0 };

            for (i, item) in iter.enumerate() {
                ptr::write(elems.add(i), item);
                guard.n_elems += 1;
            }

            // All clear. Forget the guard so it doesn't free the new ArcRwLockInner.
            mem::forget(guard);

            Self::from_ptr(ptr)
        }
    }
}

impl<T, A: Allocator> Shared<[T], A> {
    /// Allocates an `ArcRwLockInner<[T]>` with the given length.
    #[inline]
    
    unsafe fn allocate_for_slice_in(len: usize, alloc: &A) -> *mut ArcRwLockInner<[T]> {
        unsafe {
            Shared::allocate_for_layout(
                Layout::array::<T>(len).unwrap(),
                |layout| alloc.allocate(layout),
                |mem| ptr::slice_from_raw_parts_mut(mem.cast::<T>(), len) as *mut ArcRwLockInner<[T]>,
            )
        }
    }
}

/// Specialization trait used for `From<&[T]>`.

trait ArcFromSlice<T> {
    fn from_slice(slice: &[T]) -> Self;
}


impl<T: Clone> ArcFromSlice<T> for Shared<[T]> {
    #[inline]
    default fn from_slice(v: &[T]) -> Self {
        unsafe { Self::from_iter_exact(v.iter().cloned(), v.len()) }
    }
}


impl<T: Copy> ArcFromSlice<T> for Shared<[T]> {
    #[inline]
    fn from_slice(v: &[T]) -> Self {
        unsafe { Shared::copy_from_slice(v) }
    }
}


impl<T: ?Sized, A: Allocator + Clone> Clone for Shared<T, A> {
    /// Makes a clone of the `Arc` pointer.
    ///
    /// This creates another pointer to the same allocation, increasing the
    /// strong reference count.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// let _ = Arc::clone(&five);
    /// ```
    #[inline]
    fn clone(&self) -> Shared<T, A> {
        // Using a relaxed ordering is alright here, as knowledge of the
        // original reference prevents other threads from erroneously deleting
        // the object.
        // As explained in the [Boost documentation][1], Increasing the
        // reference counter can always be done with memory_order_relaxed: New
        // references to an object can only be formed from an existing
        // reference, and passing an existing reference from one thread to
        // another must already provide any required synchronization.
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        let old_size = self.inner().ref_count.fetch_add(1, Relaxed);

        // However we need to guard against massive refcounts in case someone is `mem::forget`ing
        // Arcs. If we don't do this the count can overflow and users will use-after free. This
        // branch will never be taken in any realistic program. We abort because such a program is
        // incredibly degenerate, and we don't care to support it.
        // This check is not 100% water-proof: we error when the refcount grows beyond `isize::MAX`.
        // But we do that check *after* having done the increment, so there is a chance here that
        // the worst already happened and we actually do overflow the `usize` counter. However, that
        // requires the counter to grow from `isize::MAX` to `usize::MAX` between the increment
        // above and the `abort` below, which seems exceedingly unlikely.
        // This is a global invariant, and also applies when using a compare-exchange loop to increment
        // counters in other methods.
        // Otherwise, the counter could be brought to an almost-overflow using a compare-exchange loop,
        // and then overflow using a few `fetch_add`s.
        if old_size > MAX_REFCOUNT {
            abort(AbortLevel::Panic(), crate::result::ResultCode::new(u32::MAX));
        }

        unsafe { Self::from_inner_in(self.ptr, self.alloc.clone()) }
    }
}




//unsafe impl<T: ?Sized, A: Allocator> PinCoerceUnsized for Shared<T, A> {}

// impl<T: ?Sized> Receiver for Shared<T> {}

/*
impl<T: ?Sized + CloneToUninit, A: Allocator + Clone> Shared<T, A> {
    /// Makes a mutable reference into the given `Arc`.
    ///
    /// If there are other `Arc` pointers to the same allocation, then `make_mut` will
    /// [`clone`] the inner value to a new allocation to ensure unique ownership.  This is also
    /// referred to as clone-on-write.
    ///
    /// However, if there are no other `Arc` pointers to this allocation, but some [`Weak`]
    /// pointers, then the [`Weak`] pointers will be dissociated and the inner value will not
    /// be cloned.
    ///
    /// See also [`get_mut`], which will fail rather than cloning the inner value
    /// or dissociating [`Weak`] pointers.
    ///
    /// [`clone`]: Clone::clone
    /// [`get_mut`]: Arc::get_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let mut data = Arc::new(5);
    ///
    /// *Arc::make_mut(&mut data) += 1;         // Won't clone anything
    /// let mut other_data = Arc::clone(&data); // Won't clone inner data
    /// *Arc::make_mut(&mut data) += 1;         // Clones inner data
    /// *Arc::make_mut(&mut data) += 1;         // Won't clone anything
    /// *Arc::make_mut(&mut other_data) *= 2;   // Won't clone anything
    ///
    /// // Now `data` and `other_data` point to different allocations.
    /// assert_eq!(*data, 8);
    /// assert_eq!(*other_data, 12);
    /// ```
    ///
    /// [`Weak`] pointers will be dissociated:
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let mut data = Arc::new(75);
    /// let weak = Arc::downgrade(&data);
    ///
    /// assert!(75 == *data);
    /// assert!(75 == *weak.upgrade().unwrap());
    ///
    /// *Arc::make_mut(&mut data) += 1;
    ///
    /// assert!(76 == *data);
    /// assert!(weak.upgrade().is_none());
    /// ```
    #[inline]
    
    pub fn make_mut(this: &mut Self) -> &mut T {
        let size_of_val = mem::size_of_val::<T>(&**this);

        // Note that we hold both a strong reference and a weak reference.
        // Thus, releasing our strong reference only will not, by itself, cause
        // the memory to be deallocated.
        // Use Acquire to ensure that we see any writes to `weak` that happen
        // before release writes (i.e., decrements) to `strong`. Since we hold a
        // weak count, there's no chance the ArcRwLockInner itself could be
        // deallocated.
        if this.inner().ref_count.compare_exchange(1, 0, Acquire, Relaxed).is_err() {
            // Another strong pointer exists, so we must clone.

            let this_data_ref: &T = &**this;
            // `in_progress` drops the allocation if we panic before finishing initializing it.
            let mut in_progress: UniqueArcUninit<T, A> =
                UniqueArcUninit::new(this_data_ref, this.alloc.clone());

            let initialized_clone = unsafe {
                // Clone. If the clone panics, `in_progress` will be dropped and clean up.
                this_data_ref.clone_to_uninit(in_progress.data_ptr());
                // Cast type of pointer, now that it is initialized.
                in_progress.into_arc()
            };
            *this = initialized_clone;
        } else if this.inner().weak.load(Relaxed) != 1 {
            // Relaxed suffices in the above because this is fundamentally an
            // optimization: we are always racing with weak pointers being
            // dropped. Worst case, we end up allocated a new Arc unnecessarily.

            // We removed the last strong ref, but there are additional weak
            // refs remaining. We'll move the contents to a new Arc, and
            // invalidate the other weak refs.

            // Note that it is not possible for the read of `weak` to yield
            // usize::MAX (i.e., locked), since the weak count can only be
            // locked by a thread with a strong reference.

            // Materialize our own implicit weak pointer, so that it can clean
            // up the ArcRwLockInner as needed.
            let _weak = Weak { ptr: this.ptr, alloc: this.alloc.clone() };

            // Can just steal the data, all that's left is Weaks
            // We don't need panic-protection like the above branch does, but we might as well
            // use the same mechanism.
            let mut in_progress: UniqueArcUninit<T, A> =
                UniqueArcUninit::new(&**this, this.alloc.clone());
            unsafe {
                // Initialize `in_progress` with move of **this.
                // We have to express this in terms of bytes because `T: ?Sized`; there is no
                // operation that just copies a value based on its `size_of_val()`.
                ptr::copy_nonoverlapping(
                    ptr::from_ref(&**this).cast::<u8>(),
                    in_progress.data_ptr().cast::<u8>(),
                    size_of_val,
                );

                ptr::write(this, in_progress.into_arc());
            }
        } else {
            // We were the sole reference of either kind; bump back up the
            // strong ref count.
            this.inner().ref_count.store(1, Release);
        }

        // As with `get_mut()`, the unsafety is ok because our reference was
        // either unique to begin with, or became one upon cloning the contents.
        unsafe { Self::get_mut_unchecked(this) }
    }
}
 */

impl<T: Clone, A: Allocator> Shared<T, A> {
    /// If we have the only reference to `T` then unwrap it. Otherwise, clone `T` and return the
    /// clone.
    ///
    /// Assuming `arc_t` is of type `Arc<T>`, this function is functionally equivalent to
    /// `(*arc_t).clone()`, but will avoid cloning the inner value where possible.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::{ptr, sync::Arc};
    /// let inner = String::from("test");
    /// let ptr = inner.as_ptr();
    ///
    /// let arc = Arc::new(inner);
    /// let inner = Arc::unwrap_or_clone(arc);
    /// // The inner value was not cloned
    /// assert!(ptr::eq(ptr, inner.as_ptr()));
    ///
    /// let arc = Arc::new(inner);
    /// let arc2 = arc.clone();
    /// let inner = Arc::unwrap_or_clone(arc);
    /// // Because there were 2 references, we had to clone the inner value.
    /// assert!(!ptr::eq(ptr, inner.as_ptr()));
    /// // `arc2` is the last reference, so when we unwrap it we get back
    /// // the original `String`.
    /// let inner = Arc::unwrap_or_clone(arc2);
    /// assert!(ptr::eq(ptr, inner.as_ptr()));
    /// ```
    #[inline]
    
    pub fn unwrap_or_clone(this: Self) -> T {
        Shared::try_unwrap(this).unwrap_or_else(|arc| {
            arc.inner().lock.read();
            // we know the data pointer is valid, and we are locked for read
            let cloned = unsafe { arc.inner().data.get().as_mut().unwrap()}.clone();
            // SAFETY: we are read locked, invariant upheld
            unsafe {arc.inner().lock.read_unlock()};
            cloned
        })
    }
}

impl<T: ?Sized, A: Allocator> Shared<T, A> {
    /// Returns a mutable reference into the given `Arc`, if there are
    /// no other `Arc` or [`Weak`] pointers to the same allocation.
    ///
    /// Returns [`None`] otherwise, because it is not safe to
    /// mutate a shared value.
    ///
    /// See also [`make_mut`][make_mut], which will [`clone`][clone]
    /// the inner value when there are other `Arc` pointers.
    ///
    /// [make_mut]: Arc::make_mut
    /// [clone]: Clone::clone
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let mut x = Arc::new(3);
    /// *Arc::get_mut(&mut x).unwrap() = 4;
    /// assert_eq!(*x, 4);
    ///
    /// let _y = Arc::clone(&x);
    /// assert!(Arc::get_mut(&mut x).is_none());
    /// ```
    #[inline]
    
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        if this.is_unique() {
            // This unsafety is ok because we're guaranteed that the pointer
            // returned is the *only* pointer that will ever be returned to T. Our
            // reference count is guaranteed to be 1 at this point, and we required
            // the Arc itself to be `mut`, so we're returning the only possible
            // reference to the inner data.
            unsafe { Some(Shared::get_mut_unchecked(this)) }
        } else {
            None
        }
    }

    /// Returns a mutable reference into the given `Arc`,
    /// without any check.
    ///
    /// See also [`get_mut`], which is safe and does appropriate checks.
    ///
    /// [`get_mut`]: Arc::get_mut
    ///
    /// # Safety
    ///
    /// If any other `Arc` or [`Weak`] pointers to the same allocation exist, then
    /// they must not be dereferenced or have active borrows for the duration
    /// of the returned borrow, and their inner type must be exactly the same as the
    /// inner type of this Rc (including lifetimes). This is trivially the case if no
    /// such pointers exist, for example immediately after `Arc::new`.
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let mut x = Arc::new(String::new());
    /// unsafe {
    ///     Arc::get_mut_unchecked(&mut x).push_str("foo")
    /// }
    /// assert_eq!(*x, "foo");
    /// ```
    /// Other `Arc` pointers to the same allocation must be to the same type.
    /// ```no_run
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let x: Arc<str> = Arc::from("Hello, world!");
    /// let mut y: Arc<[u8]> = x.clone().into();
    /// unsafe {
    ///     // this is Undefined Behavior, because x's inner type is str, not [u8]
    ///     Arc::get_mut_unchecked(&mut y).fill(0xff); // 0xff is invalid in UTF-8
    /// }
    /// println!("{}", &*x); // Invalid UTF-8 in a str
    /// ```
    /// Other `Arc` pointers to the same allocation must be to the exact same type, including lifetimes.
    /// ```no_run
    /// #![feature(get_mut_unchecked)]
    ///
    /// use std::sync::Arc;
    ///
    /// let x: Arc<&str> = Arc::new("Hello, world!");
    /// {
    ///     let s = String::from("Oh, no!");
    ///     let mut y: Arc<&str> = x.clone().into();
    ///     unsafe {
    ///         // this is Undefined Behavior, because x's inner type
    ///         // is &'long str, not &'short str
    ///         *Arc::get_mut_unchecked(&mut y) = &s;
    ///     }
    /// }
    /// println!("{}", &*x); // Use-after-free
    /// ```
    #[inline]
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        unsafe { (*this.ptr.as_ptr()).data.get_mut() }
    }

    /// Determine whether this is the unique reference (including weak refs) to
    /// the underlying data.
    ///
    /// Note that this requires locking the weak ref count.
    fn is_unique(&mut self) -> bool {
        // This needs to be an `Acquire` to synchronize with the decrement of the `strong`
        // counter in `drop` -- the only access that happens when any but the last reference
        // is being dropped.
        self.inner().ref_count.load(Acquire) == 1
    }
}


impl<T: ?Sized, A: Allocator> Drop for Shared<T, A> {
    /// Drops the `Arc`.
    ///
    /// This will decrement the strong reference count. If the strong reference
    /// count reaches zero then the only other references (if any) are
    /// [`Weak`], so we `drop` the inner value.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo  = Arc::new(Foo);
    /// let foo2 = Arc::clone(&foo);
    ///
    /// drop(foo);    // Doesn't print anything
    /// drop(foo2);   // Prints "dropped!"
    /// ```
    #[inline]
    fn drop(&mut self) {
        // Because `fetch_sub` is already atomic, we do not need to synchronize
        // with other threads unless we are going to delete the object. This
        // same logic applies to the below `fetch_sub` to the `weak` count.
        if self.inner().ref_count.fetch_sub(1, Release) != 1 {
            return;
        }

        // This fence is needed to prevent reordering of use of the data and
        // deletion of the data. Because it is marked `Release`, the decreasing
        // of the reference count synchronizes with this `Acquire` fence. This
        // means that use of the data happens before decreasing the reference
        // count, which happens before this fence, which happens before the
        // deletion of the data.
        // As explained in the [Boost documentation][1],
        // > It is important to enforce any possible access to the object in one
        // > thread (through an existing reference) to *happen before* deleting
        // > the object in a different thread. This is achieved by a "release"
        // > operation after dropping a reference (any access to the object
        // > through this reference must obviously happened before), and an
        // > "acquire" operation before deleting the object.
        // In particular, while the contents of an Arc are usually immutable, it's
        // possible to have interior writes to something like a Mutex<T>. Since a
        // Mutex is not acquired when it is deleted, we can't rely on its
        // synchronization logic to make writes in thread A visible to a destructor
        // running in thread B.
        // Also note that the Acquire fence here could probably be replaced with an
        // Acquire load, which could improve performance in highly-contended
        // situations. See [2].
        // [1]: (www.boost.org/doc/libs/1_55_0/doc/html/atomic/usage_examples.html)
        // [2]: (https://github.com/rust-lang/rust/pull/41714)
        acquire!(self.inner().ref_count);

        // Make sure we aren't trying to "drop" the shared static for empty slices
        // used by Default::default.
        debug_assert!(
            !ptr::addr_eq(self.ptr.as_ptr(), &STATIC_INNER_SLICE.inner),
            "Arcs backed by a static should never reach a strong count of 0. \
            Likely decrement_ref_count or from_raw were called too many times.",
        );

        unsafe {
            self.drop_slow();
        }
    }
}

impl<A: Allocator> Shared<dyn Any + Send + Sync, A> {
    /// Attempts to downcast the `Arc<dyn Any + Send + Sync>` to a concrete type.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::any::Any;
    /// use std::sync::Arc;
    ///
    /// fn print_if_string(value: Arc<dyn Any + Send + Sync>) {
    ///     if let Ok(string) = value.downcast::<String>() {
    ///         println!("String ({}): {}", string.len(), string);
    ///     }
    /// }
    ///
    /// let my_string = "Hello World".to_string();
    /// print_if_string(Arc::new(my_string));
    /// print_if_string(Arc::new(0i8));
    /// ```
    #[inline]
    pub fn downcast<T>(mut self) -> Result<Shared<T, A>, Self>
    where
        T: Any + Send + Sync,
    {
        if (unsafe {self.ptr.as_mut().data.get_mut()}).is::<T>() {
            unsafe {
                let (ptr, alloc) = Shared::into_inner_with_allocator(self);
                Ok(Shared::from_inner_in(ptr.cast(), alloc))
            }
        } else {
            Err(self)
        }
    }

    /// Downcasts the `Arc<dyn Any + Send + Sync>` to a concrete type.
    ///
    /// For a safe alternative see [`downcast`].
    ///
    /// # Examples
    ///
    /// ```
    /// #![feature(downcast_unchecked)]
    ///
    /// use std::any::Any;
    /// use std::sync::Arc;
    ///
    /// let x: Arc<dyn Any + Send + Sync> = Arc::new(1_usize);
    ///
    /// unsafe {
    ///     assert_eq!(*x.downcast_unchecked::<usize>(), 1);
    /// }
    /// ```
    ///
    /// # Safety
    ///
    /// The contained value must be of type `T`. Calling this method
    /// with the incorrect type is *undefined behavior*.
    ///
    ///
    /// [`downcast`]: Self::downcast
    #[inline]
    pub unsafe fn downcast_unchecked<T>(self) -> Shared<T, A>
    where
        T: Any + Send + Sync,
    {
        unsafe {
            let (ptr, alloc) = Shared::into_inner_with_allocator(self);
            Shared::from_inner_in(ptr.cast(), alloc)
        }
    }
}
pub(crate) trait SharedPartialEq<T: ?Sized + PartialEq, A: Allocator> {
    fn eq(&self, other: &Shared<T, A>) -> bool;
    fn ne(&self, other: &Shared<T, A>) -> bool;
}


impl<T: ?Sized + PartialEq, A: Allocator> SharedPartialEq<T, A> for Shared<T, A> {
    #[inline]
    default fn eq(&self, other: &Shared<T, A>) -> bool {
        self.inner().lock.read();
        other.inner().lock.read();
        let res = unsafe { *self.inner().data.get() == *other.inner().data.get()};
        unsafe {
            self.inner().lock.read_unlock();
            other.inner().lock.read_unlock();
        }
        res
    }
    #[inline]
    default fn ne(&self, other: &Shared<T, A>) -> bool {
        self.inner().lock.read();
        other.inner().lock.read();
        let res = unsafe { *self.inner().data.get() != *other.inner().data.get()};
        unsafe {
            self.inner().lock.read_unlock();
            other.inner().lock.read_unlock();
        }
        res
    }
}
/* TODO check if I can port MarkerEq
/// We're doing this specialization here, and not as a more general optimization on `&T`, because it
/// would otherwise add a cost to all equality checks on refs. We assume that `Arc`s are used to
/// store large values, that are slow to clone, but also heavy to check for equality, causing this
/// cost to pay off more easily. It's also more likely to have two `Arc` clones, that point to
/// the same value, than two `&T`s.
///
/// We can only do this when `T: Eq` as a `PartialEq` might be deliberately irreflexive.

impl<T: ?Sized + core_alloc::rc::MarkerEq, A: Allocator> ArcEqIdent<T, A> for Shared<T, A> {
    #[inline]
    fn eq(&self, other: &Shared<T, A>) -> bool {
        Shared::ptr_eq(self, other) || **self == **other
    }

    #[inline]
    fn ne(&self, other: &Shared<T, A>) -> bool {
        !Shared::ptr_eq(self, other) && **self != **other
    }
}
*/

impl<T: ?Sized + PartialEq, A: Allocator> PartialEq for Shared<T, A> {
    /// Equality for two `Arc`s.
    ///
    /// Two `Arc`s are equal if their inner values are equal, even if they are
    /// stored in different allocation.
    ///
    /// If `T` also implements `Eq` (implying reflexivity of equality),
    /// two `Arc`s that point to the same allocation are always equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert!(five == Arc::new(5));
    /// ```
    #[inline]
    fn eq(&self, other: &Shared<T, A>) -> bool {
        SharedPartialEq::eq(self, other)
    }

    /// Inequality for two `Arc`s.
    ///
    /// Two `Arc`s are not equal if their inner values are not equal.
    ///
    /// If `T` also implements `Eq` (implying reflexivity of equality),
    /// two `Arc`s that point to the same value are always equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert!(five != Arc::new(6));
    /// ```
    #[inline]
    fn ne(&self, other: &Shared<T, A>) -> bool {
        SharedPartialEq::ne(self, other)
    }
}

/* TODO: todo
impl<T: ?Sized + PartialOrd, A: Allocator> PartialOrd for Shared<T, A> {
    /// Partial comparison for two `Arc`s.
    ///
    /// The two are compared by calling `partial_cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert_eq!(Some(Ordering::Less), five.partial_cmp(&Arc::new(6)));
    /// ```
    fn partial_cmp(&self, other: &Shared<T, A>) -> Option<core::cmp::Ordering> {
        (**self).partial_cmp(&**other)
    }

    /// Less-than comparison for two `Arc`s.
    ///
    /// The two are compared by calling `<` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert!(five < Arc::new(6));
    /// ```
    fn lt(&self, other: &Shared<T, A>) -> bool {
        *(*self) < *(*other)
    }

    /// 'Less than or equal to' comparison for two `Arc`s.
    ///
    /// The two are compared by calling `<=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert!(five <= Arc::new(5));
    /// ```
    fn le(&self, other: &Shared<T, A>) -> bool {
        *(*self) <= *(*other)
    }

    /// Greater-than comparison for two `Arc`s.
    ///
    /// The two are compared by calling `>` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert!(five > Arc::new(4));
    /// ```
    fn gt(&self, other: &Shared<T, A>) -> bool {
        *(*self) > *(*other)
    }

    /// 'Greater than or equal to' comparison for two `Arc`s.
    ///
    /// The two are compared by calling `>=` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert!(five >= Arc::new(5));
    /// ```
    fn ge(&self, other: &Shared<T, A>) -> bool {
        *(*self) >= *(*other)
    }
}

impl<T: ?Sized + Ord, A: Allocator> Ord for Shared<T, A> {
    /// Comparison for two `Arc`s.
    ///
    /// The two are compared by calling `cmp()` on their inner values.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use std::cmp::Ordering;
    ///
    /// let five = Arc::new(5);
    ///
    /// assert_eq!(Ordering::Less, five.cmp(&Arc::new(6)));
    /// ```
    fn cmp(&self, other: &Shared<T, A>) -> core::cmp::Ordering {
        (*self.read()).cmp(*other.read())
    }
}

impl<T: ?Sized + Eq, A: Allocator> Eq for Shared<T, A> {}
*/

impl<T: ?Sized + fmt::Display, A: Allocator> fmt::Display for Shared<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner().lock.read();
        let res = fmt::Display::fmt(unsafe {&*self.inner().data.get()}, f);
        unsafe { self.inner().lock.read_unlock() };
        res
    }
}


impl<T: ?Sized + fmt::Debug, A: Allocator> fmt::Debug for Shared<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner().lock.read();
        let res = try {
            f.write_str("Shared { ..., data: ")?;
            fmt::Debug::fmt(unsafe {&*self.inner().data.get()}, f)?;
            f.write_str("}")?
        };
        unsafe { self.inner().lock.read_unlock() };
        res
    }
}


impl<T: ?Sized, A: Allocator> fmt::Pointer for Shared<T, A> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&core::ptr::addr_of!(self.inner().data), f)
    }
}



impl<T: Default> Default for Shared<T> {
    /// Creates a new `Arc<T>`, with the `Default` value for `T`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    ///
    /// let x: Arc<i32> = Default::default();
    /// assert_eq!(*x, 0);
    /// ```
    fn default() -> Shared<T> {
        Shared::new(Default::default())
    }
}

/// Struct to hold the static `ArcRwLockInner` used for empty `Arc<str/CStr/[T]>` as
/// returned by `Default::default`.
///
/// Layout notes:
/// * `repr(align(16))` so we can use it for `[T]` with `align_of::<T>() <= 16`.
/// * `repr(C)` so `inner` is at offset 0 (and thus guaranteed to actually be aligned to 16).
/// * `[u8; 1]` (to be initialized with 0) so it can be used for `Arc<CStr>`.
#[repr(C, align(16))]
struct SliceArcRwLockInnerForStatic {
    inner: ArcRwLockInner<[u8; 1]>,
}

const MAX_STATIC_INNER_SLICE_ALIGNMENT: usize = 16;

static STATIC_INNER_SLICE: SliceArcRwLockInnerForStatic = SliceArcRwLockInnerForStatic {
    inner: ArcRwLockInner {
        ref_count: atomic::AtomicUsize::new(1),
        lock: RawRwLock::new(),
        data: UnsafeCell::new([0]),
    },
};



/*
impl Default for Shared<str> {
    /// Creates an empty str inside an Arc
    ///
    /// This may or may not share an allocation with other Arcs.
    #[inline]
    fn default() -> Self {
        let arc: Shared<[u8]> = Default::default();
        let guard: SharedReadGuard<'_, [u8]> = arc.read();
        debug_assert!(core::str::from_utf8(<SharedReadGuard<'_, [u8]> as Deref>::deref(guard)).is_ok());
        let (ptr, alloc) = Shared::into_inner_with_allocator(arc);
        unsafe { Shared::from_ptr_in(ptr.as_ptr() as *mut ArcRwLockInner<str>, alloc) }
    }
}
 */


impl Default for Shared<core::ffi::CStr> {
    /// Creates an empty CStr inside an Arc
    ///
    /// This may or may not share an allocation with other Arcs.
    #[inline]
    fn default() -> Self {
        use core::ffi::CStr;
        let inner: NonNull<ArcRwLockInner<[u8]>> = NonNull::from(&STATIC_INNER_SLICE.inner);
        let inner: NonNull<ArcRwLockInner<CStr>> =
            NonNull::new(inner.as_ptr() as *mut ArcRwLockInner<CStr>).unwrap();
        // `this` semantically is the Arc "owned" by the static, so make sure not to drop it.
        let this: mem::ManuallyDrop<Shared<CStr>> =
            unsafe { mem::ManuallyDrop::new(Shared::from_inner(inner)) };
        (*this).clone()
    }
}



impl<T> Default for Shared<[T]> {
    /// Creates an empty `[T]` inside an Arc
    ///
    /// This may or may not share an allocation with other Arcs.
    #[inline]
    fn default() -> Self {
        if mem::align_of::<T>() <= MAX_STATIC_INNER_SLICE_ALIGNMENT {
            // We take a reference to the whole struct instead of the ArcRwLockInner<[u8; 1]> inside it so
            // we don't shrink the range of bytes the ptr is allowed to access under Stacked Borrows.
            // (Miri complains on 32-bit targets with Arc<[Align16]> otherwise.)
            // (Note that NonNull::from(&STATIC_INNER_SLICE.inner) is fine under Tree Borrows.)
            let inner: NonNull<SliceArcRwLockInnerForStatic> = NonNull::from(&STATIC_INNER_SLICE);
            let inner: NonNull<ArcRwLockInner<[T; 0]>> = inner.cast();
            // `this` semantically is the Arc "owned" by the static, so make sure not to drop it.
            let this: mem::ManuallyDrop<Shared<[T; 0]>> =
                unsafe { mem::ManuallyDrop::new(Shared::from_inner(inner)) };
            return (*this).clone();
        }

        // If T's alignment is too large for the static, make a new unique allocation.
        let arr: [T; 0] = [];
        Shared::from(arr)
    }
}


impl<T: ?Sized + Hash, A: Allocator> Hash for Shared<T, A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner().lock.read();
        unsafe {&*self.inner().data.get()}.hash(state);
        unsafe { self.inner().lock.read_unlock()};
    }
}



impl<T> From<T> for Shared<T> {
    /// Converts a `T` into an `Arc<T>`
    ///
    /// The conversion moves the value into a
    /// newly allocated `Arc`. It is equivalent to
    /// calling `Arc::new(t)`.
    ///
    /// # Example
    /// ```rust
    /// # use std::sync::Arc;
    /// let x = 5;
    /// let arc = Arc::new(5);
    ///
    /// assert_eq!(Arc::from(x), arc);
    /// ```
    fn from(t: T) -> Self {
        Shared::new(t)
    }
}



impl<T, const N: usize> From<[T; N]> for Shared<[T]> {
    /// Converts a [`[T; N]`](prim@array) into an `Arc<[T]>`.
    ///
    /// The conversion moves the array into a newly allocated `Arc`.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// let original: [i32; 3] = [1, 2, 3];
    /// let shared: Arc<[i32]> = Arc::from(original);
    /// assert_eq!(&[1, 2, 3], &shared[..]);
    /// ```
    #[inline]
    fn from(v: [T; N]) -> Shared<[T]> {
        Shared::<[T; N]>::from(v)
    }
}



impl<T: Clone> From<&[T]> for Shared<[T]> {
    /// Allocates a reference-counted slice and fills it by cloning `v`'s items.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// let original: &[i32] = &[1, 2, 3];
    /// let shared: Arc<[i32]> = Arc::from(original);
    /// assert_eq!(&[1, 2, 3], &shared[..]);
    /// ```
    #[inline]
    fn from(v: &[T]) -> Shared<[T]> {
        <Self as ArcFromSlice<T>>::from_slice(v)
    }
}



impl From<&str> for Shared<str> {
    /// Allocates a reference-counted `str` and copies `v` into it.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// let shared: Arc<str> = Arc::from("eggplant");
    /// assert_eq!("eggplant", &shared[..]);
    /// ```
    #[inline]
    fn from(v: &str) -> Shared<str> {
        let arc = Shared::<[u8]>::from(v.as_bytes());
        unsafe { Shared::from_raw(Shared::into_raw(arc) as *const str) }
    }
}



impl From<String> for Shared<str> {
    /// Allocates a reference-counted `str` and copies `v` into it.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// let unique: String = "eggplant".to_owned();
    /// let shared: Arc<str> = Arc::from(unique);
    /// assert_eq!("eggplant", &shared[..]);
    /// ```
    #[inline]
    fn from(v: String) -> Shared<str> {
        Shared::from(&v[..])
    }
}



impl<T: ?Sized, A: Allocator> From<Box<T, A>> for Shared<T, A> {
    /// Move a boxed object to a new, reference-counted allocation.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// let unique: Box<str> = Box::from("eggplant");
    /// let shared: Arc<str> = Arc::from(unique);
    /// assert_eq!("eggplant", &shared[..]);
    /// ```
    #[inline]
    fn from(v: Box<T, A>) -> Shared<T, A> {
        Shared::from_box_in(v)
    }
}



impl<T, A: Allocator + Clone> From<Vec<T, A>> for Shared<[T], A> {
    /// Allocates a reference-counted slice and moves `v`'s items into it.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// let unique: Vec<i32> = vec![1, 2, 3];
    /// let shared: Arc<[i32]> = Arc::from(unique);
    /// assert_eq!(&[1, 2, 3], &shared[..]);
    /// ```
    #[inline]
    fn from(v: Vec<T, A>) -> Shared<[T], A> {
        unsafe {
            let (vec_ptr, len, cap, alloc) = v.into_raw_parts_with_alloc();

            let rc_ptr = Self::allocate_for_slice_in(len, &alloc);
            ptr::copy_nonoverlapping(vec_ptr, ptr::addr_of_mut!((*rc_ptr).data) as *mut T, len);

            // Create a `Vec<T, &A>` with length 0, to deallocate the buffer
            // without dropping its contents or the allocator
            let _ = Vec::from_raw_parts_in(vec_ptr, 0, cap, &alloc);

            Self::from_ptr_in(rc_ptr, alloc)
        }
    }
}


impl<'a, B> From<Cow<'a, B>> for Shared<B>
where
    B: ToOwned + ?Sized,
    Shared<B>: From<&'a B> + From<B::Owned>,
{
    /// Creates an atomically reference-counted pointer from a clone-on-write
    /// pointer by copying its content.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// # use std::borrow::Cow;
    /// let cow: Cow<'_, str> = Cow::Borrowed("eggplant");
    /// let shared: Arc<str> = Arc::from(cow);
    /// assert_eq!("eggplant", &shared[..]);
    /// ```
    #[inline]
    fn from(cow: Cow<'a, B>) -> Shared<B> {
        match cow {
            Cow::Borrowed(s) => Shared::from(s),
            Cow::Owned(s) => Shared::from(s),
        }
    }
}


impl From<Shared<str>> for Shared<[u8]> {
    /// Converts an atomically reference-counted string slice into a byte slice.
    ///
    /// # Example
    ///
    /// ```
    /// # use std::sync::Arc;
    /// let string: Arc<str> = Arc::from("eggplant");
    /// let bytes: Arc<[u8]> = Arc::from(string);
    /// assert_eq!("eggplant".as_bytes(), bytes.as_ref());
    /// ```
    #[inline]
    fn from(rc: Shared<str>) -> Self {
        // SAFETY: `str` has the same layout as `[u8]`.
        unsafe { Shared::from_raw(Shared::into_raw(rc) as *const [u8]) }
    }
}


impl<T, A: Allocator, const N: usize> TryFrom<Shared<[T], A>> for Shared<[T; N], A> {
    type Error = Shared<[T], A>;

    fn try_from(boxed_slice: Shared<[T], A>) -> Result<Self, Self::Error> {
        let read_guard = boxed_slice.read();
        if (*read_guard).len() == N {
            let (ptr, alloc) = Shared::into_inner_with_allocator(boxed_slice);
            Ok(unsafe { Shared::from_inner_in(ptr.cast(), alloc) })
        } else {
            Err(boxed_slice)
        }
    }
}



impl<T> FromIterator<T> for Shared<[T]> {
    /// Takes each element in the `Iterator` and collects it into an `Arc<[T]>`.
    ///
    /// # Performance characteristics
    ///
    /// ## The general case
    ///
    /// In the general case, collecting into `Arc<[T]>` is done by first
    /// collecting into a `Vec<T>`. That is, when writing the following:
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// let evens: Arc<[u8]> = (0..10).filter(|&x| x % 2 == 0).collect();
    /// # assert_eq!(&*evens, &[0, 2, 4, 6, 8]);
    /// ```
    ///
    /// this behaves as if we wrote:
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// let evens: Arc<[u8]> = (0..10).filter(|&x| x % 2 == 0)
    ///     .collect::<Vec<_>>() // The first set of allocations happens here.
    ///     .into(); // A second allocation for `Arc<[T]>` happens here.
    /// # assert_eq!(&*evens, &[0, 2, 4, 6, 8]);
    /// ```
    ///
    /// This will allocate as many times as needed for constructing the `Vec<T>`
    /// and then it will allocate once for turning the `Vec<T>` into the `Arc<[T]>`.
    ///
    /// ## Iterators of known length
    ///
    /// When your `Iterator` implements `TrustedLen` and is of an exact size,
    /// a single allocation will be made for the `Arc<[T]>`. For example:
    ///
    /// ```rust
    /// # use std::sync::Arc;
    /// let evens: Arc<[u8]> = (0..10).collect(); // Just a single allocation happens here.
    /// # assert_eq!(&*evens, &*(0..10).collect::<Vec<_>>());
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ToArcSlice::to_arc_slice(iter.into_iter())
    }
}


/// Specialization trait used for collecting into `Arc<[T]>`.
trait ToArcSlice<T>: Iterator<Item = T> + Sized {
    fn to_arc_slice(self) -> Shared<[T]>;
}


impl<T, I: Iterator<Item = T>> ToArcSlice<T> for I {
    default fn to_arc_slice(self) -> Shared<[T]> {
        self.collect::<Vec<T>>().into()
    }
}


impl<T, I: iter::TrustedLen<Item = T>> ToArcSlice<T> for I {
    fn to_arc_slice(self) -> Shared<[T]> {
        // This is the case for a `TrustedLen` iterator.
        let (low, high) = self.size_hint();
        if let Some(high) = high {
            debug_assert_eq!(
                low,
                high,
                "TrustedLen iterator's size hint is not exact: {:?}",
                (low, high)
            );

            unsafe {
                // SAFETY: We need to ensure that the iterator has an exact length and we have.
                Shared::from_iter_exact(self, low)
            }
        } else {
            // TrustedLen contract guarantees that `upper_bound == None` implies an iterator
            // length exceeding `usize::MAX`.
            // The default implementation would collect into a vec which would panic.
            // Thus we panic here immediately without invoking `Vec` code.
            panic!("capacity overflow");
        }
    }
}


impl<T: ?Sized, A: Allocator> Unpin for Shared<T, A> {}

/// Gets the offset within an `ArcRwLockInner` for the payload behind a pointer.
///
/// # Safety
///
/// The pointer must point to (and have valid metadata for) a previously
/// valid instance of T, but the T is allowed to be dropped.
unsafe fn data_offset<T: ?Sized>(ptr: *const T) -> usize {
    // Align the unsized value to the end of the ArcRwLockInner.
    // Because RcBox is repr(C), it will always be the last field in memory.
    // SAFETY: since the only unsized types possible are slices, trait objects,
    // and extern types, the input safety requirement is currently enough to
    // satisfy the requirements of align_of_val_raw; this is an implementation
    // detail of the language that must not be relied upon outside of std.
    unsafe { data_offset_align(core::mem::align_of_val_raw(ptr)) }
}

#[inline]
fn data_offset_align(align: usize) -> usize {
    let layout = Layout::new::<ArcRwLockInner<()>>();
    layout.size() + layout.padding_needed_for(align)
}

/// A unique owning pointer to an [`ArcRwLockInner`] **that does not imply the contents are initialized,**
/// but will deallocate it (without dropping the value) when dropped.
///
/// This is a helper for [`Arc::make_mut()`] to ensure correct cleanup on panic.

struct UniqueArcUninit<T: ?Sized, A: Allocator> {
    ptr: NonNull<ArcRwLockInner<T>>,
    layout_for_value: Layout,
    alloc: Option<A>,
}


impl<T: ?Sized, A: Allocator> UniqueArcUninit<T, A> {
    /// Allocates an ArcRwLockInner with layout suitable to contain `for_value` or a clone of it.
    fn new(for_value: &T, alloc: A) -> UniqueArcUninit<T, A> {
        let layout = Layout::for_value(for_value);
        let ptr = unsafe {
            Shared::allocate_for_layout(
                layout,
                |layout_for_arcinner| alloc.allocate(layout_for_arcinner),
                |mem| mem.with_metadata_of(ptr::from_ref(for_value) as *const ArcRwLockInner<T>),
            )
        };
        Self { ptr: NonNull::new(ptr).unwrap(), layout_for_value: layout, alloc: Some(alloc) }
    }

    /// Returns the pointer to be written into to initialize the [`Arc`].
    fn data_ptr(&mut self) -> *mut T {
        let offset = data_offset_align(self.layout_for_value.align());
        unsafe { self.ptr.as_ptr().byte_add(offset) as *mut T }
    }

    /// Upgrade this into a normal [`Arc`].
    ///
    /// # Safety
    ///
    /// The data must have been initialized (by writing to [`Self::data_ptr()`]).
    unsafe fn into_arc(self) -> Shared<T, A> {
        let mut this = ManuallyDrop::new(self);
        let ptr = this.ptr.as_ptr();
        let alloc = this.alloc.take().unwrap();

        // SAFETY: The pointer is valid as per `UniqueArcUninit::new`, and the caller is responsible
        // for having initialized the data.
        unsafe { Shared::from_ptr_in(ptr, alloc) }
    }
}


impl<T: ?Sized, A: Allocator> Drop for UniqueArcUninit<T, A> {
    fn drop(&mut self) {
        // SAFETY:
        // * new() produced a pointer safe to deallocate.
        // * We own the pointer unless into_arc() was called, which forgets us.
        unsafe {
            self.alloc.take().unwrap().deallocate(
                self.ptr.cast(),
                arcinner_layout_for_value_layout(self.layout_for_value),
            );
        }
    }
}