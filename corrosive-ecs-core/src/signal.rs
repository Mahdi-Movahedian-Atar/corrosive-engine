use std::collections::HashSet;

#[derive(Default)]
pub struct Signal {
    pub vec: HashSet<String>,
}
impl Signal {
    pub fn trigger(&mut self, signal: &str) {
        self.vec.insert(signal.to_string());
    }
}
