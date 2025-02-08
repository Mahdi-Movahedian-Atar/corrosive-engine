use corrosive_ecs_core::build::app_scan::AppPackage;
use corrosive_ecs_core::build::codegen::{
    create_app, generate_arch_types, generate_prelude, get_all_archetypes, write_rust_file,
};
use corrosive_ecs_core::build::components_scan::{
    get_component_map, scan_components, write_component_map,
};
use corrosive_ecs_core::build::general_scan::{get_path_map, scan_directory, write_path_map};
use corrosive_ecs_core::build::tasks_scan::{get_task_map, scan_tasks, write_task_map};
use proc_macro::{TokenStream, TokenTree};
use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use std::ptr::write;
use std::{env, fs, path};
use syn::parse::{Parse, ParseStream};
use syn::token::Token;
use syn::{parse_macro_input, token, Error, Ident, Lit, Result, Token};

pub fn corrosive_engine_builder(item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(item as AppPackage);

    let mut app_path = env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set");
    app_path.push_str("/src");

    //component scan

    let mut components_path_map = get_path_map(
        format!("{}/corrosive-components/components_path_map.json", app_path).as_str(),
        format!("{}/comp", args.path).as_str(),
    );
    if !components_path_map.path.ends_with("comp") {
        components_path_map.path =
            path::Path::new(format!("{}/comp", args.path).as_str()).to_path_buf();
    }

    scan_directory(
        &mut components_path_map,
        format!("{}/comp", args.path).as_str(),
    )
    .expect("Failed to scan comp directory");

    let mut component_map = get_component_map(
        format!("{}/corrosive-components/components.json", app_path).as_str(),
        format!("{}/comp", args.path).as_str(),
    );
    if !component_map.path.ends_with("comp") {
        component_map.path = path::Path::new(format!("{}/comp", args.path).as_str()).to_path_buf();
    }

    scan_components(&components_path_map, &mut component_map).expect("Failed to scan components");

    write_component_map(
        &component_map,
        format!("{}/corrosive-components/components.json", app_path).as_str(),
    )
    .expect("Filed to write component map file");

    write_path_map(
        &components_path_map,
        format!("{}/corrosive-components/components_path_map.json", app_path).as_str(),
    )
    .expect("Filed to write path map file");

    //task scan

    let mut tasks_path_map = get_path_map(
        format!("{}/corrosive-components/tasks_path_map.json", app_path).as_str(),
        format!("{}/task", args.path).as_str(),
    );
    if !tasks_path_map.path.ends_with("task") {
        tasks_path_map.path = path::Path::new(format!("{}/task", args.path).as_str()).to_path_buf();
    }

    scan_directory(&mut tasks_path_map, format!("{}/task", args.path).as_str())
        .expect("Failed to scan task directory");

    let mut task_map = get_task_map(
        format!("{}/corrosive-components/tasks.json", app_path).as_str(),
        format!("{}/task", args.path).as_str(),
    );
    if !task_map.path.ends_with("task") {
        task_map.path = path::Path::new(format!("{}/task", args.path).as_str()).to_path_buf();
    }

    scan_tasks(&tasks_path_map, &mut task_map).expect("Failed to scan tasks");

    write_task_map(
        &task_map,
        format!("{}/corrosive-components/tasks.json", app_path).as_str(),
    )
    .expect("Filed to write component map file");

    write_path_map(
        &tasks_path_map,
        format!("{}/corrosive-components/tasks_path_map.json", app_path).as_str(),
    )
    .expect("Filed to write path map file");

    let all_components = component_map.get_all();
    let all_tasks = task_map.get_all_with_path();

    let auto_prelude_code = generate_prelude(&component_map, &task_map);

    write_rust_file(
        auto_prelude_code,
        format!("{}/.corrosive_engine/auto_prelude.rs", app_path).as_str(),
    )
    .expect("failed to create auto_prelude.ts");

    /*let arch_types = get_all_archetypes(&task_map);
    write_rust_file(
        generate_arch_types(&arch_types),
        format!("{}/corrosive-components/arch_types.rs", app_path).as_str(),
    )
    .expect("failed to create arch_types.ts");*/

    create_app(vec![args], vec![task_map]);

    TokenStream::new()
}
