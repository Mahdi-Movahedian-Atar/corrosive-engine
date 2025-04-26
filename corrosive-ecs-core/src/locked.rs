use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Used for a locked component.
/// These components can be written to but cannot be referenced.
#[derive(Debug)]
pub struct Locked<T> {
    pub value: RwLock<T>,
}
impl<T> Locked<T> {
    /// Creates a new Locked Components.
    pub fn new(value: T) -> Locked<T> {
        Locked {
            value: RwLock::new(value),
        }
    }
    /// Returns the result of the read lock of a value.
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.value.read()
    }
    /// Returns the read lock of a value.
    /// Panics with the given massage if the lock is poisoned.
    pub fn e_read(&self, massage: &str) -> RwLockReadGuard<'_, T> {
        self.value.read().expect(massage)
    }
    /// Returns the read lock of a value.
    /// Panics if the lock is poisoned.
    pub fn f_read(&self) -> RwLockReadGuard<'_, T> {
        self.value.read().expect("Failed to force read a lock")
    }

    /// Returns the result of the write lock of a value.
    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.value.write()
    }
    /// Returns the write lock of a value.
    /// Panics with the given massage if the lock is poisoned.
    pub fn e_write(&self, massage: &str) -> RwLockWriteGuard<'_, T> {
        self.value.write().expect(massage)
    }
    /// Returns the write lock of a value.
    /// Panics the lock is poisoned.
    pub fn f_write(&self) -> RwLockWriteGuard<'_, T> {
        self.value.write().expect("Failed to force write a lock")
    }
}
