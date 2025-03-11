pub mod comp;
mod style;
pub mod task;

use corrosive_ecs_core_macro::corrosive_engine_builder;

fn main() {}

corrosive_engine_builder!(setup "setup_ui_pass" after "run_renderer");
