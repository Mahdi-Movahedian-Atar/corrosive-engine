use corrosive_ecs_core_macro::corrosive_engine_builder;

pub mod comp;
pub mod task;

corrosive_engine_builder!(
    setup "start_egui" after "run_renderer",
);
