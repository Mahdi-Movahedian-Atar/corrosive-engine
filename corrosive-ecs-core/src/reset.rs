/// Used as an output for tasks.
/// Should it be triggered, thr setup phase will be triggered as well.
#[derive(Default)]
pub struct Reset(bool);
impl Reset {
    /// Will trigger the reset.
    pub fn trigger(&mut self) {
        self.0 = true;
    }
    /// Gets the value of the reset type.
    pub fn get(&self) -> bool {
        self.0
    }
}
