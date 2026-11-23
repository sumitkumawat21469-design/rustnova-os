//! Spinlock Synchronization Mechanism.
//!
//! Implements a working spinlock using hardware atomic operations
//! (AtomicBool with Acquire/Release semantics), CPU pause hints, and RAII guard.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use crate::kernel::arch::spin_hint;

pub struct SpinLock<T> {
    locked: AtomicBool,
    acquire_count: AtomicU32,
    contention_spins: AtomicU32,
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            acquire_count: AtomicU32::new(0),
            contention_spins: AtomicU32::new(0),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the spinlock, busy-waiting if currently locked by another context.
    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            self.contention_spins.fetch_add(1, Ordering::Relaxed);
            spin_hint();
        }
        self.acquire_count.fetch_add(1, Ordering::Relaxed);

        SpinLockGuard { lock: self }
    }

    /// Attempts to acquire the lock without spinning.
    pub fn try_lock(&self) -> Option<SpinLockGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            self.acquire_count.fetch_add(1, Ordering::Relaxed);
            Some(SpinLockGuard { lock: self })
        } else {
            None
        }
    }

    pub fn is_locked(&self) -> bool {
        self.locked.load(Ordering::Relaxed)
    }

    pub fn total_acquisitions(&self) -> u32 {
        self.acquire_count.load(Ordering::Relaxed)
    }

    pub fn total_contentions(&self) -> u32 {
        self.contention_spins.load(Ordering::Relaxed)
    }
}

/// RAII Scope Guard for SpinLock. Automatically unlocks upon drop.
pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<'a, T> Deref for SpinLockGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<'a, T> DerefMut for SpinLockGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<'a, T> Drop for SpinLockGuard<'a, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}
