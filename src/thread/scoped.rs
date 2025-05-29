use super::{Builder, JoinInner, Result, Thread};
use ::alloc::sync::Arc;
use core::fmt;
use core::marker::PhantomData;
use core::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// A scope to spawn scoped threads in.
///
/// See [`scope`] for details.
pub struct Scope<'scope, 'env: 'scope> {
    data: Arc<ScopeData>,
    /// Invariance over 'scope, to make sure 'scope cannot shrink,
    /// which is necessary for soundness.
    ///
    /// Without invariance, this would compile fine but be unsound:
    ///
    /// ```compile_fail,E0373
    /// std::thread::scope(|s| {
    ///     s.spawn(|| {
    ///         let a = String::from("abcd");
    ///         s.spawn(|| println!("{a:?}")); // might run after `a` is dropped
    ///     });
    /// });
    /// ```
    scope: PhantomData<&'scope mut &'scope ()>,
    env: PhantomData<&'env mut &'env ()>,
}

/// An owned permission to join on a scoped thread (block on its termination).
///
/// See [`Scope::spawn`] for details.
pub struct ScopedJoinHandle<'scope, T>(JoinInner<'scope, T>);

pub(super) struct ScopeData {
    num_running_threads: AtomicUsize,
    a_thread_panicked: AtomicBool,
    //main_thread: Thread,
}

impl ScopeData {
    pub(super) fn increment_num_running_threads(&self) {
        // We check for 'overflow' with usize::MAX / 2, to make sure there's no
        // chance it overflows to 0, which would result in unsoundness.
        if self.num_running_threads.fetch_add(1, Ordering::Relaxed) > usize::MAX / 2 {
            // This can only reasonably happen by mem::forget()'ing a lot of ScopedJoinHandles.
            self.overflow();
        }
    }

    #[cold]
    fn overflow(&self) {
        self.decrement_num_running_threads(false);
        panic!("too many running threads in thread scope");
    }

    pub(super) fn decrement_num_running_threads(&self, panic: bool) {
        if panic {
            self.a_thread_panicked.store(true, Ordering::Relaxed);
        }
    }
}

/// Create a scope for spawning scoped threads.
///
/// The function passed to `scope` will be provided a [`Scope`] object,
/// through which scoped threads can be [spawned][`Scope::spawn`].
///
/// Unlike non-scoped threads, scoped threads can borrow non-`'static` data,
/// as the scope guarantees all threads will be joined at the end of the scope.
///
/// All threads spawned within the scope that haven't been manually joined
/// will be automatically joined before this function returns.
///
/// # Panics
///
/// If any of the automatically joined threads panicked, this function will panic.
///
/// If you want to handle panics from spawned threads,
/// [`join`][ScopedJoinHandle::join] them before the end of the scope.
///
/// # Example
///
/// ```
/// use std::thread;
///
/// let mut a = vec![1, 2, 3];
/// let mut x = 0;
///
/// thread::scope(|s| {
///     s.spawn(|| {
///         println!("hello from the first scoped thread");
///         // We can borrow `a` here.
///         dbg!(&a);
///     });
///     s.spawn(|| {
///         println!("hello from the second scoped thread");
///         // We can even mutably borrow `x` here,
///         // because no other threads are using it.
///         x += a[0] + a[2];
///     });
///     println!("hello from the main thread");
/// });
///
/// // After the scope, we can modify and access our variables again:
/// a.push(4);
/// assert_eq!(x, a.len());
/// ```
///
/// # Lifetimes
///
/// Scoped threads involve two lifetimes: `'scope` and `'env`.
///
/// The `'scope` lifetime represents the lifetime of the scope itself.
/// That is: the time during which new scoped threads may be spawned,
/// and also the time during which they might still be running.
/// Once this lifetime ends, all scoped threads are joined.
/// This lifetime starts within the `scope` function, before `f` (the argument to `scope`) starts.
/// It ends after `f` returns and all scoped threads have been joined, but before `scope` returns.
///
/// The `'env` lifetime represents the lifetime of whatever is borrowed by the scoped threads.
/// This lifetime must outlast the call to `scope`, and thus cannot be smaller than `'scope`.
/// It can be as small as the call to `scope`, meaning that anything that outlives this call,
/// such as local variables defined right before the scope, can be borrowed by the scoped threads.
///
/// The `'env: 'scope` bound is part of the definition of the `Scope` type.
#[track_caller]
pub fn scope<'env, F, T>(f: F) -> T
where
    F: for<'scope> FnOnce(&'scope Scope<'scope, 'env>) -> T,
{
    // We put the `ScopeData` into an `Arc` so that other threads can finish their
    // `decrement_num_running_threads` even after this function returns.
    let scope = Scope {
        data: Arc::new(ScopeData {
            num_running_threads: AtomicUsize::new(0),
            //main_thread: current(),
            a_thread_panicked: AtomicBool::new(false),
        }),
        env: PhantomData,
        scope: PhantomData,
    };

    // Run `f`, the scoped function.
    let result = f(&scope);

    // Wait until all the threads are finished. This should always happen first try as we can only drop a thread handle when the thread has finished.
    // TODO: we can possibly detect panics if the num_running_threads > 0 - as this should never happen in our model.
    while scope.data.num_running_threads.load(Ordering::Acquire) != 0 {
        let _ = super::r#yield(super::YieldType::ToAnyThread);
    }

    result
}

