#![allow(warnings)]

use corrosive_ecs_core_macro::corrosive_engine_builder;
use corrosive_engine::auto_prelude::*;

mod comp;
mod core_test;
#[path = ".corrosive_engine/mod.rs"]
mod corrosive_engine;

#[path = "corrosive-components/arch_types.rs"]
mod e;
mod task;

use crate::task::other_tasks::other_other_task::*;
use crate::task::other_tasks::*;
use crate::task::*;

fn main() {
    //corrosive_engine!(| update , sss|, | ss);
    corrosive_engine_builder!(
        path "./src",
        setup "setup",
        setup "setup1",
        setup "setup2",
        fixed_update "fixed_task" in_group "a",
        update "update_task_signal" in_group "a" if("Signal1"&&"signal2"||"signal3"&&StateExample::A),
        long_update "fixed_task" before_group "a",
        update "update_task",
        sync_update "sync_task"
    );
}
