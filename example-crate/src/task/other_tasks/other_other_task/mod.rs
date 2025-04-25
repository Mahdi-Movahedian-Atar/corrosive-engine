use crate::comp::Position1;
use crate::corrosive_engine::arch_types;
use corrosive_ecs_core::ecs_core::{Arch, Locked};
use corrosive_ecs_core_macro::task;
use std::thread::sleep;
use std::time::Duration;

#[task]
pub fn long_task(inp: Arch<(&Locked<Position1>,)>) {
    sleep(Duration::from_secs(2));
    println!("Long");
}
