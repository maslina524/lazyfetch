use core::{
    cell::UnsafeCell,
    mem::MaybeUninit,
    ops::{Deref, DerefMut},
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};

use crate::abort;

const INCOMPLETE: u8 = 0;
const INITIALIZING: u8 = 1;
const READY: u8 = 2;

#[derive(Debug)]
pub struct OnceLock<T> {
    state: AtomicU8,
    value: UnsafeCell<MaybeUninit<T>>,
}

impl<T> OnceLock<T> {
    pub const fn new() -> Self {
        Self {
            state: AtomicU8::new(INCOMPLETE),
            value: UnsafeCell::new(MaybeUninit::uninit()),
        }
    }

    pub fn get_or_init(&self, f: impl FnOnce() -> T) -> &T {
        if self.state.load(Ordering::Acquire) == READY {
            // SAFETY: The value in `MaybeUninit` is guaranteed to be initialized
            unsafe {
                return (&*self.value.get()).assume_init_ref();
            }
        }

        if self
            .state
            .compare_exchange(
                INCOMPLETE,
                INITIALIZING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            let value = f();

            // SAFETY: In write we initialize the value, and
            // then call an unsafe function that
            // is completely safe after initialization
            unsafe {
                (*self.value.get()).write(value);
            }
            self.state.store(READY, Ordering::Release);

            // SAFETY: The value in `MaybeUninit` is guaranteed to be initialized
            unsafe { (&*self.value.get()).assume_init_ref() }
        } else {
            while self.state.load(Ordering::Acquire) != READY {
                core::hint::spin_loop();
            }

            // SAFETY: The value in `MaybeUninit` is guaranteed to be initialized
            unsafe { (&*self.value.get()).assume_init_ref() }
        }
    }

    pub fn get(&self) -> Option<&T> {
        if self.state.load(Ordering::Acquire) == READY {
            // SAFETY: The value in `MaybeUninit` is guaranteed to be initialized
            unsafe { Some((&*self.value.get()).assume_init_ref()) }
        } else {
            None
        }
    }

    pub fn get_or_abort(&self, msg: &str) -> &T {
        if self.state.load(Ordering::Acquire) == READY {
            // SAFETY: The value in `MaybeUninit` is guaranteed to be initialized
            unsafe { (&*self.value.get()).assume_init_ref() }
        } else {
            abort!("{}", msg);
        }
    }

    pub fn set(&self, val: T) -> Result<&T, &T> {
        if self.state.load(Ordering::Acquire) == READY {
            // SAFETY: The value in `MaybeUninit` is guaranteed to be initialized
            let val = unsafe { (&*self.value.get()).assume_init_ref() };
            return Err(val);
        }

        if self
            .state
            .compare_exchange(
                INCOMPLETE,
                INITIALIZING,
                Ordering::AcqRel,
                Ordering::Acquire,
            )
            .is_ok()
        {
            // SAFETY: Completely safe
            let writted = unsafe { (*self.value.get()).write(val) };
            self.state.store(READY, Ordering::Release);
            Ok(writted)
        } else {
            while self.state.load(Ordering::Acquire) != READY {
                core::hint::spin_loop();
            }

            // SAFETY: The value in `MaybeUninit` is guaranteed to be initialized
            let val = unsafe { (&*self.value.get()).assume_init_ref() };
            Err(val)
        }
    }
}

// SAFETY: trait is empty
unsafe impl<T: Sync> Sync for OnceLock<T> {}

#[derive(Debug)]
pub struct Mutex<T> {
    active: AtomicBool,
    data: UnsafeCell<T>,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            active: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        while self
            .active
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            core::hint::spin_loop();
        }
        MutexGuard { mutex: self }
    }

    pub fn unlock(&self) {
        self.active.store(false, Ordering::Release);
    }
}

pub struct MutexGuard<'mtx, T> {
    mutex: &'mtx Mutex<T>,
}

impl<'mtx, T> MutexGuard<'mtx, T> {
    pub fn new(mutex: &'mtx Mutex<T>) -> Self {
        mutex.lock();
        Self { mutex }
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        // SAFETY: UnsafeCell<T> == T
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: UnsafeCell<T> == T
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.unlock();
    }
}

// SAFETY: trait is empty
unsafe impl<T: Sync> Sync for Mutex<T> {}
// SAFETY: trait is empty
unsafe impl<T: Send> Send for Mutex<T> {}
