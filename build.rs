use corrosive_ecs_core_macro::corrosive_engine_builder;
use std::process::Command;

pub fn main() {
    //corrosive_engine_builder!(p "./src" );
    let status = Command::new("cargo")
        .arg("fmt")
        .status()
        .expect("Failed to execute cargo fmt");
}
