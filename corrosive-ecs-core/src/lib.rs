#[cfg(feature = "build")]
pub mod build;

#[cfg(feature = "core")]
pub mod ecs_core {
    use crate::ecs_core::Reference::Expired;
    use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

    pub enum Reference<T> {
        Some(T),
        Expired,
    }
    pub struct Locked<T> {
        pub value: RwLock<T>,
    }
    pub struct Ref<T> {
        pub value: Arc<RwLock<Reference<T>>>,
    }
    pub struct LockedRef<T> {
        pub value: Arc<RwLock<Reference<T>>>,
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
                value: Arc::new(RwLock::new(Reference::Some(value))),
            }
        }

        pub fn clone(&self) -> Ref<T> {
            Ref {
                value: self.value.clone(),
            }
        }

        pub fn get(&self) -> RwLockReadGuard<'_, Reference<T>> {
            self.value.read().unwrap()
        }

        pub fn expire(&mut self) {
            *self.value.write().unwrap() = Expired;
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

    pub struct Arch<T> {
        v: Vec<T>,
    }
    pub struct Res<'a, T> {
        pub value: &'a RwLock<T>,
    }
    impl<T> Res<'_, T> {
        pub fn new(value: &RwLock<T>) -> Res<T> {
            Res { value }
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
    pub struct State<'a, T> {
        pub value: &'a RwLock<T>,
    }
    impl<T> State<'_, T> {
        pub fn new(value: &RwLock<T>) -> State<T> {
            State { value }
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
    pub type DeltaTime<'a> = &'a f64;

    #[macro_export]
    macro_rules! add_entity {
    ($($x:ty = $y:expr),+ ) => {
        eprintln!("Warning: funtions must have the [tast] attrabute!");
    };
        [$x:ty , ( $y:expr ) ] => {
        eprintln!("Warning: funtions must have the [tast] attrabute!");
    };
}
    #[macro_export]
    macro_rules! signal {
        ($x:literal) => {
            eprintln!("Warning: funtions must have the [tast] attrabute!");
        };
    }

    #[macro_export]
    macro_rules! reset {
        () => {
            eprintln!("Warning: funtions must have the [tast] attrabute!");
        };
    }
}
