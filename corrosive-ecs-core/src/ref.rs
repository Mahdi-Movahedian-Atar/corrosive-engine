use crate::ecs_core::Reference;
use std::sync::{Arc, RwLock, RwLockReadGuard};

#[derive(Debug)]
pub struct Ref<T> {
    pub value: Arc<RwLock<Reference<T>>>,
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
        *self.value.write().unwrap() = Reference::Expired;
    }
}
