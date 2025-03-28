#[cfg(feature = "build")]
pub mod build;

#[cfg(feature = "core")]
pub mod ecs_core {
    use bus::{Bus, BusReader};
    use std::collections::{HashMap, HashSet};
    use std::sync::{Arc, LockResult, PoisonError, RwLock, RwLockReadGuard, RwLockWriteGuard};

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
    pub struct Member<'a, T: SharedBehavior> {
        pub id: u64,
        pub hierarchy: &'a Hierarchy<T>,
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
            *self.value.write().unwrap() = Reference::Expired;
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
    impl<'a, T: SharedBehavior> Member<'a, T> {
        pub fn new(value: T, hierarchy: &'a Hierarchy<T>) -> Member<'a, T> {
            hierarchy.new_entry(value)
        }

        pub fn clone(&self) -> Member<'a, T> {
            Member {
                id: self.id.clone(),
                hierarchy: self.hierarchy,
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

        pub fn dry_write(&self) -> LockResult<RwLockWriteGuard<'_, Reference<T>>> {
            self.value.write()
        }
        pub fn dry_e_write(&self, massage: &str) -> RwLockWriteGuard<'_, Reference<T>> {
            self.value.write().expect(massage)
        }
        pub fn dry_f_write(&self) -> RwLockWriteGuard<'_, Reference<T>> {
            self.value.write().expect("Failed to force write a lock")
        }

        pub fn write(&self, mut func: impl FnMut(LockResult<RwLockWriteGuard<'_, Reference<T>>>)) {
            func(self.value.write());
            self.hierarchy.shared_add(&self.id);
        }
        pub fn e_write(
            &self,
            mut func: impl FnMut(RwLockWriteGuard<'_, Reference<T>>),
            massage: &str,
        ) {
            func(self.value.write().expect(massage));
            self.hierarchy.shared_add(&self.id);
        }
        pub fn f_write(&self, mut func: impl FnMut(RwLockWriteGuard<'_, Reference<T>>)) {
            func(self.value.write().expect("Failed to force write a lock"));
            self.hierarchy.shared_add(&self.id);
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

        pub fn expire(&mut self) {
            self.hierarchy.remove_entry(&self.id)
        }
    }
    pub trait SharedBehavior {
        fn shaded_add_behavior(&mut self, parent: &Self);
        fn shaded_remove_behavior(&mut self);
    }

    pub trait EngineArch<T> {
        fn remove(&self, index: usize);
        fn len(&self) -> usize;
        fn get_item(&self, index: usize) -> Option<T>;
    }
    pub trait ArchBuilder<'a, T> {
        fn build(&self) -> Arch<'a, T>;
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

    pub struct Res<T> {
        pub value: Arc<RwLock<T>>,
    }
    impl<T> Res<T> {
        pub fn new(value: T) -> Res<T> {
            Res {
                value: Arc::new(RwLock::new(value)),
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

        pub fn clone(&self) -> Res<T> {
            Res {
                value: self.value.clone(),
            }
        }
    }
    pub struct State<T> {
        pub value: Arc<RwLock<T>>,
    }
    impl<T> State<T> {
        pub fn new(value: T) -> State<T> {
            State {
                value: Arc::new(RwLock::new(value)),
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

        pub fn clone(&self) -> State<T> {
            State {
                value: self.value.clone(),
            }
        }
    }
    #[derive(Default)]
    pub struct HierarchyData<T> {
        latest_id: u64,
        discarded_id: Vec<u64>,
        nodes: HashMap<u64, Arc<RwLock<Reference<T>>>>,
        dependencies: HashMap<u64, u64>,
        dependents: HashMap<u64, HashSet<u64>>,
    }
    #[derive(Default)]
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
                hierarchy: &self,
                value: Arc::new(RwLock::new(Reference::Some(value))),
            }
        }
        pub fn get_parent(&self, child: &u64) -> Option<Member<T>> {
            let lock = self.data.read().unwrap();
            let i = lock.dependencies.get(child)?;
            let data = lock.nodes.get(i).clone()?;
            Some(Member {
                id: i.clone(),
                hierarchy: &self,
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
                        hierarchy: &self,
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
        pub fn shared_add(&self, entry: &u64) {
            let mut lock = &mut self.data.write().unwrap();
            if let Some(dependents) = lock.dependents.get(entry) {
                let dependents = dependents.clone();
                dependents.iter().for_each(|x| shared_add(x, entry, lock));
            };
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
    fn shared_remove<T: SharedBehavior>(
        entry: &u64,
        guard: &mut RwLockWriteGuard<HierarchyData<T>>,
    ) {
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

    pub struct RArch<T> {
        pub vec: Vec<T>,
    }
    impl<T> Default for RArch<T> {
        fn default() -> Self {
            RArch { vec: Vec::new() }
        }
    }

    impl<T> RArch<T> {
        pub fn add(&mut self, t: T) {
            self.vec.push(t);
        }

        pub fn add_multiple<I>(&mut self, items: I)
        where
            I: IntoIterator<Item = T>,
        {
            self.vec.extend(items)
        }

        pub fn get(&self) -> &Vec<T> {
            &self.vec
        }
    }
    #[derive(Default)]
    pub struct Signal {
        pub vec: HashSet<String>,
    }
    impl Signal {
        pub fn trigger(&mut self, signal: &str) {
            self.vec.insert(signal.to_string());
        }
    }
    #[derive(Default)]
    pub struct Reset(bool);
    impl Reset {
        pub fn trigger(&mut self) {
            self.0 = true;
        }
        pub fn get(&self) -> bool {
            self.0
        }
    }
    #[macro_export]
    macro_rules! trait_for {
        (trait $e:ty => $($z:ty),+ ) => {};
    }
}
