pub(crate) mod other_other_task;

use corrosive_ecs_core_macro::task;
#[task]
pub fn sync_task() {
    println!("sync")
}
