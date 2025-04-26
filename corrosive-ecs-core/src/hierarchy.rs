use crate::ecs_core::Reference;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

/// Used for a hierarchy capable components.
/// `T` must implement SharedBehavior trait.
/// May only be crated via the associated hierarchy object.
/// Uses `Reference` as value.
/// On removal or extraction, the member is removed from the hierarchy.
#[derive(Debug)]
pub struct Member<T: SharedBehavior + 'static> {
    pub id: u64,
    pub hierarchy: Hierarchy<T>,
    pub value: Arc<RwLock<Reference<T>>>,
}
/// SharedBehavior of a hierarchy capable components.
impl<T: SharedBehavior> Member<T> {
    /// Creates a new member.
    pub fn new(value: T, hierarchy: &Hierarchy<T>) -> Member<T> {
        hierarchy.new_entry(value)
    }
    /// Clones a member.
    pub fn clone(&self) -> Member<T> {
        Member {
            id: self.id.clone(),
            hierarchy: self.hierarchy.clone(),
            value: self.value.clone(),
        }
    }
    /// Returns the result of the read lock of a value.
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, Reference<T>>> {
        self.value.read()
    }
    /// Returns the read lock of a value.
    /// Panics with the given massage if the lock is poisoned.
    pub fn e_read(&self, massage: &str) -> RwLockReadGuard<'_, Reference<T>> {
        self.value.read().expect(massage)
    }
    /// Returns the read lock of a value.
    /// Panics if the lock is poisoned.
    pub fn f_read(&self) -> RwLockReadGuard<'_, Reference<T>> {
        self.value.read().expect("Failed to force read a lock")
    }

    /// Returns the result of the write lock of a value.
    /// will not trigger the shared behavior of the hierarchy.
    pub fn dry_write(&self) -> LockResult<RwLockWriteGuard<'_, Reference<T>>> {
        self.value.write()
    }
    /// Returns the write lock of a value.
    /// Panics with the given massage if the lock is poisoned.
    /// will not trigger the shared behavior of the hierarchy.
    pub fn dry_e_write(&self, massage: &str) -> RwLockWriteGuard<'_, Reference<T>> {
        self.value.write().expect(massage)
    }
    /// Returns the write lock of a value.
    /// Panics the lock is poisoned.
    /// will not trigger the shared behavior of the hierarchy.
    pub fn dry_f_write(&self) -> RwLockWriteGuard<'_, Reference<T>> {
        self.value.write().expect("Failed to force write a lock")
    }
    /// require a closure with the result of write lock as an input.
    /// will trigger the shared behavior of the hierarchy.
    pub fn write(&self, mut func: impl FnMut(LockResult<RwLockWriteGuard<'_, Reference<T>>>)) {
        func(self.value.write());
        self.hierarchy.shared_behavior(&self.id);
    }
    /// require a closure with the write lock as an input.
    /// will trigger the shared behavior of the hierarchy.
    /// Panics with the given massage if the lock is poisoned.
    pub fn e_write(&self, mut func: impl FnMut(RwLockWriteGuard<'_, Reference<T>>), massage: &str) {
        func(self.value.write().expect(massage));
        self.hierarchy.shared_behavior(&self.id);
    }
    /// require a closure with the write lock as an input.
    /// will trigger the shared behavior of the hierarchy.
    /// Panics if the lock is poisoned.
    pub fn f_write(&self, mut func: impl FnMut(RwLockWriteGuard<'_, Reference<T>>)) {
        func(self.value.write().expect("Failed to force write a lock"));
        self.hierarchy.shared_behavior(&self.id);
    }
    pub fn get_children(&self) -> Vec<Member<T>> {
        self.hierarchy.get_children(&self.id)
    }
    pub fn get_parent(&self) -> Option<Member<T>> {
        self.hierarchy.get_parent(&self.id)
    }
    pub fn remove_child(&self, child: &Member<T>) {
        self.hierarchy.remove_child(&self.id, &child.id);
    }
    pub fn remove_child_by_id(&self, child: &u64) {
        self.hierarchy.remove_child(&self.id, &child);
    }
    pub fn remove_children(&self) {
        self.hierarchy.remove_children(&self.id)
    }
    pub fn remove_parent(&self) -> Option<u64> {
        self.hierarchy.remove_parent(&self.id)
    }
    /// Will expire tha value and removes it from the hierarchy.
    pub fn expire(&mut self) {
        self.hierarchy.remove_entry(&self.id)
    }
}
/// SharedBehavior of a hierarchy capable components.
/// All components that wrap in `Member<T>` and `Hierarchy<T>` must implement this trait.
pub trait SharedBehavior {
    /// Will be called when a hierarchy capable component is made a child of another.
    fn shaded_add_behavior(&mut self, parent: &Self);
    /// Will be called when a hierarchy capable component is removed from a parent.
    fn shaded_remove_behavior(&mut self);
}
/// The data of a hierarchy.
#[derive(Default, Debug)]
pub struct HierarchyData<T> {
    latest_id: u64,
    discarded_id: Vec<u64>,
    nodes: HashMap<u64, Arc<RwLock<Reference<T>>>>,
    dependencies: HashMap<u64, u64>,
    dependents: HashMap<u64, HashSet<u64>>,
}
/// Can be used as a task input.
/// Will be used to create a hierarchy capable component.
/// T must implement `SharedBehavior` trait.
#[derive(Default, Debug)]
pub struct Hierarchy<T: SharedBehavior> {
    data: Arc<RwLock<HierarchyData<T>>>,
}
impl<T: SharedBehavior> Hierarchy<T> {
    pub fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }
    pub fn new_entry(&self, value: T) -> Member<T> {
        let mut lock = self.data.write().unwrap();
        let id = if let Some(v) = lock.discarded_id.pop() {
            v
        } else {
            lock.latest_id += 1;
            lock.latest_id - 1
        };
        Member {
            id: id,
            hierarchy: self.clone(),
            value: Arc::new(RwLock::new(Reference::Some(value))),
        }
    }
    pub fn get_parent(&self, child: &u64) -> Option<Member<T>> {
        let lock = self.data.read().unwrap();
        let i = lock.dependencies.get(child)?;
        let data = lock.nodes.get(i).clone()?;
        Some(Member {
            id: i.clone(),
            hierarchy: self.clone(),
            value: data.clone(),
        })
    }
    pub fn get_children(&self, parent: &u64) -> Vec<Member<T>> {
        let lock = self.data.read().unwrap();
        lock.dependents
            .get(parent)
            .unwrap_or(&HashSet::new())
            .iter()
            .filter_map(|x| {
                let node = lock.nodes.get(x)?;
                Some(Member {
                    id: x.clone(),
                    hierarchy: self.clone(),
                    value: node.clone(),
                })
            })
            .collect()
    }
    pub fn remove_parent(&self, child: &u64) -> Option<u64> {
        let mut lock = self.data.write().unwrap();
        shared_remove(child, &mut lock);
        lock.dependencies.remove(child)
    }
    pub fn remove_child(&self, parent: &u64, child: &u64) {
        let mut lock = self.data.write().unwrap();
        match lock.dependencies.get(&child) {
            Some(p) if p == parent => {
                lock.dependencies.remove(&child);
                if let Some(children_set) = lock.dependents.get_mut(&parent) {
                    children_set.remove(&child);
                    shared_remove(child, &mut lock);
                }
            }
            _ => {}
        }
    }
    pub fn remove_children(&self, parent: &u64) {
        let mut lock = self.data.write().unwrap();
        if let Some(children_set) = lock.dependents.remove(&parent) {
            children_set
                .iter()
                .for_each(|x| shared_remove(x, &mut lock));
        }
    }
    pub fn add_as_child(&self, parent: &u64, child: &u64) -> Result<(), &'static str> {
        let mut lock = self.data.write().unwrap();
        if !lock.nodes.contains_key(&child) || !lock.nodes.contains_key(&parent) {
            return Err("Both nodes must exist in the graph");
        }

        if let Some(old_parent) = lock.dependencies.insert(child.clone(), parent.clone()) {
            lock.dependents.get_mut(&old_parent).unwrap().remove(&child);
        }

        lock.dependents
            .entry(parent.clone())
            .or_default()
            .insert(child.clone());
        shared_add(child, parent, &mut lock);
        Ok(())
    }
    pub fn remove_entry(&self, entry: &u64) {
        remove(entry, &mut self.data.write().unwrap())
    }
    /// Will trigger the shared behavior of the hierarchy from a member.
    pub fn shared_behavior(&self, entry: &u64) {
        let mut lock = &mut self.data.write().unwrap();
        if let Some(dependents) = lock.dependents.get(entry) {
            let dependents = dependents.clone();
            dependents.iter().for_each(|x| shared_add(x, entry, lock));
        };
    }
    /// Returns all the members that have no parent.
    pub fn get_roots(&self) -> Vec<Member<T>> {
        let lock = self.data.read().unwrap();
        lock.nodes
            .iter()
            .filter(|x| lock.dependencies.contains_key(x.0))
            .map(|x| Member {
                id: x.0.clone(),
                hierarchy: unsafe { std::mem::transmute(&self) },
                value: x.1.clone(),
            })
            .collect()
    }
}
fn remove<T>(entry: &u64, guard: &mut RwLockWriteGuard<HierarchyData<T>>) {
    guard.discarded_id.push(entry.clone());
    guard.dependencies.remove(entry);
    if let Some(val) = guard.dependents.remove(entry) {
        val.iter().for_each(|x| remove(x, guard))
    };
    if let Some(t) = guard.nodes.remove(entry) {
        *t.write().unwrap() = Reference::Expired;
    };
}
fn shared_add<T: SharedBehavior>(
    child: &u64,
    parent: &u64,
    guard: &mut RwLockWriteGuard<HierarchyData<T>>,
) {
    if let Some(t) = guard.nodes.get(parent) {
        if let Reference::Some(parent) = &*t.read().unwrap() {
            if let Some(t) = guard.nodes.get(child) {
                if let Reference::Some(child) = &mut *t.write().unwrap() {
                    child.shaded_add_behavior(parent);
                }
            }
        }
    }

    if let Some(dependents) = guard.dependents.get(child) {
        let dependents = dependents.clone();
        dependents.iter().for_each(|x| shared_add(x, child, guard));
    };
}
fn shared_remove<T: SharedBehavior>(entry: &u64, guard: &mut RwLockWriteGuard<HierarchyData<T>>) {
    if let Some(t) = guard.nodes.get(entry) {
        if let Reference::Some(child) = &mut *t.write().unwrap() {
            child.shaded_remove_behavior();
        }
    }

    if let Some(dependents) = guard.dependents.get(entry) {
        let dependents = dependents.clone();
        dependents.iter().for_each(|x| shared_add(x, entry, guard));
    };
}
