use corrosive_ecs_core::build::general_helper::create_engine;
use std::env;
use std::process::Command;

pub fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    env::set_var("CORROSIVE_APP_ROOT", &current_dir);
    //create_engine();
    Command::new("cargo")
        .arg("fmt")
        .status()
        .expect("Failed to execute cargo fmt");
}
