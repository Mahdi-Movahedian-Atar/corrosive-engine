use crate::corrosive_engine::arch_types;
use corrosive_ecs_core::reset;
use corrosive_ecs_core_macro::task;
use std::thread::sleep;
use std::time::Duration;

//#[task]
pub fn long_task() {
    reset!();
    sleep(Duration::from_secs(2));
    println!("Long");
}
