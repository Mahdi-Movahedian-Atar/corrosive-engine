use crate::comp::State;
use corrosive_ecs_core_macro::corrosive_engine_builder;

pub mod assets;
pub mod color;
pub mod comp;
pub mod helper;
pub mod material;
pub mod render_graph;
pub mod task;

pub(crate) static mut STATE: Option<State> = None;

corrosive_engine_builder!(
    setup "run_renderer"
);