impl<'scope> Scope<'scope, '_> {
    /// Spawns a new thread within a scope, returning a [`ScopedJoinHandle`] for it.
    ///
    /// Unlike non-scoped threads, threads spawned with this function may
    /// borrow non-`'static` data from the outside the scope. See [`scope`] for
    /// details.
    ///
    /// The join handle provides a [`join`] method that can be used to join the spawned
    /// thread. If the spawned thread panics, [`join`] will return an [`Err`] containing
    /// the panic payload.
    ///
    /// If the join handle is dropped, the spawned thread will implicitly joined at the
    /// end of the scope. In that case, if the spawned thread panics, [`scope`] will
    /// panic after all threads are joined.
    ///
    /// This call will create a thread using default parameters of [`Builder`].
    /// If you want to specify the stack size or the name of the thread, use
    /// [`Builder::spawn_scoped`] instead.
    ///
    /// # Panics
    ///
    /// Panics if the OS fails to create a thread; use [`Builder::spawn_scoped`]
    /// to recover from such errors.
    ///
    /// [`join`]: ScopedJoinHandle::join
    pub fn spawn<F, T>(&'scope self, f: F) -> ScopedJoinHandle<'scope, T>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        Builder::new()
            .spawn_scoped(self, f)
            .expect("failed to spawn thread")
    }
}

impl Builder {
    /// Spawns a new scoped thread using the settings set through this `Builder`.
    ///
    /// Unlike [`Scope::spawn`], this method yields a [`Result`] to
    /// capture any failure to create the thread at the OS level.
    ///
    /// [`Result`]: crate::result::Result
    ///
    /// # Panics
    ///
    /// Panics if a thread name was set and it contained null bytes.
    ///
    /// # Example
    ///
    /// ```
    /// use std::thread;
    ///
    /// let mut a = vec![1, 2, 3];
    /// let mut x = 0;
    ///
    /// thread::scope(|s| {
    ///     thread::Builder::new()
    ///         .name("first".to_string())
    ///         .spawn_scoped(s, ||
    ///     {
    ///         println!("hello from the {:?} scoped thread", thread::current().name());
    ///         // We can borrow `a` here.
    ///         dbg!(&a);
    ///     })
    ///     .unwrap();
    ///     thread::Builder::new()
    ///         .name("second".to_string())
    ///         .spawn_scoped(s, ||
    ///     {
    ///         println!("hello from the {:?} scoped thread", thread::current().name());
    ///         // We can even mutably borrow `x` here,
    ///         // because no other threads are using it.
    ///         x += a[0] + a[2];
    ///     })
    ///     .unwrap();
    ///     println!("hello from the main thread");
    /// });
    ///
    /// // After the scope, we can modify and access our variables again:
    /// a.push(4);
    /// assert_eq!(x, a.len());
    /// ```
    pub fn spawn_scoped<'scope, 'env, F, T>(
        self,
        scope: &'scope Scope<'scope, 'env>,
        f: F,
    ) -> crate::result::Result<ScopedJoinHandle<'scope, T>>
    where
        F: FnOnce() -> T + Send + 'scope,
        T: Send + 'scope,
    {
        Ok(ScopedJoinHandle(unsafe {
            self.spawn_unchecked_(f, Some(scope.data.clone()))
        }?))
    }
}

impl<T> ScopedJoinHandle<'_, T> {
    /// Extracts a handle to the underlying thread.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread;
    ///
    /// thread::scope(|s| {
    ///     let t = s.spawn(|| {
    ///         println!("hello");
    ///     });
    ///     println!("thread id: {:?}", t.thread().id());
    /// });
    /// ```
    #[must_use]
    pub fn thread(&self) -> &Thread {
        &self.0.thread
    }

    /// Waits for the associated thread to finish.
    ///
    /// This function will return immediately if the associated thread has already finished.
    ///
    /// In terms of [atomic memory orderings], the completion of the associated
    /// thread synchronizes with this function returning.
    /// In other words, all operations performed by that thread
    /// [happen before](https://doc.rust-lang.org/nomicon/atomics.html#data-accesses)
    /// all operations that happen after `join` returns.
    ///
    /// If the associated thread panics, [`Err`] is returned with the panic payload.
    ///
    /// [atomic memory orderings]: core::sync::atomic
    ///
    /// # Examples
    ///
    /// ```
    /// use std::thread;
    ///
    /// thread::scope(|s| {
    ///     let t = s.spawn(|| {
    ///         panic!("oh no");
    ///     });
    ///     assert!(t.join().is_err());
    /// });
    /// ```
    pub fn join(self) -> Result<T> {
        self.0.join()
    }

    /// Checks if the associated thread has finished running its main function.
    ///
    /// `is_finished` supports implementing a non-blocking join operation, by checking
    /// `is_finished`, and calling `join` if it returns `true`. This function does not block. To
    /// block while waiting on the thread to finish, use [`join`][Self::join].
    ///
    /// This might return `true` for a brief moment after the thread's main
    /// function has returned, but before the thread itself has stopped running.
    /// However, once this returns `true`, [`join`][Self::join] can be expected
    /// to return quickly, without blocking for any significant amount of time.
    pub fn is_finished(&self) -> bool {
        Arc::strong_count(&self.0.packet) == 1
    }
}

impl fmt::Debug for Scope<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Scope")
            .field(
                "num_running_threads",
                &self.data.num_running_threads.load(Ordering::Relaxed),
            )
            .field(
                "a_thread_panicked",
                &self.data.a_thread_panicked.load(Ordering::Relaxed),
            )
            //.field("main_thread", &self.data.main_thread)
            .finish_non_exhaustive()
    }
}

impl<T> fmt::Debug for ScopedJoinHandle<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScopedJoinHandle").finish_non_exhaustive()
    }
}
