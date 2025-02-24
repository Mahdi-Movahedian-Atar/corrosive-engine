use corrosive_ecs_core_macro::corrosive_engine_builder;

pub mod comp;
mod render_graph;
pub mod task;

corrosive_engine_builder!(
    setup "run_renderer"
);
