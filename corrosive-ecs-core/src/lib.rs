pub mod ecs_core {
    use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

    pub enum Reference<T> {
        Some(T),
        Expired,
    }
    pub struct Locked<T> {
        value: RwLock<T>,
    }
    pub struct Ref<T> {
        value: Arc<Reference<T>>,
    }
    pub struct LockedRef<T> {
        value: Arc<RwLock<Reference<T>>>,
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

    impl<T> Ref<T> {
        pub fn new(value: T) -> Ref<T> {
            Ref {
                value: Arc::new(Reference::Some(value)),
            }
        }

        pub fn clone(&self) -> Ref<T> {
            Ref {
                value: self.value.clone(),
            }
        }
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
}
