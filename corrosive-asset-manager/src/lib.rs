use std::cell::UnsafeCell;
use std::collections::HashMap;
use std::hash::Hasher;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

pub mod asset_server;

pub fn dynamic_hasher(val: &str) -> u64 {
    const FNV_OFFSET_BASIS: u64 = 14695981039346656037;
    const FNV_PRIME: u64 = 1099511628211;

    let mut hash = FNV_OFFSET_BASIS;
    for byte in val.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}
