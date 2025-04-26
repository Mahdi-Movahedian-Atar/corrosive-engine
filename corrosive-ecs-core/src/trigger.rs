use bus::{Bus, BusReader};

/// Used by engine to manage task schedules.
pub struct Trigger(Bus<()>);
/// Used by engine to manage task schedules.
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
