pub mod comp;
mod math2d;
mod mesh2d;
pub mod task;

use corrosive_ecs_core_macro::corrosive_engine_builder;

fn main() {
    println!("Hello, world!");
}

corrosive_engine_builder!();
