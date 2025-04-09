use corrosive_ecs_core::build::general_helper::create_engine_package;
use std::env;

fn main() {
    let crate_name = env!("CARGO_PKG_NAME");
    let current_dir = env::current_dir().expect("Failed to get current directory");
    corrosive_asset_manager::asset_package::append_assets(
        current_dir.to_str().unwrap(),
        crate_name,
    )
    .expect("Could not append assets");
    create_engine_package(crate_name, current_dir.to_str().unwrap());
}
