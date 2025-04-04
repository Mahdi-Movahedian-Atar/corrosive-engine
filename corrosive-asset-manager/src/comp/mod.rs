use corrosive_ecs_core::ecs_core::Res;
use corrosive_ecs_core_macro::Resource;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum AssetValue<T: 'static> {
    Ready(T),
    NotReady(&'static Option<T>),
}
#[derive(Resource)]
pub struct AssetServer<T: 'static> {
    pub values: HashMap<u64, RwLock<AssetValue<T>>>,
    pub references: HashMap<u64, AtomicUsize>,
    pub default: Option<T>,
}
impl<T: 'static> Default for AssetServer<T> {
    fn default() -> Self {
        AssetServer {
            values: HashMap::new(),
            references: HashMap::new(),
            default: None,
        }
    }
}

pub trait AssetTrait<T> {
    fn load(&self, id: u64, asset: impl FnOnce() -> T + Send + 'static) -> Asset<T>;
    fn add(&self, id: u64, asset: T) -> Asset<T>;
    fn add_original(&self, asset: T);
}
impl<T: Send + Sync> AssetTrait<T> for Res<AssetServer<T>> {
    fn load(&self, id: u64, asset: impl FnOnce() -> T + Send + 'static) -> Asset<T> {
        let binding = self.value.clone();

        let (val, ref_count) = unsafe {
            let mut lock = binding.write().unwrap();
            (
                std::mem::transmute(if let Some(t) = lock.references.get(&id) {
                    AtomicUsize::fetch_add(t, 1, Ordering::SeqCst);
                    &lock.values[&id]
                } else {
                    lock.references.insert(id.clone(), AtomicUsize::new(1));
                    let v = std::mem::transmute(&lock.default);
                    lock.values
                        .insert(id.clone(), RwLock::new(AssetValue::NotReady(v)));
                    &lock.values[&id]
                }),
                std::mem::transmute(&lock.references[&id]),
            )
        };
        let new_id = id.clone();
        std::thread::spawn(move || {
            let value = asset();
            if let Some(t) = binding
                .write()
                .expect(format!("Could not add {}.", id).as_str())
                .values
                .get_mut(&new_id)
            {
                *t.write().unwrap() = AssetValue::Ready(value)
            }
        });
        Asset {
            asset_server: self.clone(),
            data: val,
            ref_count,
            id,
        }
    }
    fn add(&self, id: u64, asset: T) -> Asset<T> {
        let mut lock = self.write().unwrap();
        if let Some(t) = lock.references.get(&id) {
            AtomicUsize::fetch_add(t, 1, Ordering::SeqCst);
        } else {
            lock.references.insert(id.clone(), AtomicUsize::new(1));
        }
        lock.values
            .insert(id.clone(), RwLock::new(AssetValue::Ready(asset)));
        let (data, ref_count) = unsafe {
            (
                std::mem::transmute(&lock.values[&id]),
                std::mem::transmute(&lock.references[&id]),
            )
        };
        Asset {
            asset_server: self.clone(),
            data,
            ref_count,
            id,
        }
    }
    fn add_original(&self, asset: T) {
        self.value.write().unwrap().default = Some(asset);
    }
}

pub struct Asset<T: 'static> {
    asset_server: Res<AssetServer<T>>,
    data: &'static RwLock<AssetValue<T>>,
    ref_count: &'static AtomicUsize,
    id: u64,
}
impl<T: 'static> Asset<T> {
    pub fn get<'b>(&self) -> &'b T {
        loop {
            let guard = self.data.read().unwrap();
            match &*self.data.read().unwrap() {
                AssetValue::Ready(t) => {
                    return unsafe { std::mem::transmute(t) };
                }
                AssetValue::NotReady(Some(t)) => return unsafe { std::mem::transmute(t) },
                _ => {}
            }
        }
    }
}
impl<T: 'static> Clone for Asset<T> {
    fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        Asset {
            asset_server: self.asset_server.clone(),
            data: self.data,
            ref_count: self.ref_count,
            id: self.id.clone(),
        }
    }
}
impl<T: 'static> Drop for Asset<T> {
    fn drop(&mut self) {
        if self.ref_count.fetch_sub(1, Ordering::SeqCst) == 0 {
            let mut lock = self.asset_server.write().unwrap();
            lock.references.remove(&self.id);
            lock.values.remove(&self.id);
        }
    }
}

#[derive(Resource)]
pub struct CacheServer<T: 'static> {
    pub values: HashMap<u64, RwLock<AssetValue<T>>>,
    pub references: HashMap<u64, AtomicUsize>,
    pub default: Option<T>,
}
impl<T: 'static> Default for crate::comp::CacheServer<T> {
    fn default() -> Self {
        crate::comp::AssetServer {
            values: HashMap::new(),
            references: HashMap::new(),
            default: None,
        }
    }
}
