use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

pub trait AssetObject {
    fn get_server() -> &'static Mutex<AssetServer<Self>>
    where
        Self: Sized;
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum AssetValue<T: 'static> {
    Ready(T),
    NotReady(&'static Option<T>),
}
pub struct AssetServerObject<T: 'static> {
    pub server: LazyLock<Mutex<AssetServer<T>>>,
}
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
impl<T: Send + Sync + AssetObject> AssetServer<T> {
    pub fn add(id: u64, asset: impl FnOnce() -> T + Send + 'static) -> Asset<T> {
        let binding = T::get_server();

        let (val, ref_count) = unsafe {
            let mut lock = binding.lock().unwrap();

            if let Some(t) = lock.references.get(&id) {
                AtomicUsize::fetch_add(t, 1, Ordering::SeqCst);
            } else {
                lock.references.insert(id.clone(), AtomicUsize::new(1));
                let v = std::mem::transmute(&lock.default);
                lock.values
                    .insert(id.clone(), RwLock::new(AssetValue::NotReady(v)));

                let new_id = id.clone();
                std::thread::spawn(move || {
                    let value = asset();
                    if let Some(t) = T::get_server()
                        .lock()
                        .expect(format!("Could not add {}.", id).as_str())
                        .values
                        .get_mut(&new_id)
                    {
                        *t.write().unwrap() = AssetValue::Ready(value)
                    }
                });
            };
            (
                std::mem::transmute(&lock.values[&id]),
                std::mem::transmute(&lock.references[&id]),
            )
        };
        Asset {
            asset_server: binding,
            data: val,
            ref_count,
            id,
        }
    }
    pub fn add_sync(id: u64, asset: impl FnOnce() -> T) -> Asset<T> {
        let binding = T::get_server();

        let (val, ref_count) = unsafe {
            let mut lock = binding.lock().unwrap();

            if let Some(t) = lock.references.get(&id) {
                AtomicUsize::fetch_add(t, 1, Ordering::SeqCst);
            } else {
                lock.references.insert(id.clone(), AtomicUsize::new(1));
                let v = std::mem::transmute(&lock.default);
                lock.values
                    .insert(id.clone(), RwLock::new(AssetValue::NotReady(v)));

                let value = asset();
                if let Some(t) = T::get_server()
                    .lock()
                    .expect(format!("Could not add {}.", id).as_str())
                    .values
                    .get_mut(&id.clone())
                {
                    *t.write().unwrap() = AssetValue::Ready(value)
                }
            };
            (
                std::mem::transmute(&lock.values[&id]),
                std::mem::transmute(&lock.references[&id]),
            )
        };
        Asset {
            asset_server: binding,
            data: val,
            ref_count,
            id,
        }
    }
    pub fn add_default(asset: T) {
        T::get_server().lock().unwrap().default = Some(asset);
    }
    pub fn get(id: u64) -> Option<Asset<T>> {
        let binding = T::get_server();
        let (val, ref_count) = unsafe {
            let lock = binding.lock().unwrap();
            (
                std::mem::transmute(&lock.values.get(&id)?),
                std::mem::transmute(&lock.references[&id]),
            )
        };
        Some(Asset {
            asset_server: binding,
            data: val,
            ref_count,
            id,
        })
    }
}

pub struct Asset<T: 'static> {
    asset_server: &'static Mutex<AssetServer<T>>,
    data: &'static RwLock<AssetValue<T>>,
    ref_count: &'static AtomicUsize,
    id: u64,
}
impl<T: 'static> Asset<T> {
    pub fn get<'b>(&self) -> &'b T {
        loop {
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
            let mut lock = self.asset_server.lock().unwrap();
            lock.references.remove(&self.id);
            lock.values.remove(&self.id);
        }
    }
}
