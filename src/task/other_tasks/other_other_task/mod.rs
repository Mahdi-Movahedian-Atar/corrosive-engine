use crate::corrosive_engine::arch_types::arch_types::TestUtArch2;
use corrosive_ecs_core_macro::task;
use std::thread::sleep;
use std::time::Duration;

//#[task]
pub fn long_task(inp: TestUtArch2) -> bool {
    let mut reset = false;
    sleep(Duration::from_secs(2));
    println!("Long");
    //reset = true;
    (reset)
}
