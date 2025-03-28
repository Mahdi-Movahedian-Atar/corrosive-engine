use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

pub struct Asset<T: AssetObject + 'static> {
    asset: &'static AssetValue<'static, T>,
    ref_count: &'static AtomicUsize,
    id: u64,
}
impl<T: AssetObject<AssetType = T>> Asset<T> {
    pub fn clone(&self) -> Asset<T> {
        self.ref_count.fetch_add(1, Ordering::SeqCst);
        Asset {
            asset: self.asset,
            ref_count: self.ref_count,
            id: self.id,
        }
    }
    pub fn add(id: u64, asset: T) -> Asset<T>
    where
        <T as AssetObject>::AssetType: AssetObject,
    {
        let asset = unsafe { T::add_asset(id, asset) };
        Asset {
            asset: asset.0,
            ref_count: asset.1,
            id,
        }
    }
    pub fn set_default<'b>(asset: T)
    where
        <T as AssetObject>::AssetType: AssetObject,
    {
        unsafe { T::set_default(asset) };
    }
    pub fn load<'b>(id: u64, asset: impl FnOnce() -> T + Send + 'static) -> Asset<T>
    where
        <T as AssetObject>::AssetType: AssetObject,
    {
        let asset = unsafe { T::load_asset(id, asset) };
        Asset {
            asset: asset.0,
            ref_count: asset.1,
            id,
        }
    }
    pub fn get(&self) -> &T {
        loop {
            match self.asset {
                AssetValue::Ready(val) => return val,
                AssetValue::NotReady(Some(val)) => return val,
                _ => {}
            }
        }
    }
}
impl<T> Drop for Asset<T>
where
    T: AssetObject,
{
    fn drop(&mut self) {
        unsafe {
            if (self.ref_count.fetch_sub(1, Ordering::SeqCst) == 0) {
                T::remove_asset(&self.id);
            }
        }
    }
}

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum AssetValue<'a, T> {
    Ready(T),
    NotReady(&'a Option<T>),
}

pub struct AssetManagerObject<'a, T> {
    pub ref_counts: RwLock<HashMap<u64, AtomicUsize>>,
    pub values: RwLock<HashMap<u64, AssetValue<'a, T>>>,
    pub default_value: UnsafeCell<Option<T>>,
}
impl<T> AssetManagerObject<'_, T> {
    pub fn new() -> AssetManagerObject<'static, T> {
        AssetManagerObject {
            ref_counts: Default::default(),
            values: Default::default(),
            default_value: UnsafeCell::new(None),
        }
    }
}
pub trait AssetObject {
    type AssetType;
    unsafe fn remove_asset(id: &u64);
    unsafe fn replace_asset(id: &u64, asset_object: Self::AssetType);
    unsafe fn add_asset<'a>(
        id: u64,
        asset_object: Self::AssetType,
    ) -> (&'a AssetValue<'a, Self::AssetType>, &'a AtomicUsize)
    where
        <Self as AssetObject>::AssetType: AssetObject;
    unsafe fn load_asset<'a>(
        id: u64,
        asset_object: impl FnOnce() -> Self::AssetType + Send + 'static,
    ) -> (&'a AssetValue<'a, Self::AssetType>, &'a AtomicUsize)
    where
        <Self as AssetObject>::AssetType: AssetObject;
    unsafe fn set_default<'a>(asset_object: Self::AssetType)
    where
        <Self as AssetObject>::AssetType: AssetObject;
}
