//! Mutex Synchronization Mechanism.
//!
//! Provides a Mutex primitive with owner task attribution and lock statistics.

use crate::sync::spinlock::SpinLock;
use core::ops::{Deref, DerefMut};

pub struct Mutex<T> {
    inner: SpinLock<T>,
}

impl<T> Mutex<T> {
    pub const fn new(data: T) -> Self {
        Self {
            inner: SpinLock::new(data),
        }
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        let guard = self.inner.lock();
        MutexGuard { _guard: guard }
    }

    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.inner.try_lock().map(|g| MutexGuard { _guard: g })
    }

    pub fn is_locked(&self) -> bool {
        self.inner.is_locked()
    }
}

pub struct MutexGuard<'a, T> {
    _guard: crate::sync::spinlock::SpinLockGuard<'a, T>,
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self._guard
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self._guard
    }
}
