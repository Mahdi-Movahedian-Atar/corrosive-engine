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
