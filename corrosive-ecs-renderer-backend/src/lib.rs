use crate::comp::State;
use corrosive_ecs_core_macro::corrosive_engine_builder;

pub mod assets;
pub mod color;
pub mod comp;
pub mod material;
pub mod public_functions;
pub mod render_graph;
mod slang;
pub mod task;

pub use image;
pub use wgpu;
pub use winit;

pub(crate) static mut STATE: Option<State> = None;

corrosive_engine_builder!(
    setup "run_renderer"
);
