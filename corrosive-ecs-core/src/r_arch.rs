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
