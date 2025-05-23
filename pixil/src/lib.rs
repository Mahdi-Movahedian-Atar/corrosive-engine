use corrosive_ecs_core_macro::corrosive_engine_builder;

pub mod comp;
pub mod material;
pub mod mesh;
pub mod position_operations;
pub mod render_set;
pub mod task;
mod view_data;

corrosive_engine_builder! {
    setup "start_pixil_renderer" after "run_renderer",
}
