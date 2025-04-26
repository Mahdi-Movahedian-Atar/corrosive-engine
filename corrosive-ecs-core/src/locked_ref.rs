use crate::ecs_core::Reference;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Used for a locked reference component.
/// These components can be written and be referenced.
/// Uses `Reference` as value.
/// On removal the values will be expired.
#[derive(Debug)]
pub struct LockedRef<T> {
    pub value: Arc<RwLock<Reference<T>>>,
}
impl<T> LockedRef<T> {
    /// Creates a new LockedRef Components.
    pub fn new(value: T) -> LockedRef<T> {
        LockedRef {
            value: Arc::new(RwLock::new(Reference::Some(value))),
        }
    }
    /// Clones the LockedRef.
    pub fn clone(&self) -> LockedRef<T> {
        LockedRef {
            value: self.value.clone(),
        }
    }
    /// Returns the result of the read lock of a value.
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, Reference<T>>> {
        self.value.read()
    }
    /// Returns the read lock of a value.
    /// Panics with the given massage if the lock is poisoned.
    pub fn e_read(&self, massage: &str) -> RwLockReadGuard<'_, Reference<T>> {
        self.value.read().expect(massage)
    }
    /// Returns the read lock of a value.
    /// Panics if the lock is poisoned.
    pub fn f_read(&self) -> RwLockReadGuard<'_, Reference<T>> {
        self.value.read().expect("Failed to force read a lock")
    }

    /// Returns the result of the write lock of a value.
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, Reference<T>>> {
        self.value.write()
    }
    /// Returns the write lock of a value.
    /// Panics with the given massage if the lock is poisoned.
    pub fn e_write(&self, massage: &str) -> RwLockWriteGuard<'_, Reference<T>> {
        self.value.write().expect(massage)
    }
    /// Returns the write lock of a value.
    /// Panics the lock is poisoned.
    pub fn f_write(&self) -> RwLockWriteGuard<'_, Reference<T>> {
        self.value.write().expect("Failed to force write a lock")
    }

    /// Will expire the value.
    pub fn expire(&mut self) {
        *self.value.write().unwrap() = Reference::Expired;
    }
}
