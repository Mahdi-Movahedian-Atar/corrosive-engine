pub(crate) mod other_other_task;

use crate::corrosive_engine::arch_types::arch_types::TestUtArch2;
use corrosive_ecs_core_macro::task;
#[task]
pub fn sync_task(inp: TestUtArch2) {
    println!("sync")
}
