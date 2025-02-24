use corrosive_ecs_core_macro::corrosive_engine_builder;

mod camera;
pub mod comp;
mod lights;
mod model;
pub mod task;
mod texture;

corrosive_engine_builder!(
    setup "run_renderer"
);
