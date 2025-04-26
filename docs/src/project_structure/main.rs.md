# main.rs

1. It needs to have `corrosive_engine_builder!` macro.
2. `corrosive_engine` needs to be imported from `".corrosive_engine/mod.rs"`.
3. `run_engine()` needs to be called.

## example
```
use crate::corrosive_engine::engine::run_engine;
use corrosive_ecs_core_macro::corrosive_engine_builder;

mod comp;
mod task;

#[path = ".corrosive_engine/mod.rs"]
mod corrosive_engine;

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

fn main() {
    run_engine()
}
```