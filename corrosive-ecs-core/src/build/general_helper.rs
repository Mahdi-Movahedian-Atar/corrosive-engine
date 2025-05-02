use crate::build::app_scan::{get_app_package, write_app_package, AppPackage};
use crate::build::codegen::{create_app, generate_arch_types, generate_prelude, write_rust_file};
use crate::build::components_scan::{get_component_map, scan_components, write_component_map};
use crate::build::general_scan::{get_path_map, scan_directory, write_path_map};
use crate::build::tasks_scan::{get_task_map, scan_tasks, write_task_map};
use std::path::{Path, PathBuf};
use std::{env, fs};
use syn::{parse2, parse_file, Item};

pub fn create_engine() {
    let mut app_path = env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set");
    app_path.push_str("/src/main.rs");
    let main_rs = PathBuf::from(app_path);
    let content = fs::read_to_string(&main_rs).expect("Failed to read lib");

    let ast = parse_file(&content).expect("Failed to parse main");

    let mut args: Option<AppPackage> = None;
    for item in ast.items {
        if let Item::Macro(ref macro_item) = item {
            if macro_item.mac.path.segments.last().unwrap().ident == "corrosive_engine_builder" {
                let tokens = macro_item.mac.tokens.clone();
                args = Some(parse2::<AppPackage>(tokens).expect("Failed to parse macro input"))
            }
        }
    }

    if args.is_none() {
        panic!("Failed to find corrosive_engine_builder macro in main.rs");
    }
    let args = args.unwrap();

    let mut app_path = env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set");
    app_path.push_str("/src");

    let path = args.path.clone();

    //component scan

    let mut components_path_map = get_path_map(
        format!("{}/.corrosive_engine/components_path_map.json", app_path).as_str(),
        format!("{}/comp", path).as_str(),
    );
    if !components_path_map.path.ends_with("comp") {
        components_path_map.path = Path::new(format!("{}/comp", path).as_str()).to_path_buf();
    }

    scan_directory(&mut components_path_map, format!("{}/comp", path).as_str())
        .expect("Failed to scan comp directory");

    let mut component_map = get_component_map(
        format!("{}/.corrosive_engine/components.json", app_path).as_str(),
        format!("{}/comp", path).as_str(),
    );
    if !component_map.path.ends_with("comp") {
        component_map.path = Path::new(format!("{}/comp", path).as_str()).to_path_buf();
    }

    scan_components(&components_path_map, &mut component_map).expect("Failed to scan components");

    write_component_map(
        &component_map,
        format!("{}/.corrosive_engine/components.json", app_path).as_str(),
    )
    .expect("Filed to write component map file");

    write_path_map(
        &components_path_map,
        format!("{}/.corrosive_engine/components_path_map.json", app_path).as_str(),
    )
    .expect("Filed to write path map file");

    let mut tasks_path_map = get_path_map(
        format!("{}/.corrosive_engine/tasks_path_map.json", app_path).as_str(),
        format!("{}/task", path).as_str(),
    );
    if !tasks_path_map.path.ends_with("task") {
        tasks_path_map.path = Path::new(format!("{}/task", path).as_str()).to_path_buf();
    }

    scan_directory(&mut tasks_path_map, format!("{}/task", path).as_str())
        .expect("Failed to scan task directory");

    let mut task_map = get_task_map(
        format!("{}/.corrosive_engine/tasks.json", app_path).as_str(),
        format!("{}/task", path).as_str(),
    );
    if !task_map.path.ends_with("task") {
        task_map.path = Path::new(format!("{}/task", path).as_str()).to_path_buf();
    }

    scan_tasks(&tasks_path_map, &mut task_map).expect("Failed to scan tasks");

    write_task_map(
        &task_map,
        format!("{}/.corrosive_engine/tasks.json", app_path).as_str(),
    )
    .expect("Filed to write component map file");

    write_path_map(
        &tasks_path_map,
        format!("{}/.corrosive_engine/tasks_path_map.json", app_path).as_str(),
    )
    .expect("Filed to write path map file");

    let mut trait_to_components = component_map.get_trait_to_components();
    let mut tasks = vec![task_map.clone()];
    let mut component_map = vec![(component_map, "crate".to_string())];
    let mut task_map = vec![(task_map, "crate".to_string())];
    let mut app_packages = vec![args];

    match fs::read_dir(Path::new(
        format!("{}/.corrosive_engine/packages/", app_path).as_str(),
    )) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        if let Some(folder_name) = entry_path.file_name().and_then(|n| n.to_str()) {
                            let component = get_component_map(
                                format!(
                                    "{}/.corrosive_engine/packages/{}/components.json",
                                    app_path, folder_name
                                )
                                .as_str(),
                                format!("{}/comp", path).as_str(),
                            );
                            let task = get_task_map(
                                format!(
                                    "{}/.corrosive_engine/packages/{}/tasks.json",
                                    app_path, folder_name
                                )
                                .as_str(),
                                format!("{}/task", path).as_str(),
                            );
                            let app_package = get_app_package(
                                format!(
                                    "{}/.corrosive_engine/packages/{}/app_package.json",
                                    app_path, folder_name
                                )
                                .as_str(),
                            );
                            for i in &component.get_trait_to_components() {
                                if let Some(t) = trait_to_components.get_mut(i.0) {
                                    t.extend(i.1.clone());
                                } else {
                                    trait_to_components.insert(i.0.clone(), i.1.clone());
                                }
                            }
                            tasks.push(task.clone());
                            task_map.push((task, folder_name.to_string()));
                            component_map.push((component, folder_name.to_string()));
                            app_packages.push(app_package);
                        }
                    }
                } else {
                    panic!("Failed to read an entry.");
                }
            }
        }
        Err(_) => {
            panic!(
                "{}",
                format!("{}/.corrosive_engine/packages/", app_path).as_str()
            );
        }
    }

    let auto_prelude_code = generate_prelude(component_map, task_map);

    write_rust_file(
        auto_prelude_code,
        format!("{}/.corrosive_engine/auto_prelude.rs", app_path).as_str(),
    )
    .expect("failed to create auto_prelude.ts");

    let app = create_app(app_packages, tasks, trait_to_components);

    write_rust_file(
        generate_arch_types(&app.1),
        format!("{}/.corrosive_engine/arch_types.rs", app_path).as_str(),
    )
    .expect("failed to create arch_types.ts");

    write_rust_file(
        app.0,
        format!("{}/.corrosive_engine/engine.rs", app_path).as_str(),
    )
    .expect("failed to create arch_types.ts");
}
pub fn create_engine_package(package_name: &str, crate_root: &str) {
    let mut app_path = env::var("CORROSIVE_APP_ROOT").expect("CORROSIVE_APP_ROOT is not set");
    app_path.push_str(format!("/src/.corrosive_engine/packages/{}", package_name).as_str());
    fs::create_dir_all(&app_path).unwrap();

    //app package

    let main_rs = PathBuf::from(format!("{}/src/lib.rs", crate_root).as_str());
    let content = fs::read_to_string(&main_rs).expect("Failed to read lib");

    let ast = parse_file(&content).expect("Failed to parse lib");

    let mut app_package: Option<AppPackage> = None;
    for item in ast.items {
        if let Item::Macro(ref macro_item) = item {
            if macro_item.mac.path.segments.last().unwrap().ident == "corrosive_engine_builder" {
                let tokens = macro_item.mac.tokens.clone();
                app_package =
                    Some(parse2::<AppPackage>(tokens).expect("Failed to parse macro input"))
            }
        }
    }

    if app_package.is_none() {
        panic!("Failed to find corrosive_engine_builder macro in lib.rs");
    }

    let mut app = app_package.unwrap();
    app.name = package_name.to_string();

    write_app_package(&app, format!("{}/app_package.json", app_path).as_str())
        .expect("Failed to write app package");

    //component scan

    let mut components_path_map = get_path_map("", format!("{}/src/comp", crate_root).as_str());
    scan_directory(
        &mut components_path_map,
        format!("{}/src/comp", crate_root).as_str(),
    )
    .expect("Failed to scan comp directory");
    let mut component_map = get_component_map("", format!("{}/src/comp", crate_root).as_str());
    scan_components(&components_path_map, &mut component_map).expect("Failed to scan components");
    write_component_map(
        &component_map,
        format!("{}/components.json", app_path).as_str(),
    )
    .expect("Filed to write component map file");

    //task_scan

    let mut tasks_path_map = get_path_map("", format!("{}/src/task", crate_root).as_str());

    scan_directory(
        &mut tasks_path_map,
        format!("{}/src/task", crate_root).as_str(),
    )
    .expect("Failed to scan task directory");

    let mut task_map = get_task_map("", format!("{}/src/task", crate_root).as_str());
    scan_tasks(&tasks_path_map, &mut task_map).expect("Failed to scan tasks");

    write_task_map(&task_map, format!("{}/tasks.json", app_path).as_str())
        .expect("Filed to write component map file");
}
