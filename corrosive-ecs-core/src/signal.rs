use std::collections::HashSet;

/// Used to trigger signals.
/// Can be used as task outputs.
/// Signals can be used as conditions in `corrosive_app_builder!`
#[derive(Default)]
pub struct Signal {
    pub vec: HashSet<String>,
}

impl Signal {
    /// Will trigger a certain signal.
    pub fn trigger(&mut self, signal: &str) {
        self.vec.insert(signal.to_string());
    }
}
