use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct Locked<T> {
    pub value: RwLock<T>,
}
impl<T> Locked<T> {
    pub fn new(value: T) -> Locked<T> {
        Locked {
            value: RwLock::new(value),
        }
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.value.read()
    }
    pub fn e_read(&self, massage: &str) -> RwLockReadGuard<'_, T> {
        self.value.read().expect(massage)
    }
    pub fn f_read(&self) -> RwLockReadGuard<'_, T> {
        self.value.read().expect("Failed to force read a lock")
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, T>> {
        self.value.write()
    }
    pub fn e_write(&self, massage: &str) -> RwLockWriteGuard<'_, T> {
        self.value.write().expect(massage)
    }
    pub fn f_write(&self) -> RwLockWriteGuard<'_, T> {
        self.value.write().expect("Failed to force write a lock")
    }
}
