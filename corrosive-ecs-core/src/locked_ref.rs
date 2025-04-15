use crate::ecs_core::Reference;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct LockedRef<T> {
    pub value: Arc<RwLock<Reference<T>>>,
}
impl<T> LockedRef<T> {
    pub fn new(value: T) -> LockedRef<T> {
        LockedRef {
            value: Arc::new(RwLock::new(Reference::Some(value))),
        }
    }

    pub fn clone(&self) -> LockedRef<T> {
        LockedRef {
            value: self.value.clone(),
        }
    }

    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, Reference<T>>> {
        self.value.read()
    }
    pub fn e_read(&self, massage: &str) -> RwLockReadGuard<'_, Reference<T>> {
        self.value.read().expect(massage)
    }
    pub fn f_read(&self) -> RwLockReadGuard<'_, Reference<T>> {
        self.value.read().expect("Failed to force read a lock")
    }

    pub fn write(&self) -> LockResult<RwLockWriteGuard<'_, Reference<T>>> {
        self.value.write()
    }
    pub fn e_write(&self, massage: &str) -> RwLockWriteGuard<'_, Reference<T>> {
        self.value.write().expect(massage)
    }
    pub fn f_write(&self) -> RwLockWriteGuard<'_, Reference<T>> {
        self.value.write().expect("Failed to force write a lock")
    }

    pub fn expire(&mut self) {
        *self.value.write().unwrap() = Reference::Expired;
    }
}
