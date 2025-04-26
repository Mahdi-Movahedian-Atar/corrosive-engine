pub mod comp;
mod style;
pub mod task;

use corrosive_ecs_core_macro::corrosive_engine_builder;

pub const Z_INDEX_STEPS: f32 = 0.00001;

fn main() {}

corrosive_engine_builder!(setup "setup_ui_pass" after "run_renderer", update "rerender_ui");
