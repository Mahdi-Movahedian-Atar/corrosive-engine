use crate::ecs_core::Reference;
use std::sync::{Arc, RwLock, RwLockReadGuard};

/// Used for a referencable component.
/// These components can be referenced but cannot be written to.
#[derive(Debug)]
pub struct Ref<T> {
    value: Arc<RwLock<Reference<T>>>,
}
impl<T> Ref<T> {
    ///Create a new Ref
    pub fn new(value: T) -> Ref<T> {
        Ref {
            value: Arc::new(RwLock::new(Reference::Some(value))),
        }
    }
    /// Clones the Ref.
    pub fn clone(&self) -> Ref<T> {
        Ref {
            value: self.value.clone(),
        }
    }
    /// Returns the result of the read lock of a value.
    pub fn get(&self) -> RwLockReadGuard<'_, Reference<T>> {
        self.value.read().unwrap()
    }
    /// Expires the Ref.
    pub fn expire(&mut self) {
        *self.value.write().unwrap() = Reference::Expired;
    }
}
