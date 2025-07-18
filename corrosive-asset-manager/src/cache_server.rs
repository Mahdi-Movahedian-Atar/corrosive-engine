use crate::asset_server::{Asset, AssetServer, AssetValue};
use crate::dynamic_hasher;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{LazyLock, Mutex, RwLock};

pub trait CacheObject {
    fn get_server() -> &'static Mutex<CacheServer<Self>>
    where
        Self: Sized;
}
pub trait CacheFile {
    fn load_file(file: &str) -> Result<Self, Box<dyn Error>>
    where
        Self: Sized;
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum CacheValue<T: 'static> {
    Ready(T),
    NotReady(&'static Option<T>),
}
pub struct CacheServerObject<T: 'static> {
    pub server: LazyLock<Mutex<CacheServer<T>>>,
}
pub struct CacheServer<T: 'static> {
    pub values: HashMap<u64, RwLock<CacheValue<T>>>,
    pub default: Option<T>,
}
impl<T: 'static> Default for CacheServer<T> {
    fn default() -> Self {
        CacheServer {
            values: HashMap::new(),
            default: None,
        }
    }
}
impl<T: Send + Sync + CacheObject> CacheServer<T> {
    pub fn add<'a>(
        id: u64,
        asset: impl FnOnce() -> Result<T, Box<dyn Error>> + Send + 'static,
    ) -> Cache<T> {
        let binding = T::get_server();

        let (val) = unsafe {
            let mut lock = binding.lock().unwrap();

            if lock.values.get(&id).is_none() {
                let v = std::mem::transmute(&lock.default);
                lock.values
                    .insert(id.clone(), RwLock::new(CacheValue::NotReady(v)));
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
                                *t.write().unwrap() = CacheValue::Ready(v)
                            }
                        }
                        Err(v) => {
                            println!("{:?}", v)
                        }
                    };
                });
            }
            std::mem::transmute(&lock.values[&id])
        };
        Cache { data: val, id }
    }
    pub fn add_sync<'a>(id: u64, asset: impl FnOnce() -> Result<T, Box<dyn Error>>) -> Cache<T> {
        let binding = T::get_server();

        let (val) = unsafe {
            let mut lock = binding.lock().unwrap();

            if lock.values.get(&id).is_none() {
                let v = std::mem::transmute(&lock.default);
                lock.values
                    .insert(id.clone(), RwLock::new(CacheValue::NotReady(v)));
                let new_id = id.clone();
                match asset() {
                    Ok(v) => {
                        if let Some(t) = T::get_server()
                            .lock()
                            .expect(format!("Could not add {}.", id).as_str())
                            .values
                            .get_mut(&new_id)
                        {
                            *t.write().unwrap() = CacheValue::Ready(v)
                        }
                    }
                    Err(v) => {
                        println!("{:?}", v)
                    }
                };
            }
            std::mem::transmute(&lock.values[&id])
        };
        Cache { data: val, id }
    }
    pub fn add_default(asset: T) {
        T::get_server().lock().unwrap().default = Some(asset);
    }
    pub fn get<'a>(id: u64) -> Option<Cache<T>> {
        let binding = T::get_server();
        let val = unsafe {
            let lock = binding.lock().unwrap();
            std::mem::transmute(&*lock.values.get(&id)?)
        };
        Some(Cache { data: val, id })
    }
    pub fn get_or_add(
        id: u64,
        asset: impl FnOnce() -> Result<T, Box<dyn Error>> + Send + 'static,
    ) -> Cache<T> {
        if let Some(t) = CacheServer::get(id) {
            return t;
        }
        CacheServer::add(id, asset)
    }
    pub fn get_or_add_sync(
        id: u64,
        asset: impl FnOnce() -> Result<T, Box<dyn Error>> + Send + 'static,
    ) -> Cache<T> {
        if let Some(t) = CacheServer::get(id) {
            return t;
        }
        CacheServer::add_sync(id, asset)
    }
}
impl<T: Send + Sync + CacheObject + CacheFile> CacheServer<T> {
    pub fn load(file_path: &str) -> Cache<T> {
        let new = file_path.to_owned();
        CacheServer::add(dynamic_hasher(file_path), move || {
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
    pub fn load_sync(file_path: &str) -> Cache<T> {
        CacheServer::add_sync(dynamic_hasher(file_path), || {
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
                Ok(v) => CacheServer::add_default(v),
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

pub struct Cache<T: 'static> {
    data: &'static RwLock<CacheValue<T>>,
    id: u64,
}
impl<T: 'static> Cache<T> {
    pub fn get<'a>(&self) -> &'a T {
        loop {
            match &*self.data.read().unwrap() {
                CacheValue::Ready(t) => {
                    return unsafe { std::mem::transmute(t) };
                }
                CacheValue::NotReady(Some(t)) => return unsafe { std::mem::transmute(t) },
                _ => {}
            }
        }
    }
}
impl<T: 'static> Clone for Cache<T> {
    fn clone(&self) -> Self {
        Cache {
            data: self.data,
            id: self.id.clone(),
        }
    }
}
