/// Automatically implemented for archetype input wrappers.
pub trait EngineArch<T> {
    fn remove(&self, index: usize);
    fn len(&self) -> usize;
    fn get_item(&self, index: usize) -> Option<T>;
}

/// Used as input archetypes for tasks.
/// Generic type T is the item type of the archetype.
/// T must be a tuple.
/// Arch can be iterated over by `.iter()` function.
///
/// Example:
/// ```rust
/// #[task]
/// pub fn render_2d(meta: Arch<(&dyn Mesh2D, &RendererMeta2D)>, renderer2d_data: Res<Renderer2dData>) {
///     //code
///     }
/// }
/// ```
pub struct Arch<'a, T> {
    pub arch: &'a dyn EngineArch<T>,
    pub index: usize,
}
impl<'a, T> Arch<'a, T> {
    pub fn new(arch: &'a dyn EngineArch<T>) -> Self {
        Arch { arch, index: 0 }
    }
    /// Marks an entity for removal.
    pub fn remove(&self, index: usize) {
        self.arch.remove(index);
    }
    /// Returns the number of entities in the archetype.
    pub fn len(&self) -> usize {
        self.arch.len()
    }
    /// Returns an Iterator for the archetype.
    pub fn iter(&self) -> ArchIterator<'_, T> {
        ArchIterator {
            arch: self.arch,
            index: self.index,
        }
    }
}
/// Iterator wrapper for arch.
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
