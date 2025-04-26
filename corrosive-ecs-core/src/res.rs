use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Used for singleton objects.
/// Can be used as a task input.
/// T Must implement the `Default` trait.
#[derive(Debug)]
pub struct Res<T: Default> {
    pub value: Arc<RwLock<T>>,
}
impl<T: Default> Res<T> {
    /// Used by engine to create a Resource.
    pub fn new(value: T) -> Res<T> {
        Res {
            value: Arc::new(RwLock::new(value)),
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

    ///Clones a resource.
    pub fn clone(&self) -> Res<T> {
        Res {
            value: self.value.clone(),
        }
    }
}
