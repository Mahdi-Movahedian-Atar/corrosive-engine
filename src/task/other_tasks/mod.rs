pub(crate) mod other_other_task;

use corrosive_ecs_core::ecs_core::DeltaTime;
use corrosive_ecs_core_macro::task;
#[task]
pub fn sync_task(d: DeltaTime) {
    println!("sync");
    println!("{}", d)
}
