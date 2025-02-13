#[cfg(feature = "build")]
pub mod build;

#[cfg(feature = "core")]
pub mod ecs_core {
    use crate::ecs_core::Reference::Expired;
    use bus::{Bus, BusReader};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::mpsc::{Receiver, Sender};
    use std::sync::{mpsc, Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};
    use std::thread;
    use std::thread::Scope;

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

    pub trait EngineArch<T> {
        fn remove(&self, index: usize);
        fn len(&self) -> usize;
        fn get_item(&self, index: usize) -> Option<T>;
    }

    pub struct Arch<'a, T> {
        pub arch: &'a dyn EngineArch<T>,
        pub index: usize,
    }

    impl<'a, T> Arch<'a, T> {
        pub fn new(arch: &'a dyn EngineArch<T>) -> Self {
            Arch { arch, index: 0 }
        }

        pub fn remove(&self, index: usize) {
            self.arch.remove(index);
        }

        pub fn len(&self) -> usize {
            self.arch.len()
        }

        pub fn iter(&self) -> ArchIterator<'_, T> {
            ArchIterator {
                arch: self.arch,
                index: self.index,
            }
        }
    }

    pub struct ArchIterator<'a, T> {
        pub arch: &'a dyn EngineArch<T>,
        pub index: usize,
    }
    impl<'a, T> Iterator for ArchIterator<'a, T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            let result = self.arch.get_item(self.index);
            self.index += 1;
            result
        }
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

    pub struct Trigger(Bus<()>);
    pub struct TriggerReader(BusReader<()>);
    impl Trigger {
        pub fn new() -> Trigger {
            Trigger(Bus::new(1))
        }

        pub fn add_trigger(&mut self) -> TriggerReader {
            TriggerReader(self.0.add_rx())
        }

        pub fn trigger(&mut self) {
            self.0.broadcast(())
        }
    }
    impl TriggerReader {
        pub fn read(&mut self, message: &str) {
            self.0.recv().expect(message);
        }
    }

    #[macro_export]
    macro_rules! add_entity {
    ($($x:ty = $y:expr),+ ) => {
        println!("Warning: funtions must have the [tast] attrabute!");
    };
        [$x:ty , ( $y:expr ) ] => {
        println!("Warning: funtions must have the [tast] attrabute!");
    };
}
    #[macro_export]
    macro_rules! signal {
        ($x:literal) => {
            println!("Warning: funtions must have the [tast] attrabute!");
        };
    }

    #[macro_export]
    macro_rules! reset {
        () => {
            println!("Warning: funtions must have the [tast] attrabute!");
        };
    }
}
