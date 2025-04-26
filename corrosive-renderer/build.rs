use anyhow::*;
use corrosive_ecs_core::build::general_helper::create_engine_package;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> Result<()> {
    let out_dir = env::var("OUT_DIR")?;
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let mut paths_to_copy = Vec::new();
    paths_to_copy.push("res/");
    copy_items(&paths_to_copy, out_dir, &copy_options)?;

    let crate_name = env!("CARGO_PKG_NAME");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    create_engine_package(crate_name, current_dir.to_str().unwrap());
    Ok(())
}
