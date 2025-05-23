pub mod comp;
pub mod material2d;
mod mesh2d;
pub mod position2d_operations;
pub mod task;

use corrosive_ecs_core_macro::corrosive_engine_builder;

fn main() {
    println!("Hello, world!");
}

corrosive_engine_builder!(
    setup "start_2d_renderer" before "run_renderer",
    setup "init_camera" after "run_renderer",
    update "render_2d",
    update "update_position",
);
