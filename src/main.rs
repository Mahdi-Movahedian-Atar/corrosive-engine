#![allow(warnings)]

use crate::corrosive_engine::engine::run_engine;
use corrosive_ecs_core::build::general_helper::create_engine;
use corrosive_ecs_core_macro::corrosive_engine_builder;
use corrosive_engine::auto_prelude::*;
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::env;

mod comp;
mod core_test;
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
