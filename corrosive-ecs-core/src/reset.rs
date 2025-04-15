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
