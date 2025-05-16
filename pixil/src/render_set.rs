use std::sync::Mutex;

pub struct SparseSet<T> {
    sparse: Vec<usize>,   // Maps keys to indices in dense/data
    dense: Vec<usize>,    // Contains active keys
    data: Vec<T>,         // Contains values associated with keys
}

impl<T> SparseSet<T> {
    pub(crate) const  fn new() -> Self {
        Self {
            sparse: Vec::new(),
            dense: Vec::new(),
            data: Vec::new(),
        }
    }

    pub(crate) fn insert(&mut self, key: usize, value: T) {
        if key >= self.sparse.len() {
            self.sparse.resize(key + 1, usize::MAX);
        }

        if let Some(index) = self.get_index(key) {
            self.data[index] = value;
        } else {
            let index = self.dense.len();
            self.sparse[key] = index;
            self.dense.push(key);
            self.data.push(value);
        }
    }
    pub(crate) fn remove(&mut self, key: usize) -> Option<T> {
        let index = self.get_index(key)?;
        let last_key = *self.dense.last().unwrap();

        let len = self.dense.len() - 1;

        self.dense.swap(index, len);
        self.data.swap(index, len);

        self.sparse[last_key] = index;

        self.dense.pop();
        let value = self.data.pop().unwrap();

        self.sparse[key] = usize::MAX;

        Some(value)
    }

    pub(crate) fn contains(&self, key: usize) -> bool {
        self.get_index(key).is_some()
    }

    pub(crate) fn get(&self, key: usize) -> Option<&T> {
        self.get_index(key).map(|index| &self.data[index])
    }

    pub(crate) fn get_mut(&mut self, key: usize) -> Option<&mut T> {
        self.get_index(key).map(|index| &mut self.data[index])
    }

    pub(crate) fn len(&self) -> usize {
        self.dense.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.dense.is_empty()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (usize, &T)> {
        self.dense.iter().zip(self.data.iter()).map(|(&k, v)| (k, v))
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (usize, &mut T)> {
        self.dense.iter().zip(self.data.iter_mut()).map(|(&k, v)| (k, v))
    }

    pub(crate) fn get_index(&self, key: usize) -> Option<usize> {
        if key >= self.sparse.len() {
            return None;
        }
        let index = self.sparse[key];
        if index < self.dense.len() && self.dense[index] == key {
            Some(index)
        } else {
            None
        }
    }
}

pub(crate) struct RenderSet<T>{
    pub(crate) data : Mutex<RenderData<T>>
}
pub struct RenderData<T>{
    pub(crate) latest: usize,
    pub(crate) enabled: SparseSet<T>,
    pub(crate) disabled: SparseSet<T>,
}
impl<T> RenderSet<T>{
    pub(crate) const fn new() -> RenderSet<T>{
        RenderSet{
            data: Mutex::new(RenderData{
                latest: 0,
                enabled: SparseSet::new(),
                disabled: SparseSet::new(),
            })
        }
    }
    pub(crate) fn add_enabled(&self, id: T){
        let data = &mut *self.data.lock().unwrap();
        data.latest += 1;
        data.enabled.insert(data.latest, id);
    }
    pub(crate) fn add_disabled(&self, id: T){
        let data = &mut *self.data.lock().unwrap();
        data.latest += 1;
        data.disabled.insert(data.latest, id);
    }
    pub(crate) fn remove(&self, id: usize){
        let mut data = self.data.lock().unwrap();
        data.disabled.remove(id);
        data.enabled.remove(id);
    }
    pub(crate) fn enable(&self, id: usize){
        let mut data = self.data.lock().unwrap();
        match data.disabled.remove(id){
            None => {}
            Some(v) => {data.enabled.insert(id,v);}
        }
    }
    pub(crate) fn disable(&self, id: usize){
        let mut data = self.data.lock().unwrap();
        match data.enabled.remove(id){
            None => {}
            Some(v) => {data.disabled.insert(id,v);}
        }
    }
}