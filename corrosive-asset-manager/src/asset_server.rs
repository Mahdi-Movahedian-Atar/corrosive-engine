use crate::dynamic_hasher;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

pub trait AssetObject {
    fn get_server() -> &'static Mutex<AssetServer<Self>>
    where
        Self: Sized;
}
pub trait AssetFile {
    fn load_file(file: &str) -> Result<Self, Box<dyn Error>>
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
    pub fn add<'a>(
        id: u64,
        asset: impl FnOnce() -> Result<T, Box<dyn Error>> + Send + 'static,
    ) -> Asset<T> {
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
                    match asset() {
                        Ok(v) => {
                            if let Some(t) = T::get_server()
                                .lock()
                                .expect(format!("Could not add {}.", id).as_str())
                                .values
                                .get_mut(&new_id)
                            {
                                *t.write().unwrap() = AssetValue::Ready(v)
                            }
                        }
                        Err(v) => {
                            println!("{:?}", v)
                        }
                    };
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
    pub fn add_sync<'a>(id: u64, asset: impl FnOnce() -> Result<T, Box<dyn Error>>) -> Asset<T> {
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

                match asset() {
                    Ok(v) => {
                        if let Some(t) = lock.values.get_mut(&id.clone()) {
                            *t.write().unwrap() = AssetValue::Ready(v)
                        }
                    }
                    Err(v) => {
                        println!("{:?}", v)
                    }
                };
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
    pub fn get<'a>(id: u64) -> Option<Asset<T>> {
        let binding = T::get_server();
        let (val, ref_count) = unsafe {
            let lock = binding.lock().unwrap();
            (
                std::mem::transmute(&*lock.values.get(&id)?),
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
    pub fn get_or_add(
        id: u64,
        asset: impl FnOnce() -> Result<T, Box<dyn Error>> + Send + 'static,
    ) -> Asset<T> {
        if let Some(t) = AssetServer::get(id) {
            return t;
        };
        AssetServer::add(id, asset)
    }
    pub fn get_or_add_sync(
        id: u64,
        asset: impl FnOnce() -> Result<T, Box<dyn Error>> + Send + 'static,
    ) -> Asset<T> {
        if let Some(t) = AssetServer::get(id) {
            return t;
        };
        AssetServer::add_sync(id, asset)
    }
}
impl<T: Send + Sync + AssetObject + AssetFile> AssetServer<T> {
    pub fn load(file_path: &str) -> Asset<T> {
        let new = file_path.to_owned();
        AssetServer::add(dynamic_hasher(file_path), move || {
            #[cfg(debug_assertions)]
            {
                return T::load_file(
                    format!(
                        "{}/{}",
                        env::var("CORROSIVE_APP_ROOT").unwrap_or(".".to_string()),
                        new.as_str()
                    )
                    .as_str(),
                );
            }
            #[cfg(not(debug_assertions))]
            {
                T::load_file(new.as_str())
            }
        })
    }
    pub fn load_sync(file_path: &str) -> Asset<T> {
        AssetServer::add_sync(dynamic_hasher(file_path), || {
            #[cfg(debug_assertions)]
            {
                return T::load_file(
                    format!(
                        "{}/{}",
                        env::var("CORROSIVE_APP_ROOT").unwrap_or(".".to_string()),
                        file_path
                    )
                    .as_str(),
                );
            }
            #[cfg(not(debug_assertions))]
            {
                T::load_file(new.as_str())
            }
        })
    }
    pub fn load_default(file_path: &str) {
        #[cfg(debug_assertions)]
        {
            match T::load_file(
                format!(
                    "{}/{}",
                    env::var("CORROSIVE_APP_ROOT").unwrap_or(".".to_string()),
                    file_path
                )
                .as_str(),
            ) {
                Ok(v) => AssetServer::add_default(v),
                Err(e) => {
                    println!("{:?}", e)
                }
            }
            return;
        }
        #[cfg(not(debug_assertions))]
        {
            match T::load_file(file_path) {
                Ok(v) => AssetServer::add_default(v),
                Err(e) => {
                    println!("{:?}", e)
                }
            }
        }
    }
}

pub struct Asset<T: 'static> {
    asset_server: &'static Mutex<AssetServer<T>>,
    data: &'static RwLock<AssetValue<T>>,
    ref_count: &'static AtomicUsize,
    id: u64,
}
impl<T: 'static> Asset<T> {
    pub fn get<'a>(&self) -> &'a T {
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
            asset_server: self.asset_server,
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
