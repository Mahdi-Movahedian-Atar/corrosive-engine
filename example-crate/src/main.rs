#![allow(warnings)]

use crate::corrosive_engine::engine::run_engine;
use corrosive_ecs_core_macro::corrosive_engine_builder;

mod comp;
#[path = ".corrosive_engine/mod.rs"]
mod corrosive_engine;
mod task;

corrosive_engine_builder!(
    path "./src",
    setup "setup",
    setup "setup1",
    setup "setup2",
    fixed_update "fixed_task" in_group "a",
    update "update_task_signal" in_group "a" if("Signal1"&&"signal2"||"signal3"&&StateExample::A),
    long_update "long_task" before_group "a",
    update "update_task",
    sync_update "sync_task"
);

/*corrosive_engine_builder!(
    update "macro_test",
);*/
/*
corrosive_engine_builder!(
    package "corrosive-ecs-renderer-backend",
    package "corrosive-ui"
);*/

/*corrosive_engine_builder!(
    package "corrosive-ecs-renderer-backend",
    package "corrosive-2d",
    package "corrosive-egui",
    package "corrosive-events",
    setup "test2_0" after "run_renderer",
    update "move_camera"
);*/

fn main() {
    //create_engine();
    run_engine()
    //corrosive_engine!(| update , sss|, | ss);
    /*corrosive_engine_builder!(
        path "./src",
        setup "setup",
        setup "setup1",
        setup "setup2",
        fixed_update "fixed_task" in_group "a",
        update "update_task_signal" in_group "a" if("Signal1"&&"signal2"||"signal3"&&StateExample::A),
        long_update "long_task" before_group "a",
        update "update_task",
        sync_update "sync_task"
    );*/
}
