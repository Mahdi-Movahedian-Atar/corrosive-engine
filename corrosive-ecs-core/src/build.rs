pub const ENGINE_DIR: &str = ".corrosive-components";
#[allow(non_snake_case)]
pub mod general_helper {
    use crate::build::app_scan::{get_app_package, write_app_package, AppPackage};
    use crate::build::codegen::{
        create_app, generate_arch_types, generate_prelude, write_rust_file,
    };
    use crate::build::components_scan::{get_component_map, scan_components, write_component_map};
    use crate::build::general_scan::{get_path_map, scan_directory, write_path_map};
    use crate::build::tasks_scan::{get_task_map, scan_tasks, write_task_map};
    use std::path::{Path, PathBuf};
    use std::{env, fs};
    use syn::{parse2, parse_file, Item};

    pub fn create_engine() {
        let main_rs = PathBuf::from("src/main.rs");
        let content = fs::read_to_string(&main_rs).expect("Failed to read lib");

        let ast = parse_file(&content).expect("Failed to parse lib");

        let mut args: Option<AppPackage> = None;
        for item in ast.items {
            if let Item::Macro(ref macro_item) = item {
                if macro_item.mac.path.segments.last().unwrap().ident == "corrosive_engine_builder"
                {
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

        scan_components(&components_path_map, &mut component_map)
            .expect("Failed to scan components");

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
                            if let Some(folder_name) =
                                entry_path.file_name().and_then(|n| n.to_str())
                            {
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
                                tasks.push(task.clone());
                                task_map.push((task, folder_name.to_string()));
                                component_map.push((component, folder_name.to_string()));
                                app_packages.push(app_package)
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

        let app = create_app(app_packages, tasks);

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
                if macro_item.mac.path.segments.last().unwrap().ident == "corrosive_engine_builder"
                {
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
        scan_components(&components_path_map, &mut component_map)
            .expect("Failed to scan components");
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
        /*if !component_map.path.ends_with("comp") {
                component_map.path = Path::new(format!("{}/comp", args.path).as_str()).to_path_buf();
            }

            scan_components(&components_path_map, &mut component_map)
                .expect("Failed to scan components");

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
                format!("{}/task", args.path).as_str(),
            );
            if !tasks_path_map.path.ends_with("task") {
                tasks_path_map.path = Path::new(format!("{}/task", args.path).as_str()).to_path_buf();
            }

            scan_directory(&mut tasks_path_map, format!("{}/task", args.path).as_str())
                .expect("Failed to scan task directory");

            let mut task_map = get_task_map(
                format!("{}/.corrosive_engine/tasks.json", app_path).as_str(),
                format!("{}/task", args.path).as_str(),
            );
            if !task_map.path.ends_with("task") {
                task_map.path = Path::new(format!("{}/task", args.path).as_str()).to_path_buf();
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

            let auto_prelude_code = generate_prelude(&component_map, &task_map);

            write_rust_file(
                auto_prelude_code,
                format!("{}/.corrosive_engine/auto_prelude.rs", app_path).as_str(),
            )
                .expect("failed to create auto_prelude.ts");

            let app = create_app(vec![args], vec![task_map]);

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
        }*/
    }
}
#[allow(non_snake_case)]
pub mod general_scan {
    use crate::build::ENGINE_DIR;
    use std::path::{Path, PathBuf};
    use std::time::SystemTime;
    use std::{fs, io};

    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Default)]
    pub enum ModifiedState {
        #[default]
        Changed,
        Removed,
        None,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct PathMap {
        pub path: PathBuf,
        pub modified_time: SystemTime,
        pub modified_state: ModifiedState,
        pub sub_maps: Vec<PathMap>,
    }

    impl Default for PathMap {
        fn default() -> Self {
            PathMap {
                path: Path::new("./").to_path_buf(),
                modified_time: SystemTime::now(),
                sub_maps: Vec::new(),
                modified_state: ModifiedState::Changed,
            }
        }
    }

    impl PathMap {
        fn remove(&mut self) {
            self.modified_state = ModifiedState::Removed;
            for m in &mut self.sub_maps {
                m.modified_state = ModifiedState::Removed;
                m.remove()
            }
        }

        fn none(&mut self) {
            self.modified_state = ModifiedState::None;
            for m in &mut self.sub_maps {
                m.modified_state = ModifiedState::None;
                m.none()
            }
        }
    }

    pub fn get_path_map(file_path: &str, default_path: &str) -> PathMap {
        match fs::read_to_string(file_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => PathMap {
                path: Path::new(default_path).to_path_buf(),
                modified_time: SystemTime::now(),
                sub_maps: Vec::new(),
                modified_state: ModifiedState::Changed,
            },
        }
    }

    pub fn write_path_map(path_map: &PathMap, path: &str) -> io::Result<()> {
        let serialized = serde_json::to_string_pretty(path_map)?;
        fs::write(path, serialized)?;
        Ok(())
    }

    pub fn scan_directory(path_map: &mut PathMap, start_path: &str) -> io::Result<()> {
        if path_map.path.as_path().is_dir() && !path_map.path.as_path().ends_with(ENGINE_DIR) {
            let mut files = Vec::new();
            for entry in fs::read_dir(path_map.path.as_path())? {
                let entry = entry?;
                let path = entry.path();
                let meta_data = fs::metadata(&path)?;
                files.push(path.clone());

                match path_map.sub_maps.iter_mut().find(|item| item.path == path) {
                    Some(T) => {
                        if T.modified_time != meta_data.modified()? {
                            T.modified_time = meta_data.modified()?;
                            T.modified_state = ModifiedState::Changed;
                        } else {
                            T.none()
                        }
                        scan_directory(T, start_path)?
                    }
                    None => {
                        path_map.sub_maps.push(PathMap {
                            path: path.clone(),
                            modified_time: meta_data.modified()?,
                            modified_state: ModifiedState::Changed,
                            sub_maps: vec![],
                        });
                        scan_directory(path_map.sub_maps.last_mut().unwrap(), start_path)
                    }?,
                }
            }
            let _ = path_map
                .sub_maps
                .iter_mut()
                .filter(|item| !files.contains(&item.path))
                .for_each(|item| item.remove());
            path_map
                .sub_maps
                .retain(|item| item.modified_state != ModifiedState::Removed);
        }
        if path_map.path.as_path() == Path::new(start_path) {
            path_map.modified_state = ModifiedState::None;
            path_map
                .sub_maps
                .iter()
                .find(|item| item.modified_state == ModifiedState::Changed)
                .into_iter()
                .for_each(|_| path_map.modified_state = ModifiedState::Changed);
        }
        Ok(())
    }
}
#[allow(non_snake_case)]
pub mod components_scan {
    use crate::build::general_scan::{ModifiedState, PathMap};
    use quote::ToTokens;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::{fs, io};
    use syn::{Attribute, File, Item, ItemEnum, ItemStruct, ItemType};

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct ComponentMap {
        pub path: PathBuf,
        pub sub_maps: Vec<ComponentMap>,
        pub components: Vec<String>,
    }

    impl ComponentMap {
        pub fn get_all(&self) -> HashMap<String, String> {
            let mut data: HashMap<String, String> = HashMap::new();
            let path = self.path.as_path().iter().last().unwrap().to_str().unwrap();
            for i in &self.components {
                data.insert(i.clone(), format!("{}::{}", path, i).to_string());
            }
            for i in &self.sub_maps {
                for i in i.get_all() {
                    data.insert(i.0, format!("{}::{}", path, i.1).to_string());
                }
            }
            data
        }
    }

    impl Default for ComponentMap {
        fn default() -> Self {
            ComponentMap {
                path: Path::new("./").to_path_buf(),
                sub_maps: Vec::new(),
                components: Vec::new(),
            }
        }
    }

    pub fn get_component_map(file_path: &str, default_path: &str) -> ComponentMap {
        match fs::read_to_string(file_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => ComponentMap {
                path: Path::new(default_path).to_path_buf(),
                sub_maps: Vec::new(),
                components: Vec::new(),
            },
        }
    }

    pub fn write_component_map(component_map: &ComponentMap, file_path: &str) -> io::Result<()> {
        let serialized = serde_json::to_string_pretty(component_map)?;
        fs::write(file_path, serialized)?;
        Ok(())
    }

    pub fn scan_components(path_map: &PathMap, component_map: &mut ComponentMap) -> io::Result<()> {
        let mut file_to_scan: Option<&PathMap> = None;
        let mut directories_to_scan = Vec::new();

        for sub_map in &path_map.sub_maps {
            if sub_map.path.is_dir() {
                directories_to_scan.push(sub_map)
            } else if sub_map.path.ends_with("mod.rs") {
                file_to_scan = Some(sub_map)
            }
        }

        let mut visited_paths = Vec::new();

        'outer: for directory in directories_to_scan {
            visited_paths.push(directory.path.clone());
            for sub_map in &mut component_map.sub_maps {
                if sub_map.path == directory.path {
                    if directory.modified_state == ModifiedState::Changed {
                        scan_components(directory, sub_map)?;
                    }
                    continue 'outer;
                }
            }
            let mut new_component_map = ComponentMap {
                path: directory.path.clone(),
                sub_maps: vec![],
                components: vec![],
            };
            scan_components(directory, &mut new_component_map)?;
            component_map.sub_maps.push(new_component_map);
        }
        component_map
            .sub_maps
            .retain(|item| visited_paths.contains(&item.path));

        if let Some(T) = file_to_scan {
            component_map.components = find_structs_with_component(T.path.as_path())
        } else {
            component_map.components = Vec::new()
        }
        Ok(())
    }

    fn find_structs_with_component(file_path: &Path) -> Vec<String> {
        let content = match fs::read_to_string(file_path) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("Failed to read file {}: {}", file_path.display(), err);
                return vec![];
            }
        };

        let syntax: File = match syn::parse_file(&content) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to parse file {}: {}", file_path.display(), err);
                return vec![];
            }
        };

        let mut struct_names = Vec::new();

        for item in syntax.items {
            match item {
                Item::Struct(ItemStruct { attrs, ident, .. }) => {
                    if has_component_attr(&attrs) {
                        struct_names.push(ident.to_string());
                    }
                }
                Item::Enum(ItemEnum { attrs, ident, .. }) => {
                    if has_component_attr(&attrs) {
                        struct_names.push(ident.to_string());
                    }
                }
                Item::Type(ItemType { attrs, ident, .. }) => {
                    if has_component_attr(&attrs) {
                        struct_names.push(ident.to_string());
                    }
                }
                _ => {}
            }
        }

        struct_names
    }

    fn has_component_attr(attrs: &[Attribute]) -> bool {
        for attr in attrs {
            if attr.path().is_ident("derive") {
                let tokens = attr.to_token_stream().to_string();
                if tokens.contains("Component")
                    || tokens.contains("State")
                    || tokens.contains("Resource")
                {
                    return true;
                }
            }
        }

        false
    }
}
#[allow(non_snake_case)]
pub mod tasks_scan {
    use crate::build::general_scan::{ModifiedState, PathMap};
    use quote::ToTokens;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::{fs, io};
    use syn::punctuated::Punctuated;
    use syn::token::Comma;
    use syn::visit::Visit;
    use syn::{
        Attribute, File, FnArg, GenericArgument, Item, ItemFn, LitStr, Pat, PathArguments, Stmt,
        Type, TypeTuple,
    };

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
    pub struct Task {
        pub name: String,
        pub input_archs: Vec<(String, Vec<String>)>,
        pub input_resources: Vec<(String, String)>,
        pub input_states: Vec<(String, String)>,
        pub input_delta_time: Option<String>,
        pub output_archs: Vec<Vec<(String, String)>>,
        pub output_signals: Vec<String>,
        pub output_reset: bool,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
    pub struct TaskMap {
        pub path: PathBuf,
        pub sub_maps: Vec<TaskMap>,
        pub tasks: Vec<Task>,
    }

    impl TaskMap {
        pub fn get_all_with_path(&self) -> HashMap<Task, String> {
            let mut data: HashMap<Task, String> = HashMap::new();
            let path = self.path.as_path().iter().last().unwrap().to_str().unwrap();
            for i in &self.tasks {
                data.insert(i.clone(), format!("{}::{}", path, i.name).to_string());
            }
            for i in &self.sub_maps {
                for i in i.get_all_with_path() {
                    data.insert(i.0, format!("{}::{}", path, i.1).to_string());
                }
            }
            data
        }
        pub fn get_all(&self) -> HashMap<String, Task> {
            let mut data: HashMap<String, Task> = HashMap::new();
            for i in &self.tasks {
                data.insert(i.name.clone(), i.clone());
            }
            for i in &self.sub_maps {
                data.extend(i.get_all());
            }
            data
        }
    }

    impl Default for TaskMap {
        fn default() -> Self {
            TaskMap {
                path: Path::new("./").to_path_buf(),
                sub_maps: Vec::new(),
                tasks: Vec::new(),
            }
        }
    }

    pub fn get_task_map(file_path: &str, default_path: &str) -> TaskMap {
        match fs::read_to_string(file_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => TaskMap {
                path: Path::new(default_path).to_path_buf(),
                sub_maps: Vec::new(),
                tasks: Vec::new(),
            },
        }
    }

    pub fn write_task_map(task_map: &TaskMap, file_path: &str) -> io::Result<()> {
        let serialized = serde_json::to_string_pretty(task_map)?;
        fs::write(file_path, serialized)?;
        Ok(())
    }

    pub fn scan_tasks(path_map: &PathMap, task_map: &mut TaskMap) -> io::Result<()> {
        let mut file_to_scan: Option<&PathMap> = None;
        let mut directories_to_scan = Vec::new();

        for sub_map in &path_map.sub_maps {
            if sub_map.path.is_dir() {
                directories_to_scan.push(sub_map)
            } else if sub_map.path.ends_with("mod.rs") {
                file_to_scan = Some(sub_map)
            }
        }

        let mut visited_paths = Vec::new();

        'outer: for directory in directories_to_scan {
            visited_paths.push(directory.path.clone());
            for sub_map in &mut task_map.sub_maps {
                if sub_map.path == directory.path {
                    if directory.modified_state == ModifiedState::Changed {
                        scan_tasks(directory, sub_map)?;
                    }
                    continue 'outer;
                }
            }
            let mut new_task_map = TaskMap {
                path: directory.path.clone(),
                sub_maps: vec![],
                tasks: vec![],
            };
            scan_tasks(directory, &mut new_task_map)?;
            task_map.sub_maps.push(new_task_map);
        }
        task_map
            .sub_maps
            .retain(|item| visited_paths.contains(&item.path));

        if let Some(T) = file_to_scan {
            task_map.tasks = find_structs_with_task(T.path.as_path())
        } else {
            task_map.tasks = Vec::new()
        }
        Ok(())
    }

    fn find_structs_with_task(file_path: &Path) -> Vec<Task> {
        let content = match fs::read_to_string(file_path) {
            Ok(contents) => contents,
            Err(err) => {
                eprintln!("Failed to read file {}: {}", file_path.display(), err);
                return vec![];
            }
        };

        let syntax: File = match syn::parse_file(&content) {
            Ok(file) => file,
            Err(err) => {
                eprintln!("Failed to parse file {}: {}", file_path.display(), err);
                return vec![];
            }
        };

        let mut tasks: Vec<Task> = Vec::new();

        for item in syntax.items {
            if let Item::Fn(ItemFn {
                attrs, block, sig, ..
            }) = item
            {
                if has_task_attr(attrs) {
                    let outputs = get_task_output(block.stmts);
                    let inputs = get_task_input(sig.inputs);
                    tasks.push(Task {
                        name: sig.ident.to_string(),
                        input_archs: inputs.0,
                        input_resources: inputs.1,
                        input_states: inputs.2,
                        input_delta_time: inputs.3,
                        output_archs: outputs.0,
                        output_signals: outputs.1,
                        output_reset: outputs.2,
                    });
                }
            }
        }

        tasks
    }

    fn has_task_attr(attrs: Vec<Attribute>) -> bool {
        for attr in attrs {
            let tokens = attr.to_token_stream().to_string();
            if tokens.contains("task") {
                return true;
            }
        }
        false
    }

    pub fn get_task_input(
        token_stream: Punctuated<FnArg, Comma>,
    ) -> (
        Vec<(String, Vec<String>)>,
        Vec<(String, String)>,
        Vec<(String, String)>,
        Option<String>,
    ) {
        let mut input_arch: Vec<(String, Vec<String>)> = Vec::new();
        let mut input_resource: Vec<(String, String)> = Vec::new();
        let mut input_state: Vec<(String, String)> = Vec::new();
        let mut input_delta_time: Option<String> = None;

        for input in token_stream.iter() {
            if let FnArg::Typed(pat_type) = input {
                let name = match *pat_type.pat {
                    Pat::Ident(ref pat_ident) => pat_ident.ident.to_string(),
                    _ => "_".to_string(),
                };

                let ty = &*pat_type.ty;
                if let Type::Path(type_path) = ty {
                    if let Some(segment) = type_path.path.segments.last() {
                        if let PathArguments::AngleBracketed(generic_args) = &segment.arguments {
                            if let Some(GenericArgument::Type(inner_type)) =
                                generic_args.args.first()
                            {
                                if segment.ident == "Arch" {
                                    if let Type::Tuple(TypeTuple { elems, .. }) = inner_type {
                                        let elems: Vec<String> = elems
                                            .iter()
                                            .map(|elem| {
                                                elem.to_token_stream()
                                                    .to_string()
                                                    .replace(" ", "")
                                                    .replace("&", "")
                                            })
                                            .collect();
                                        input_arch.push((name, elems));
                                    } else {
                                        input_arch.push((
                                            name,
                                            vec![inner_type
                                                .to_token_stream()
                                                .to_string()
                                                .replace(" ", "")],
                                        ));
                                    }
                                    continue;
                                }

                                if segment.ident == "Res" {
                                    input_resource.push((
                                        name,
                                        inner_type.to_token_stream().to_string().replace(" ", ""),
                                    ));
                                    continue;
                                }

                                if segment.ident == "State" {
                                    input_state.push((
                                        name,
                                        inner_type.to_token_stream().to_string().replace(" ", ""),
                                    ));
                                    continue;
                                }
                            }
                        }
                    }
                    if type_path.to_token_stream().to_string() == "DeltaTime" {
                        input_delta_time = Some(name);
                        continue;
                    }
                }
            }
        }
        (input_arch, input_resource, input_state, input_delta_time)
    }

    struct MacroReader {
        output_arch: Vec<Vec<(String, String)>>,
        output_signals: Vec<String>,
        output_reset: bool,
    }

    impl<'ast> Visit<'ast> for MacroReader {
        // This method is called for every statement in the syntax tree.
        fn visit_stmt(&mut self, stmt: &'ast Stmt) {
            if let Stmt::Macro(mac) = stmt {
                match mac
                    .mac
                    .path
                    .segments
                    .last()
                    .unwrap()
                    .ident
                    .to_string()
                    .as_str()
                {
                    "add_entity" => {
                        let mut type_flag = true;
                        let mut ty: Vec<String> = vec!["".to_string()];
                        let mut va: Vec<String> = vec!["".to_string()];
                        for token in mac.mac.tokens.clone() {
                            let t = token.to_string().replace(" ", "");
                            if t == "," {
                                type_flag = true;
                                ty.push("".to_string());
                                va.push("".to_string());
                                continue;
                            }
                            if t == "=" {
                                type_flag = false;
                                continue;
                            }
                            if type_flag {
                                if let Some(last) = ty.last_mut() {
                                    last.push_str(&t);
                                }
                            } else {
                                if let Some(last) = va.last_mut() {
                                    last.push_str(&t);
                                }
                            }
                        }
                        let mut tuples: Vec<_> = ty.into_iter().zip(va.into_iter()).collect();

                        tuples.sort_by(|(a, _), (b, _)| a.cmp(b));

                        if self.output_arch.contains(&tuples) {
                            return;
                        }

                        self.output_arch.push(tuples);
                    }
                    "signal" => {
                        let signal: LitStr = mac.mac.parse_body().unwrap();

                        if self.output_signals.contains(&signal.value()) {
                            return;
                        }

                        self.output_signals.push(signal.value())
                    }
                    "reset" => {
                        self.output_reset = true;
                    }
                    _ => {}
                }
            };
            syn::visit::visit_stmt(self, stmt);
        }
    }

    impl MacroReader {
        fn new() -> MacroReader {
            MacroReader {
                output_arch: Vec::new(),
                output_signals: Vec::new(),
                output_reset: false,
            }
        }
    }

    fn get_task_output(stmts: Vec<Stmt>) -> (Vec<Vec<(String, String)>>, Vec<String>, bool) {
        let mut macro_reader = MacroReader::new();

        for st in stmts.iter() {
            macro_reader.visit_stmt(st);
        }
        (
            macro_reader.output_arch,
            macro_reader.output_signals,
            macro_reader.output_reset,
        )
    }
}
#[allow(non_snake_case)]
pub mod app_scan {
    use proc_macro2::{TokenStream, TokenTree};
    use quote::ToTokens;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::collections::{BinaryHeap, HashMap};
    use std::str::FromStr;
    use std::{fmt, fs, io};
    use syn::parse::{Parse, ParseBuffer, ParseStream, Result};
    use syn::token::Paren;
    use syn::{Error, Ident, Lit, LitStr, Token};

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone)]
    pub enum LogicalOperator {
        And,
        Or,
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone)]
    pub enum LogicalExpression {
        Signal(String),
        State(String, String),
        Operator(LogicalOperator),
        Not(Box<LogicalExpression>),
        Grouped(Vec<LogicalExpression>),
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Clone, Hash)]
    pub enum TaskType {
        Update,
        Fixed,
        Sync,
        Long,
        Setup,
    }
    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
    pub enum DependencyType {
        GroupStart(String),
        GroupEnd(String),
        Task(String),
    }
    impl fmt::Display for DependencyType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                DependencyType::GroupStart(s) => write!(f, "GroupStart({})", s),
                DependencyType::GroupEnd(s) => write!(f, "GroupEnd({})", s),
                DependencyType::Task(s) => write!(f, "Task({})", s),
            }
        }
    }
    impl Serialize for DependencyType {
        fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let s = match self {
                DependencyType::GroupStart(inner) => format!("GroupStart({})", inner),
                DependencyType::GroupEnd(inner) => format!("GroupEnd({})", inner),
                DependencyType::Task(inner) => format!("Task({})", inner),
            };
            serializer.serialize_str(&s)
        }
    }
    impl FromStr for DependencyType {
        type Err = String;
        fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
            if s.starts_with("GroupStart(") && s.ends_with(")") {
                let inner = &s["GroupStart(".len()..s.len() - 1];
                Ok(DependencyType::GroupStart(inner.to_string()))
            } else if s.starts_with("GroupEnd(") && s.ends_with(")") {
                let inner = &s["GroupEnd(".len()..s.len() - 1];
                Ok(DependencyType::GroupEnd(inner.to_string()))
            } else if s.starts_with("Task(") && s.ends_with(")") {
                let inner = &s["Task(".len()..s.len() - 1];
                Ok(DependencyType::Task(inner.to_string()))
            } else {
                Err(format!("Invalid dependency type string: {}", s))
            }
        }
    }
    impl<'de> Deserialize<'de> for DependencyType {
        fn deserialize<D>(deserializer: D) -> core::result::Result<DependencyType, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            DependencyType::from_str(&s).map_err(serde::de::Error::custom)
        }
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq)]
    pub struct DependencyGraph {
        pub dependents: HashMap<DependencyType, Vec<DependencyType>>,
        pub in_degrees: HashMap<DependencyType, usize>,
    }
    impl DependencyGraph {
        pub fn new() -> Self {
            DependencyGraph {
                dependents: HashMap::new(),
                in_degrees: HashMap::new(),
            }
        }

        pub fn add_node(&mut self, node: DependencyType) {
            self.dependents.entry(node.clone()).or_default();
            self.in_degrees.entry(node).or_insert(0);
        }

        pub fn add_dependency(&mut self, from: DependencyType, to: DependencyType) {
            self.dependents
                .entry(to.clone())
                .or_default()
                .push(from.clone());

            *self.in_degrees.entry(from.clone()).or_insert(0) += 1;
            self.in_degrees.entry(to.clone()).or_insert(0);
        }

        pub fn merge(&mut self, other: &Self) {
            for node in other.in_degrees.keys() {
                self.add_node(node.clone());
            }

            for (to, dependents) in &other.dependents {
                for from in dependents {
                    self.add_dependency(from.clone(), to.clone());
                }
            }
        }

        pub fn topological_sort(&self) -> core::result::Result<Vec<DependencyType>, &str> {
            let mut in_degrees = self.in_degrees.clone();
            let dependents = self.dependents.clone();
            let mut queue = BinaryHeap::new();

            for (node, &degree) in &in_degrees {
                if degree == 0 {
                    queue.push((degree, node.clone()));
                }
            }

            let mut sorted = Vec::new();
            while let Some((_, node)) = queue.pop() {
                sorted.push(node.clone());

                if let Some(dependent_nodes) = dependents.get(&node) {
                    for dependent in dependent_nodes {
                        if let Some(degree) = in_degrees.get_mut(dependent) {
                            *degree -= 1;
                            if *degree == 0 {
                                queue.push((*degree, dependent.clone()));
                            }
                        }
                    }
                }
            }

            if sorted.len() == self.in_degrees.len() {
                Ok(sorted)
            } else {
                Err("Circular dependency detected")
            }
        }

        pub fn get_task_leaves(&self, node: &DependencyType) -> Vec<&String> {
            let mut nodes: Vec<&String> = vec![];
            for node in &self.dependents[node] {
                if let DependencyType::Task(s) = node {
                    nodes.push(&s);
                } else {
                    nodes.extend(self.get_task_leaves(node));
                }
            }
            nodes
        }
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct AppPackage {
        pub name: String,
        pub path: String,
        pub setup_dependency: DependencyGraph,
        pub runtime_dependency: DependencyGraph,
        pub sync_dependency: DependencyGraph,
        pub tasks: HashMap<String, (TaskType, Option<LogicalExpression>)>,
        pub packages: Vec<String>,
    }
    impl Default for AppPackage {
        fn default() -> Self {
            AppPackage {
                name: "main".to_string(),
                path: "./src".to_string(),
                setup_dependency: DependencyGraph::new(),
                runtime_dependency: DependencyGraph::new(),
                sync_dependency: DependencyGraph::new(),
                tasks: HashMap::new(),
                packages: vec![],
            }
        }
    }

    pub fn write_app_package(app_package: &AppPackage, file_path: &str) -> io::Result<()> {
        let serialized = serde_json::to_string_pretty(app_package)?;
        fs::write(file_path, serialized)?;
        Ok(())
    }
    pub fn get_app_package(file_path: &str) -> AppPackage {
        match fs::read_to_string(file_path) {
            Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
            Err(_) => AppPackage::default(),
        }
    }

    impl Parse for LogicalExpression {
        fn parse(input: ParseStream) -> Result<Self> {
            let mut logical_expressions: Vec<LogicalExpression> = Vec::new();
            let mut is_not = false;
            while !input.is_empty() && !input.peek(Token![,]) {
                if input.peek(Paren) {
                    let content: ParseBuffer;
                    syn::parenthesized!(content in input);

                    if is_not {
                        logical_expressions.push(LogicalExpression::Not(Box::from(
                            content.parse::<LogicalExpression>()?,
                        )));
                        is_not = false;
                    } else {
                        logical_expressions.push(content.parse::<LogicalExpression>()?);
                    }
                }
                if input.peek(LitStr) {
                    let signal: LitStr = input.parse()?;

                    if is_not {
                        logical_expressions.push(LogicalExpression::Not(Box::from(
                            LogicalExpression::Signal(signal.value()),
                        )));
                        is_not = false;
                    } else {
                        logical_expressions.push(LogicalExpression::Signal(signal.value()));
                    }
                }
                if input.peek(Ident) {
                    let mut tokens: TokenStream = TokenStream::new();
                    tokens.extend(Some(input.parse::<TokenTree>()?));
                    let t = tokens.to_string();

                    while !input.is_empty()
                        && !input.peek(Token![,])
                        && !input.peek(Token![||])
                        && !input.peek(Token![&&])
                    {
                        let token_tree: TokenTree = input.parse()?;
                        tokens.extend(Some(token_tree));
                    }

                    if is_not {
                        logical_expressions.push(LogicalExpression::Not(Box::from(
                            LogicalExpression::State(t, tokens.to_string()),
                        )));
                        is_not = false;
                    } else {
                        logical_expressions.push(LogicalExpression::State(t, tokens.to_string()));
                    }
                }
                if input.peek(Token![!]) {
                    input.parse::<Token![!]>()?;
                    is_not = true;
                }
                if input.peek(Token![||]) {
                    input.parse::<Token![||]>()?;
                    logical_expressions.push(LogicalExpression::Operator(LogicalOperator::Or));
                }
                if input.peek(Token![&&]) {
                    input.parse::<Token![&&]>()?;
                    logical_expressions.push(LogicalExpression::Operator(LogicalOperator::And));
                }
            }

            Ok(LogicalExpression::Grouped(logical_expressions))
        }
    }

    impl Parse for AppPackage {
        fn parse(input: ParseStream) -> Result<Self> {
            if input.is_empty() {
                return Ok(AppPackage::default());
            }
            enum InternalTaskType {
                Runtime,
                Setup,
                Sync,
            }

            let mut app_package: AppPackage = AppPackage::default();
            let mut task_name: Option<(String, TaskType)> = None;
            let mut internal_task_type: InternalTaskType = InternalTaskType::Runtime;
            let mut nodes: Vec<DependencyType> = vec![];
            let mut dependencies: Vec<(DependencyType, DependencyType)> = vec![];

            let ident: Ident = match input.parse::<Ident>() {
                Ok(T) => T,
                Err(E) => {
                    return Err(Error::new_spanned(
                        E.into_compile_error(),
                        "Expected and Ident",
                    ));
                }
            };

            match ident.to_string().as_str() {
                "path" => match input.parse::<Lit>() {
                    Ok(Lit::Str(T)) => {
                        app_package.path = T.value();
                    }
                    T => {
                        return Err(Error::new_spanned(
                    match T {
                        Ok(T) => { T.to_token_stream() }
                        Err(E) => {E.into_compile_error()}
                    },
                    "String literal of the path of the package.\nExample: (path \"./src/lib\")"));
                    }
                },
                "update" => match input.parse::<Lit>() {
                    Ok(Lit::Str(T)) => task_name = Some((T.value(), TaskType::Update)),
                    T => {
                        return Err(Error::new_spanned(
                            match T {
                                Ok(T) => T.to_token_stream(),
                                Err(E) => E.into_compile_error(),
                            },
                            "String literal of name of a task.\nExample: (update \"normal_task\")",
                        ));
                    }
                },
                "fixed_update" => match input.parse::<Lit>() {
                    Ok(Lit::Str(T)) => task_name = Some((T.value(), TaskType::Fixed)),
                    T => {
                        return Err(Error::new_spanned(
                        match T {
                            Ok(T) => { T.to_token_stream() }
                            Err(E) => {E.into_compile_error()}
                        },
                        "String literal of name of a task.\nExample: (fixed_update \"fixed_task\")"));
                    }
                },
                "sync_update" => match input.parse::<Lit>() {
                    Ok(Lit::Str(T)) => {
                        internal_task_type = InternalTaskType::Sync;
                        task_name = Some((T.value(), TaskType::Sync));
                    }
                    T => {
                        return Err(Error::new_spanned(
                        match T {
                            Ok(T) => { T.to_token_stream() }
                            Err(E) => {E.into_compile_error()}
                        },
                        "String literal of name of a task.\nExample: (sync_update \"sync_task\")"));
                    }
                },
                "long_update" => match input.parse::<Lit>() {
                    Ok(Lit::Str(T)) => task_name = Some((T.value(), TaskType::Long)),
                    T => {
                        return Err(Error::new_spanned(
                        match T {
                            Ok(T) => { T.to_token_stream() }
                            Err(E) => {E.into_compile_error()}
                        },
                        "String literal of name of a task.\nExample: (long_update \"long_task\")"));
                    }
                },
                "setup" => match input.parse::<Lit>() {
                    Ok(Lit::Str(T)) => {
                        internal_task_type = InternalTaskType::Setup;
                        task_name = Some((T.value(), TaskType::Setup))
                    }
                    T => {
                        return Err(Error::new_spanned(
                            match T {
                                Ok(T) => T.to_token_stream(),
                                Err(E) => E.into_compile_error(),
                            },
                            "String literal of name of a task.\nExample: (setup \"setup_task\")",
                        ));
                    }
                },
                "group" => {
                    if input.peek(syn::Ident) {
                        match input.parse::<Ident>() {
                            Ok(T) => {
                                if (T.to_string().as_str()) == "setup" {
                                    internal_task_type = InternalTaskType::Setup
                                } else {
                                    return Err(Error::new_spanned(
                                T.to_token_stream(),
                                "Expected setup.\nExample: (group setup \"example_group\" before \"example_task\")"));
                                }
                            }
                            Err(E) => {
                                return Err(Error::new_spanned(
                                    E.into_compile_error(),
                                    "Expected sync or setup.\nExample: (group setup \"example_group\" before \"example_task\")",
                                ));
                            }
                        };
                    }

                    match input.parse::<Lit>() {
                        Ok(Lit::Str(J)) => {
                            let ident: Ident = match input.parse::<Ident>() {
                                Ok(T) => T,
                                Err(E) => {
                                    return Err(Error::new_spanned(
                                        E.into_compile_error(),
                                        "Expected an Ident",
                                    ));
                                }
                            };
                            match ident.to_string().as_str() {
                                "before" => match input.parse::<Lit>() {
                                    Ok(Lit::Str(T)) => {
                                        nodes.push(DependencyType::GroupStart(J.value()));
                                        nodes.push(DependencyType::GroupEnd(J.value()));
                                        nodes.push(DependencyType::Task(T.value()));
                                        dependencies.push((
                                            DependencyType::GroupEnd(J.value()),
                                            DependencyType::Task(T.value()),
                                        ));
                                    }
                                    T => {
                                        return Err(Error::new_spanned(
                                            match T {
                                                Ok(T) => { T.to_token_stream() }
                                                Err(E) => { E.into_compile_error() }
                                            },
                                            "String literal of name of a task.\nExample: (group \"example_group\" before \"example_task\")"));
                                    }
                                },
                                "before_group" => match input.parse::<Lit>() {
                                    Ok(Lit::Str(T)) => {
                                        nodes.push(DependencyType::GroupStart(J.value()));
                                        nodes.push(DependencyType::GroupEnd(J.value()));
                                        nodes.push(DependencyType::GroupStart(T.value()));
                                        nodes.push(DependencyType::GroupEnd(T.value()));
                                        dependencies.push((
                                            DependencyType::GroupEnd(J.value()),
                                            DependencyType::GroupStart(T.value()),
                                        ));
                                    }
                                    T => {
                                        return Err(Error::new_spanned(
                                            match T {
                                                Ok(T) => { T.to_token_stream() }
                                                Err(E) => { E.into_compile_error() }
                                            },
                                            "String literal of name of a group.\nExample: (group \"example_group\" before_group \"example_group\")"));
                                    }
                                },
                                "after" => match input.parse::<Lit>() {
                                    Ok(Lit::Str(T)) => {
                                        nodes.push(DependencyType::GroupStart(J.value()));
                                        nodes.push(DependencyType::GroupEnd(J.value()));
                                        nodes.push(DependencyType::Task(T.value()));
                                        dependencies.push((
                                            DependencyType::Task(T.value()),
                                            DependencyType::GroupStart(J.value()),
                                        ));
                                    }
                                    T => {
                                        return Err(Error::new_spanned(
                                            match T {
                                                Ok(T) => { T.to_token_stream() }
                                                Err(E) => { E.into_compile_error() }
                                            },
                                            "String literal of name of a task.\nExample: (group \"example_group\" after \"example_task\")"));
                                    }
                                },
                                "after_group" => match input.parse::<Lit>() {
                                    Ok(Lit::Str(T)) => {
                                        nodes.push(DependencyType::GroupStart(J.value()));
                                        nodes.push(DependencyType::GroupEnd(J.value()));
                                        nodes.push(DependencyType::GroupStart(T.value()));
                                        nodes.push(DependencyType::GroupEnd(T.value()));
                                        dependencies.push((
                                            DependencyType::GroupEnd(T.value()),
                                            DependencyType::GroupStart(J.value()),
                                        ));
                                    }
                                    T => {
                                        return Err(Error::new_spanned(
                                            match T {
                                                Ok(T) => { T.to_token_stream() }
                                                Err(E) => { E.into_compile_error() }
                                            },
                                            "String literal of name of a group.\nExample: (group \"example_group\" after_group \"example_group\")"));
                                    }
                                },
                                "in_group" => match input.parse::<Lit>() {
                                    Ok(Lit::Str(T)) => {
                                        nodes.push(DependencyType::GroupStart(J.value()));
                                        nodes.push(DependencyType::GroupEnd(J.value()));
                                        nodes.push(DependencyType::GroupStart(T.value()));
                                        nodes.push(DependencyType::GroupEnd(T.value()));
                                        dependencies.push((
                                            DependencyType::GroupStart(T.value()),
                                            DependencyType::GroupStart(J.value()),
                                        ));
                                        dependencies.push((
                                            DependencyType::GroupEnd(J.value()),
                                            DependencyType::GroupEnd(T.value()),
                                        ));
                                    }
                                    T => {
                                        return Err(Error::new_spanned(
                                            match T {
                                                Ok(T) => T.to_token_stream(),
                                                Err(E) => E.into_compile_error(),
                                            },
                                            "String literal of name of a group.\nExample: (group \"example_group\" in_group \"example_group\")",
                                        ));
                                    }
                                },
                                _ => {
                                    return Err(Error::new_spanned(
                                        ident,
                                        "Expected before, after, before_group, after_group or in_group.",
                                    ));
                                }
                            };
                        }
                        T => {
                            return Err(Error::new_spanned(
                                match T {
                                    Ok(T) => { T.to_token_stream() }
                                    Err(E) => { E.into_compile_error() }
                                },
                                "String literal of name of a group.\nExample: (group \"group_name\" before \"another_group\")"));
                        }
                    }
                }
                "package" => match input.parse::<Lit>() {
                    Ok(Lit::Str(T)) => app_package.packages.push(T.value()),
                    T => {
                        return Err(Error::new_spanned(
                        match T {
                            Ok(T) => { T.to_token_stream() }
                            Err(E) => {E.into_compile_error()}
                        },
                        "String literal of name of a package.\nExample: (package \"package_name\")"));
                    }
                },
                _ => {
                    return Err(Error::new_spanned(
                ident,
                "Expected path, update, fixed_update, sync_update, long_update, setup, group or package."));
                }
            }

            if let Some(J) = task_name {
                nodes.push(DependencyType::Task(J.0.clone()));
                if input.peek(syn::Ident) {
                    /*if J.1 == TaskType::Setup {
                        internal_task_type == true;
                    }*/

                    let ident: Ident = match input.parse::<Ident>() {
                        Ok(T) => T,
                        Err(E) => {
                            return Err(Error::new_spanned(
                                E.into_compile_error(),
                                "Expected before, after, before_group, after_group or in after",
                            ));
                        }
                    };

                    match ident.to_string().as_str() {
                        "before" => match input.parse::<Lit>() {
                            Ok(Lit::Str(T)) => {
                                nodes.push(DependencyType::Task(T.value()));
                                dependencies.push((
                                    DependencyType::Task(J.0.clone()),
                                    DependencyType::Task(T.value()),
                                ));
                            }
                            T => {
                                return Err(Error::new_spanned(
                            match T {
                                Ok(T) => { T.to_token_stream() }
                                Err(E) => {E.into_compile_error()}
                            },
                            "String literal of name of a task.\nExample: (update \"example_task\" before \"example_task\")"));
                            }
                        },
                        "before_group" => match input.parse::<Lit>() {
                            Ok(Lit::Str(T)) => {
                                nodes.push(DependencyType::GroupStart(T.value()));
                                nodes.push(DependencyType::GroupEnd(T.value()));
                                dependencies.push((
                                    DependencyType::Task(J.0.clone()),
                                    DependencyType::GroupStart(T.value()),
                                ));
                            }
                            T => {
                                return Err(Error::new_spanned(
                            match T {
                                Ok(T) => { T.to_token_stream() }
                                Err(E) => {E.into_compile_error()}
                            },
                            "String literal of name of a group.\nExample: (update \"example_task\" before_group \"example_group\")"));
                            }
                        },
                        "after" => match input.parse::<Lit>() {
                            Ok(Lit::Str(T)) => {
                                nodes.push(DependencyType::Task(T.value()));
                                dependencies.push((
                                    DependencyType::Task(T.value()),
                                    DependencyType::Task(J.0.clone()),
                                ));
                            }
                            T => {
                                return Err(Error::new_spanned(
                            match T {
                                Ok(T) => { T.to_token_stream() }
                                Err(E) => {E.into_compile_error()}
                            },
                            "String literal of name of a task.\nExample: (update \"example_task\" after \"example_task\")"));
                            }
                        },
                        "after_group" => match input.parse::<Lit>() {
                            Ok(Lit::Str(T)) => {
                                nodes.push(DependencyType::GroupStart(T.value()));
                                nodes.push(DependencyType::GroupEnd(T.value()));
                                dependencies.push((
                                    DependencyType::GroupEnd(T.value()),
                                    DependencyType::Task(J.0.clone()),
                                ));
                            }
                            T => {
                                return Err(Error::new_spanned(
                            match T {
                                Ok(T) => { T.to_token_stream() }
                                Err(E) => {E.into_compile_error()}
                            },
                            "String literal of name of a group.\nExample: (update \"example_task\" after_group \"example_group\")"));
                            }
                        },
                        "in_group" => match input.parse::<Lit>() {
                            Ok(Lit::Str(T)) => {
                                nodes.push(DependencyType::GroupStart(T.value()));
                                nodes.push(DependencyType::GroupEnd(T.value()));
                                dependencies.push((
                                    DependencyType::GroupStart(T.value()),
                                    DependencyType::Task(J.0.clone()),
                                ));
                                dependencies.push((
                                    DependencyType::Task(J.0.clone()),
                                    DependencyType::GroupEnd(T.value()),
                                ));
                            }
                            T => {
                                return Err(Error::new_spanned(
                            match T {
                                Ok(T) => T.to_token_stream(),
                                Err(E) => E.into_compile_error(),
                            },
                            "String literal of name of a group.\nExample: (Setup \"example_task\" in_group \"example_group\")",
                        ));
                            }
                        },
                        _ => {
                            return Err(Error::new_spanned(
                                ident,
                                "Expected before, after, before_group, after_group or in_group.",
                            ));
                        }
                    }
                }

                if input.peek(Token![if]) {
                    let _: Token![if] = input.parse()?;

                    app_package
                        .tasks
                        .insert(J.0, (J.1, Some(input.parse::<LogicalExpression>()?)));
                } else {
                    app_package.tasks.insert(J.0, (J.1, None));
                }
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
                let mut sub: AppPackage = input.parse()?;
                sub.packages.extend(app_package.packages);
                sub.tasks.extend(app_package.tasks);
                sub.path = app_package.path;
                sub.name = app_package.name;
                app_package = sub;
            }

            match internal_task_type {
                InternalTaskType::Setup => {
                    for node in nodes {
                        app_package.setup_dependency.add_node(node);
                    }
                    for dependency in dependencies {
                        app_package
                            .setup_dependency
                            .add_dependency(dependency.0, dependency.1)
                    }
                }
                InternalTaskType::Runtime => {
                    for node in nodes {
                        app_package.runtime_dependency.add_node(node);
                    }
                    for dependency in dependencies {
                        app_package
                            .runtime_dependency
                            .add_dependency(dependency.0, dependency.1)
                    }
                }
                InternalTaskType::Sync => {
                    for node in nodes {
                        app_package.sync_dependency.add_node(node);
                    }
                    for dependency in dependencies {
                        app_package
                            .sync_dependency
                            .add_dependency(dependency.0, dependency.1)
                    }
                }
            }

            Ok(app_package)
        }
    }
}
#[allow(non_snake_case)]
pub mod codegen {
    use crate::build::app_scan::{
        AppPackage, DependencyGraph, DependencyType, LogicalExpression, LogicalOperator, TaskType,
    };
    use crate::build::components_scan::ComponentMap;
    use crate::build::tasks_scan::{Task, TaskMap};
    use proc_macro2::{Span, TokenStream};
    use quote::quote;
    use std::collections::{HashMap, HashSet};
    use std::fmt::Debug;
    use std::fs::File;
    use std::io::Write;
    use std::{io, vec};
    use syn::spanned::Spanned;
    use syn::token::Semi;
    use syn::visit_mut::{self, VisitMut};
    use syn::{parse2, parse_str, LitStr, Stmt};

    #[derive(Debug)]
    pub struct ArchTypes {
        arch_types: Vec<Vec<String>>,
        tasks: HashMap<String, TasksInputOutput>,
        signals: HashSet<String>,
        resources: HashSet<String>,
        states: HashSet<String>,
    }
    #[derive(Debug)]
    pub struct TaskArchType {
        arch_type_type: Vec<String>,
        task_index: usize,
        input_arch_type_indexes: Vec<(usize, Vec<usize>)>,
    }
    #[derive(Debug)]
    pub struct TasksInputOutput {
        input: Vec<TaskArchType>,
        output: Vec<usize>,
    }

    pub fn create_app(
        app_packages: Vec<AppPackage>,
        task_maps: Vec<TaskMap>,
    ) -> (TokenStream, ArchTypes) {
        let mut tasks: HashMap<&String, Task> = HashMap::new();
        let mut task_options: HashMap<&String, &(TaskType, Option<LogicalExpression>)> =
            HashMap::new();
        let mut setup_dependency_map: DependencyGraph = DependencyGraph::new();
        let mut sync_dependency_map: DependencyGraph = DependencyGraph::new();
        let mut runtime_dependency_map: DependencyGraph = DependencyGraph::new();

        {
            let mut all_tasks: HashMap<String, Task> = task_maps
                .into_iter()
                .flat_map(|task_map| task_map.get_all())
                .collect();
            let mut packages: Vec<&str> = vec!["main"];
            let mut index = 0;

            while index < packages.len() {
                let package = packages[index];
                for app_package in &app_packages {
                    if package == app_package.name {
                        for v in &app_package.packages {
                            if !packages.contains(&v.as_str()) {
                                packages.push(v.as_str());
                            }
                        }
                        if setup_dependency_map.dependents.len() == 0 {
                            setup_dependency_map = app_package.setup_dependency.clone();
                        } else {
                            setup_dependency_map.merge(&app_package.setup_dependency);
                        }
                        if runtime_dependency_map.dependents.len() == 0 {
                            runtime_dependency_map = app_package.runtime_dependency.clone();
                        } else {
                            runtime_dependency_map.merge(&app_package.runtime_dependency);
                        }

                        if sync_dependency_map.dependents.len() == 0 {
                            sync_dependency_map = app_package.sync_dependency.clone();
                        } else {
                            sync_dependency_map.merge(&app_package.sync_dependency);
                        }
                        app_package.tasks.iter().for_each(|x| {
                            task_options.insert(x.0, x.1);
                            tasks.insert(
                                x.0,
                                all_tasks
                                    .remove(x.0)
                                    .unwrap_or_else(|| panic!("Tasks {} not defined", x.0)),
                            );
                        });
                    }
                }
                index += 1;
            }
        }
        let arch_types = get_all_archetypes(tasks.values().collect::<Vec<&Task>>());

        println!(
            "{}",
            generate_app_body(
                &tasks,
                &task_options,
                &setup_dependency_map,
                &sync_dependency_map,
                &runtime_dependency_map,
                &arch_types,
            )
            .to_string()
        );
        (
            generate_app_body(
                &tasks,
                &task_options,
                &setup_dependency_map,
                &sync_dependency_map,
                &runtime_dependency_map,
                &arch_types,
            ),
            arch_types,
        )
    }

    pub fn write_rust_file(token_stream: TokenStream, path: &str) -> io::Result<()> {
        let token_stream_str = token_stream.to_string();

        let mut file = File::create(path)?;
        file.write_all(token_stream_str.as_bytes())?;

        Ok(())
    }

    pub fn get_all_archetypes(tasks: Vec<&Task>) -> ArchTypes {
        let mut archetypes: ArchTypes = ArchTypes {
            arch_types: vec![],
            tasks: HashMap::new(),
            signals: Default::default(),
            resources: Default::default(),
            states: Default::default(),
        };

        for task in &tasks {
            for output_arch in &task.output_archs {
                let output = output_arch
                    .iter()
                    .map(|x| x.0.clone())
                    .collect::<Vec<String>>();
                if !archetypes
                    .arch_types
                    .contains(&output.iter().map(|x| x.clone()).collect::<Vec<String>>())
                {
                    archetypes.arch_types.push(output);
                }
            }
            archetypes.signals.extend(task.output_signals.clone());
            archetypes
                .resources
                .extend(task.input_resources.iter().map(|x| x.1.clone()));
            archetypes
                .states
                .extend(task.input_states.iter().map(|x| x.1.clone()));
        }
        for task in &tasks {
            let mut new = TasksInputOutput {
                input: vec![],
                output: task
                    .output_archs
                    .iter()
                    .map(|x| x.iter().map(|x| x.0.clone()).collect::<Vec<String>>())
                    .filter_map(|x| archetypes.arch_types.iter().position(|y| y == &x))
                    .collect::<Vec<usize>>(),
            };

            let mut index: usize = 0;

            for input_arch in &task.input_archs {
                new.input.push(TaskArchType {
                    arch_type_type: input_arch.1.clone(),
                    task_index: index,
                    input_arch_type_indexes: archetypes
                        .arch_types
                        .iter()
                        .enumerate()
                        .filter_map(|(outer_index, sub_vec)| {
                            if input_arch.1.iter().all(|b_elem| sub_vec.contains(b_elem)) {
                                Some((
                                    outer_index,
                                    input_arch
                                        .1
                                        .iter()
                                        .enumerate()
                                        .filter_map(|a| {
                                            if let Some(t) = sub_vec.iter().position(|i| i == a.1) {
                                                Some(t)
                                            } else {
                                                None
                                            }
                                        })
                                        .collect(),
                                ))
                            } else {
                                None
                            }
                        })
                        .collect(),
                });
                index += 1;
            }

            archetypes.tasks.insert(task.name.clone(), new);
        }
        archetypes
    }

    pub fn generate_prelude(
        component_map: Vec<(ComponentMap, String)>,
        task_map: Vec<(TaskMap, String)>,
    ) -> TokenStream {
        let mut code: TokenStream = TokenStream::new();

        for component in component_map {
            let all_components: HashMap<String, String> = component.0.get_all();
            if component.1 == "crate" {
                for component in all_components {
                    let name: TokenStream =
                        parse_str(component.1.as_str()).expect("Failed to parse component map");
                    code.extend(quote!(pub use crate::#name;).into_iter());
                }
            } else {
                let prefix: TokenStream = parse_str(component.1.replace("-", "_").as_str())
                    .expect("Failed to parse component map prefix");
                for component in all_components {
                    let name: TokenStream =
                        parse_str(component.1.as_str()).expect("Failed to parse component map");
                    code.extend(quote!(pub use #prefix::#name;).into_iter());
                }
            };
        }

        for task in task_map {
            let all_tasks: HashMap<Task, String> = task.0.get_all_with_path();
            if task.1 == "crate" {
                for task in all_tasks {
                    let name: TokenStream =
                        parse_str(task.1.as_str()).expect("Failed to parse component map");
                    code.extend(quote!(pub use crate::#name;).into_iter());
                }
            } else {
                let prefix: TokenStream = parse_str(task.1.replace("-", "_").as_str())
                    .expect("Failed to parse component map prefix");
                for task in all_tasks {
                    let name: TokenStream =
                        parse_str(task.1.as_str()).expect("Failed to parse component map");
                    code.extend(quote!(pub use #prefix::#name;).into_iter());
                }
            };
        }

        quote! {
            #code
            pub use crate::corrosive_engine::arch_types::*;
            pub use corrosive_ecs_core::ecs_core::{State, Res, Arch, Locked, LockedRef, Ref};
        }
    }

    pub fn generate_arch_types(arch_types: &ArchTypes) -> TokenStream {
        let mut code: TokenStream = TokenStream::new();

        for task in &arch_types.tasks {
            let exact_name = parse_str::<TokenStream>(format!("\"{}\"", &task.0).as_str()).unwrap();
            for input_arch_type in &task.1.input {
                let arch_type_name: TokenStream =
                    parse_str(format!("{}{}", &task.0, input_arch_type.task_index).as_str())
                        .unwrap();
                let mut arch_type_type: TokenStream = TokenStream::new();
                let mut members: TokenStream = TokenStream::new();
                let mut new_fn: TokenStream = TokenStream::new();
                let mut new_fn_len: TokenStream = TokenStream::new();
                let mut remove_fn: TokenStream = TokenStream::new();
                let mut iter_code: TokenStream = TokenStream::new();

                let mut index: usize = 0;

                //members
                for input_arch_type in &input_arch_type.input_arch_type_indexes {
                    let var_name: TokenStream = parse_str(format!("ve{}", index).as_str()).unwrap();
                    let mut var_types: TokenStream = TokenStream::new();
                    let var_remove_name: TokenStream =
                        parse_str(format!("rve{}", index).as_str()).unwrap();
                    let mut iter_types: TokenStream = TokenStream::new();

                    for input_arch_type in &arch_types.arch_types[input_arch_type.0] {
                        let val: TokenStream =
                            parse_str(format!("{},", input_arch_type).as_str()).unwrap();
                        var_types.extend(val);
                    }
                    for input_arch_type_index in &input_arch_type.1 {
                        let val: TokenStream =
                            parse_str(format!("{}", input_arch_type_index).as_str()).unwrap();
                        iter_types.extend(quote! {&self.#var_name[index].#val,});
                    }
                    iter_code.extend(quote! {
                    if index < self.#var_name.len() {
                        return Some((#iter_types));
                    };
                    index -= self.#var_name.len();
                                });

                    members.extend(quote! {#var_name: &'a Vec<(#var_types)>,});
                    members.extend(quote! {#var_remove_name: &'a RwLock<HashSet<usize>>,});
                    //remove_fn
                    remove_fn.extend(quote! {
                        if index < self.#var_name.len() {
                            self.#var_remove_name.write().unwrap().insert(index);
                            return;
                        };
                        index -= self.#var_name.len();
                    });
                    index += 1;
                }
                //new_fn
                for i in 0..index {
                    let val: TokenStream =
                        parse_str(format!("ve{},rve{},", i, i).as_str()).unwrap();
                    new_fn.extend(val);
                    if index == (i + 1) {
                        let val: TokenStream =
                            parse_str(format!("ve{}.len(),", i).as_str()).unwrap();
                        new_fn_len.extend(val);
                    } else {
                        let val: TokenStream =
                            parse_str(format!("ve{}.len() +", i).as_str()).unwrap();
                        new_fn_len.extend(val);
                    }
                }
                //arch_type_type
                for arch_type_name in &input_arch_type.arch_type_type {
                    let val: TokenStream =
                        parse_str(format!("{}", arch_type_name).as_str()).unwrap();
                    arch_type_type.extend(quote! {&'a #val,});
                }

                code.extend(quote! {
                            #[derive(Copy, Clone)]
                            pub struct #arch_type_name<'a> {
                            #members
                            len: usize,
                        }
                            impl<'a> #arch_type_name<'a> {
                    pub fn new(
                        #members
                    ) -> Self {
                        #arch_type_name {
                            #new_fn
                            len: #new_fn_len
                        }
                    }
                }
                impl<'a> EngineArch<(#arch_type_type)> for #arch_type_name<'a> {

                        fn remove(&self, mut index: usize) {
                        #remove_fn
                            eprintln!("Warning: index of out of {} is out of bounds",#exact_name);
                    }

                        fn len(&self) -> usize {
                            self.len
                        }

                    fn get_item(&self, mut index: usize) -> Option<(#arch_type_type)> {
                        #iter_code
                        None
                    }
                }
                                })
            }
        }

        quote! {
        use crate::corrosive_engine::auto_prelude::*;
        use corrosive_ecs_core::ecs_core::EngineArch;
        use std::collections::HashSet;
        use std::sync::RwLock;
                #code
                }
    }

    struct MacroReplacer {
        var_arch: Vec<Stmt>,
        var_signals: Vec<Stmt>,
        var_reset: Option<Stmt>,

        out_type: TokenStream,
        bool_num: usize,

        out_arch: TokenStream,
        out_signals: TokenStream,
        out_reset: TokenStream,

        index_arch: HashMap<Vec<(String, String)>, usize>,
        index_signals: HashMap<String, usize>,
        is_reset: bool,
    }

    impl MacroReplacer {
        fn new() -> MacroReplacer {
            MacroReplacer {
                var_arch: Vec::new(),
                var_signals: Vec::new(),
                var_reset: None,

                out_type: TokenStream::new(),
                bool_num: 0,

                out_arch: TokenStream::new(),
                out_signals: TokenStream::new(),
                out_reset: TokenStream::new(),

                index_arch: HashMap::new(),
                index_signals: HashMap::new(),
                is_reset: false,
            }
        }
    }

    impl VisitMut for MacroReplacer {
        fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
            // Process nested statements first
            visit_mut::visit_stmt_mut(self, stmt);

            let new_stmt: Stmt = if let Stmt::Macro(mac) = stmt {
                match mac
                    .mac
                    .path
                    .segments
                    .last()
                    .unwrap()
                    .ident
                    .to_string()
                    .as_str()
                {
                    "add_entity" => {
                        let mut type_flag = true;
                        let mut ty: Vec<String> = vec!["".to_string()];
                        let mut va: Vec<String> = vec!["".to_string()];
                        for token in mac.mac.tokens.clone() {
                            let t = token.to_string().replace(" ", "");
                            if t == "," {
                                type_flag = true;
                                ty.push("".to_string());
                                va.push("".to_string());
                                continue;
                            }
                            if t == "=" {
                                type_flag = false;
                                continue;
                            }
                            if type_flag {
                                if let Some(last) = ty.last_mut() {
                                    last.push_str(&t);
                                }
                            } else {
                                if let Some(last) = va.last_mut() {
                                    last.push_str(&t);
                                }
                            }
                        }
                        let mut tuples: Vec<_> = ty.into_iter().zip(va.into_iter()).collect();

                        tuples.sort_by(|(a, _), (b, _)| a.cmp(b));

                        let mut is_new = false;
                        if !self.index_arch.contains_key(&tuples) {
                            self.index_arch
                                .insert(tuples.clone(), self.index_arch.len());
                            is_new = true;
                        }

                        let mut vec_type = TokenStream::new();
                        let mut vec_input = TokenStream::new();
                        let vec_name: TokenStream = parse_str(
                            format!("engine_add_arch{}", self.index_arch[&tuples]).as_str(),
                        )
                        .unwrap();

                        for tuple in tuples {
                            let v: TokenStream = parse_str(tuple.1.as_str()).unwrap();
                            vec_input.extend(quote! {#v,});
                            if is_new {
                                let t: TokenStream = parse_str(tuple.0.as_str()).unwrap();
                                vec_type.extend(quote! {#t,});
                            }
                        }

                        if is_new {
                            self.var_arch.push(
                                parse2(quote! {let mut #vec_name: Vec<(#vec_type)> = Vec::new();})
                                    .expect("Failed to parse TokenStream into Stmt"),
                            );
                            self.out_arch.extend(quote! {#vec_name,});
                            self.out_type.extend(quote! {Vec<(#vec_type)>,})
                        }

                        parse2(quote! { #vec_name.push((#vec_input)); })
                            .expect("Failed to parse TokenStream into Stmt")
                    }
                    "signal" => {
                        let signal: LitStr = mac.mac.parse_body().unwrap();

                        let mut is_new = false;
                        if !self.index_signals.contains_key(&signal.value()) {
                            self.index_signals
                                .insert(signal.value(), self.index_signals.len());
                            is_new = true;
                            self.bool_num += 1;
                        }

                        let vec_name: TokenStream = parse_str(
                            format!(
                                "engine_trigger_signal{}",
                                self.index_signals[&signal.value()]
                            )
                            .as_str(),
                        )
                        .unwrap();

                        if is_new {
                            self.var_signals.push(
                                parse2(quote! {let mut #vec_name: bool = false;})
                                    .expect("Failed to parse TokenStream into Stmt"),
                            );
                            self.out_signals.extend(quote! {#vec_name,});
                        }

                        parse2(quote! { #vec_name = true; })
                            .expect("Failed to parse TokenStream into Stmt")
                    }
                    "reset" => {
                        if !self.is_reset {
                            self.var_reset = Some(
                                parse2(quote! {let mut engine_signal_trigger: bool = false;})
                                    .expect("Failed to parse TokenStream into Stmt"),
                            );
                            self.out_reset.extend(quote! {engine_signal_trigger,});
                            self.is_reset = true;
                            self.bool_num += 1;
                        }

                        parse2(quote! { engine_signal_trigger = true; })
                            .expect("Failed to parse TokenStream into Stmt")
                    }
                    _ => stmt.clone(),
                }
            } else {
                stmt.clone()
            };
            *stmt = new_stmt;
        }
    }

    pub fn generate_task_body(stmts: &mut Vec<Stmt>) -> TokenStream {
        let mut replacer: MacroReplacer = MacroReplacer::new();
        for stmt in stmts.iter_mut() {
            replacer.visit_stmt_mut(stmt);
        }

        let out_arch = replacer.out_arch;
        let out_signals = replacer.out_signals;
        let out_reset = replacer.out_reset;

        for i in replacer.var_arch {
            stmts.insert(0, i);
        }
        for i in replacer.var_signals {
            stmts.insert(0, i);
        }
        if let Some(t) = replacer.var_reset {
            stmts.insert(0, t);
        }

        if let Some(last_stmt) = stmts.last_mut() {
            if let Stmt::Expr(expr, None) = last_stmt {
                *last_stmt = Stmt::Expr(
                    expr.clone(),
                    Some(Semi {
                        spans: [expr.span()],
                    }),
                );
            }
        }
        let new_stmt: Stmt = syn::parse_quote! {return (#out_arch #out_signals #out_reset);};
        stmts.push(new_stmt);

        for _ in 0..replacer.bool_num {
            replacer.out_type.extend(quote! {bool, })
        }

        replacer.out_type
    }

    pub fn generate_app_body(
        all_tasks: &HashMap<&String, Task>,
        task_options: &HashMap<&String, &(TaskType, Option<LogicalExpression>)>,
        setup_dependency_map: &DependencyGraph,
        sync_dependency_map: &DependencyGraph,
        runtime_dependency_map: &DependencyGraph,
        arch_types: &ArchTypes,
    ) -> TokenStream {
        let variables = generate_app_variables(arch_types, task_options);
        let overwrite = generate_app_overwrite(arch_types);
        let mut runtime_bus = generate_bus_channels(runtime_dependency_map);
        let setup_bus = generate_bus_channels(setup_dependency_map);
        let mut runtime_tasks: TokenStream = TokenStream::new();
        let mut runtime_joins: TokenStream = TokenStream::new();
        let mut setup_tasks: TokenStream = TokenStream::new();
        let mut setup_joins: TokenStream = TokenStream::new();
        let mut sync_tasks: TokenStream = TokenStream::new();
        for task in generate_app_task_body(
            &all_tasks,
            &task_options,
            &arch_types,
            &runtime_dependency_map,
        ) {
            runtime_tasks.extend(task.1);
            let name: TokenStream = parse_str(format!("{}_end", task.0).as_str()).unwrap();
            let update_name: TokenStream = parse_str(format!("ut_{}", task.0).as_str()).unwrap();
            runtime_joins.extend(quote! {#name.read("failed");});
            runtime_bus.extend(quote! {let mut #update_name = loop_trigger.add_trigger();});
        }
        for task in generate_app_task_body(
            &all_tasks,
            &task_options,
            &arch_types,
            &setup_dependency_map,
        ) {
            let name: TokenStream = parse_str(format!("handle_{}", task.0).as_str()).unwrap();
            let task: TokenStream = task.1;
            setup_tasks.extend(quote! {let #name = #task});
            setup_joins.extend(quote! {#name.join().expect("TODO: panic message");})
        }
        for task in sync_dependency_map.topological_sort().unwrap() {
            let tasks = generate_app_task_body(
                &all_tasks,
                &task_options,
                &arch_types,
                &sync_dependency_map,
            );
            if let DependencyType::Task(v) = task {
                sync_tasks.extend(tasks[&v].clone())
            }
        }

        quote! {
            pub fn run_engine(){
            #variables
            let mut loop_trigger = Trigger::new();
            #runtime_bus
            thread::scope(|s: &Scope| {
                #runtime_tasks
                if reset.load(SeqCst) {
                    #setup_bus
                    thread::scope(|s: &Scope| {
                    reset.store(false, Ordering::SeqCst);
                    #setup_tasks
                    #setup_joins
                    });
                }
                loop{
                    #overwrite

                    current_time = Instant::now();
                    let new_current_time = current_time
                        .duration_since(last_time)
                        .as_secs_f64()
                        .to_bits();
                    delta_time.store(new_current_time.clone(), Ordering::Relaxed);
                    last_time = current_time;

                    fixed_delta_time += new_current_time;
                    if (fixed_time.load(Ordering::Relaxed) <= fixed_delta_time) {
                        fixed_delta_time = 0;
                        is_fixed.store(true, SeqCst);
                    } else {
                        is_fixed.store(false, SeqCst);
                    }

                    #sync_tasks

                    loop_trigger.trigger();

                    #runtime_joins
                }
            });}
        }
    }
    fn generate_app_task_body<'a>(
        tasks: &'a HashMap<&String, Task>,
        task_options: &'a HashMap<&String, &(TaskType, Option<LogicalExpression>)>,
        arch_types: &ArchTypes,
        dependency_graph: &'a DependencyGraph,
    ) -> HashMap<&'a String, TokenStream> {
        let mut task_codes: HashMap<&'a String, TokenStream> = HashMap::new();

        for task in &dependency_graph.dependents {
            let task_name = match task.0 {
                DependencyType::GroupStart(_) => continue,
                DependencyType::GroupEnd(_) => continue,
                DependencyType::Task(v) => v,
            };

            let task_name_code: TokenStream = parse_str(format!("{}", task_name).as_str()).unwrap();

            let mut code: TokenStream = TokenStream::new();

            //call function
            for t in &arch_types.tasks[task_name].input {
                let arch_name: TokenStream =
                    parse_str(format!("{}{}", task_name, t.task_index).as_str()).unwrap();
                let mut arch_inputs: TokenStream = TokenStream::new();
                for input_arch_type_index in &t.input_arch_type_indexes {
                    let name: TokenStream =
                        parse_str(format!("a{}", input_arch_type_index.0).as_str()).unwrap();
                    let remove: TokenStream =
                        parse_str(format!("or{}", input_arch_type_index.0).as_str()).unwrap();

                    arch_inputs.extend(quote! {&*#name.read().unwrap(),});
                    arch_inputs.extend(quote! {&#remove,});
                }

                code.extend(quote! {
                    Arch::new(&mut #arch_name::new(
                    #arch_inputs
                    )),
                });
            }

            for input_resource in &tasks[task_name].input_resources {
                let resource_name: TokenStream =
                    parse_str(format!("r_{}", input_resource.1).as_str()).unwrap();
                code.extend(quote! {Res::new(&#resource_name),})
            }

            for input_state in &tasks[task_name].input_states {
                let state_name: TokenStream =
                    parse_str(format!("st_{}", input_state.1).as_str()).unwrap();
                code.extend(quote! {State::new(&#state_name),})
            }

            if tasks[task_name].input_delta_time != None {
                code.extend(quote! {&f64::from_bits(delta_time.load(Ordering::Relaxed)),});
            }

            code = quote! {
                let o = #task_name_code(
                    #code
                );
            };

            //output
            let mut index: usize = 0;

            for output in &arch_types.tasks[task_name].output {
                let name: TokenStream = parse_str(format!("o.{}", index).as_str()).unwrap();
                let arch_name: TokenStream = parse_str(format!("o{}", output).as_str()).unwrap();

                code.extend(quote! {
                    (&#arch_name).write().unwrap().extend(#name);
                });
                index += 1;
            }

            for task in &tasks[task_name].output_signals {
                let name: TokenStream = parse_str(format!("o.{}", index).as_str()).unwrap();
                let signal_name: TokenStream = parse_str(format!("s_{}", task).as_str()).unwrap();

                code.extend(quote! {
                    if #name {
                        #signal_name.store(#name, Ordering::Relaxed);
                    }
                });
                index += 1
            }

            if tasks[task_name].output_reset {
                let name: TokenStream = parse_str(format!("o.{}", index).as_str()).unwrap();

                code.extend(quote! {
                    if #name {
                        reset.store(#name, Ordering::Relaxed);
                    }
                });
            }

            //condition
            if let Some(t) = &task_options[task_name].1 {
                let c = t.get_code();
                code = quote! {
                    if #c{
                        #code
                    }
                };
            }

            if task_options[task_name].0 == TaskType::Long {
                let mut lock_add_code: TokenStream = TokenStream::new();
                let mut lock_sub_code: TokenStream = TokenStream::new();

                let mut lock_names: HashSet<String> = HashSet::new();
                for t in &arch_types.tasks[task_name].input {
                    for input_arch_type_index in &t.input_arch_type_indexes {
                        lock_names.insert(format!("la{}", input_arch_type_index.0));
                    }
                }
                for lock_name in lock_names {
                    let lock_name = parse_str::<TokenStream>(lock_name.as_str()).unwrap();
                    lock_add_code.extend(quote! {#lock_name.fetch_add(1, Ordering::SeqCst);});
                    lock_sub_code.extend(quote! {#lock_name.fetch_sub(1, Ordering::SeqCst);});
                }

                code = quote! {
                    match lock.take() {
                        Some(task) if task.is_finished() => {
                            task.join().expect("Task finished but failed to join");
                        }
                        Some(task) => {
                            lock = Some(task);
                        }
                        None => {
                            lock = Some(s.spawn(|| {
                                #lock_add_code

                                #code

                                #lock_sub_code
                            }));
                        }
                    }
                };
            }

            if task_options[task_name].0 == TaskType::Fixed {
                code = quote! {
                    if is_fixed.load(SeqCst) {
                        #code
                    }
                }
            }

            //dependency
            if task_options[task_name].0 != TaskType::Sync {
                let start_signal = if task_options[task_name].0 == TaskType::Update
                    || task_options[task_name].0 == TaskType::Long
                    || task_options[task_name].0 == TaskType::Fixed
                {
                    let name: TokenStream =
                        parse_str(format!("ut_{}", task_name).as_str()).unwrap();
                    quote! {#name.read("failed");}
                } else {
                    TokenStream::new()
                };

                let end_signal = {
                    let name: TokenStream =
                        parse_str(format!("bus_{}", task_name).as_str()).unwrap();
                    quote! {#name.trigger();}
                };

                let mut dependency: TokenStream = TokenStream::new();

                for get_task_leaf in dependency_graph.get_task_leaves(task.0) {
                    let name: TokenStream =
                        parse_str(format!("{}_{}", task_name, get_task_leaf).as_str()).unwrap();
                    dependency.extend(quote! {#name.read("failed");});
                }

                let long_task_handle: TokenStream = if task_options[task_name].0 == TaskType::Long {
                    quote! {let mut lock: Option<ScopedJoinHandle<_>> = None::<ScopedJoinHandle<'_, _>>;}
                } else {
                    quote! {}
                };

                if task_options[task_name].0 == TaskType::Setup
                    || task_options[task_name].0 == TaskType::Sync
                {
                    code = quote! {
                        s.spawn(|| {
                            #start_signal
                            #dependency
                            #code
                            #end_signal
                        });
                    }
                } else {
                    code = quote! {
                        s.spawn(|| {
                            #long_task_handle
                            loop {
                                #start_signal
                                #dependency
                                #code
                                #end_signal
                            }
                        });
                    }
                }
            }

            task_codes.insert(task_name, code);
        }

        task_codes
    }
    fn generate_bus_channels(dependency_graph: &DependencyGraph) -> TokenStream {
        let mut trigger_code: TokenStream = TokenStream::new();
        let mut bus_code: TokenStream = TokenStream::new();

        for dependency in &dependency_graph.dependents {
            if let DependencyType::Task(v) = dependency.0 {
                let trigger: TokenStream = parse_str(format!("bus_{}", v).as_str()).unwrap();
                let trigger_end: TokenStream = parse_str(format!("{}_end", v).as_str()).unwrap();

                trigger_code.extend(quote! {let mut #trigger_end = #trigger.add_trigger();});
                bus_code.extend(quote! {let mut #trigger = Trigger::new();});

                for task_leaf in dependency_graph.get_task_leaves(dependency.0) {
                    let trigger: TokenStream =
                        parse_str(format!("{}_{}", v, task_leaf).as_str()).unwrap();

                    let buss: TokenStream =
                        parse_str(format!("bus_{}", task_leaf).as_str()).unwrap();

                    trigger_code.extend(quote! {let mut #trigger = #buss.add_trigger();});
                }
            }
        }
        quote! {
            #bus_code
            #trigger_code
        }
    }
    impl LogicalExpression {
        pub fn get_code(&self) -> TokenStream {
            match self {
                LogicalExpression::Grouped(v) => {
                    let mut code: TokenStream = TokenStream::new();
                    for value in v {
                        code.extend(value.get_code());
                    }
                    quote! {(#code)}
                }
                LogicalExpression::Signal(v) => {
                    let v: TokenStream = parse_str(format!("s_{}", v).as_str()).unwrap();
                    quote! {#v.load(Ordering::Relaxed)}
                }
                LogicalExpression::State(n, t) => {
                    let n: TokenStream = parse_str(format!("st_{}", n).as_str()).unwrap();
                    let t: TokenStream = parse_str(t.as_str()).unwrap();
                    quote! {*#n.read().unwrap() == #t}
                }
                LogicalExpression::Not(v) => {
                    let v = v.get_code();
                    quote! {!#v}
                }
                LogicalExpression::Operator(v) => match v {
                    LogicalOperator::And => quote! {&&},
                    LogicalOperator::Or => quote! {||},
                },
            }
        }
        pub fn get_states(&self) -> HashSet<&String> {
            let mut values: HashSet<&String> = HashSet::new();
            if let LogicalExpression::State(t, _) = self {
                values.insert(t);
            }
            if let LogicalExpression::Grouped(v) = self {
                for v in v {
                    values.extend(v.get_states());
                }
            }
            values
        }
        pub fn get_signals(&self) -> HashSet<&String> {
            let mut values: HashSet<&String> = HashSet::new();
            if let LogicalExpression::Signal(t) = self {
                values.insert(t);
            }
            if let LogicalExpression::Grouped(v) = self {
                for v in v {
                    values.extend(v.get_signals());
                }
            }
            values
        }
    }
    fn generate_app_variables(
        arch_types: &ArchTypes,
        task_options: &HashMap<&String, &(TaskType, Option<LogicalExpression>)>,
    ) -> TokenStream {
        let mut arch_code = TokenStream::new();
        let mut index: usize = 0;

        let mut signals: HashSet<&String> = HashSet::new();
        let mut states: HashSet<&String> = HashSet::new();

        for arch_type in &arch_types.arch_types {
            let name: TokenStream = parse_str(format!("a{}", index).as_str()).unwrap();
            let overwrite_name: TokenStream = parse_str(format!("o{}", index).as_str()).unwrap();
            let remove_name: TokenStream = parse_str(format!("or{}", index).as_str()).unwrap();
            let lock_name: TokenStream = parse_str(format!("la{}", index).as_str()).unwrap();

            let mut c = TokenStream::new();
            for arch in arch_type {
                c.extend(parse_str::<TokenStream>(format!("{},", arch).as_str()).unwrap());
            }
            arch_code.extend(quote! {
                let #name: RwLock<Vec<(#c)>> = RwLock::new(Vec::new());
                let #overwrite_name: RwLock<Vec<(#c)>> = RwLock::new(Vec::new());
                let #remove_name: RwLock<HashSet<usize>> = RwLock::new(HashSet::new());
                let #lock_name: AtomicU8 = AtomicU8::new(0);
            });
            index += 1;
        }

        for task_option in task_options {
            if let Some(T) = &task_option.1 .1 {
                signals.extend(T.get_signals());
                states.extend(T.get_states());
            }
        }

        for signal in &arch_types.signals {
            signals.insert(signal);
        }
        for signal in signals {
            let name: TokenStream = parse_str(format!("s_{}", signal).as_str()).unwrap();
            let overwrite_name: TokenStream = parse_str(format!("so_{}", signal).as_str()).unwrap();

            arch_code.extend(quote! {
                let #name: AtomicBool = AtomicBool::new(false);
                let #overwrite_name: AtomicBool = AtomicBool::new(false);
            });
        }

        for state in &arch_types.states {
            states.insert(state);
        }
        for state in states {
            let name: TokenStream = parse_str(format!("st_{}", state).as_str()).unwrap();
            let t: TokenStream = parse_str(state.as_str()).unwrap();

            arch_code.extend(quote! {
                let #name: RwLock<#t> = RwLock::new(#t::default());
            });
        }

        for resource in &arch_types.resources {
            let name: TokenStream = parse_str(format!("r_{}", resource).as_str()).unwrap();
            let t: TokenStream = parse_str(resource.as_str()).unwrap();

            arch_code.extend(quote! {
                let #name: RwLock<#t> = RwLock::new(#t::default());
            });
        }
        quote! {
            use crate::corrosive_engine::auto_prelude::{*};
            use corrosive_ecs_core::ecs_core::{*};
            use std::cmp::PartialEq;
            use std::collections::HashSet;
            use std::mem::take;
            use std::sync::atomic::Ordering::SeqCst;
            use std::sync::atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering};
            use std::sync::RwLock;
            use std::thread;
            use std::thread::{Scope, ScopedJoinHandle};
            use std::time::Instant;
            use corrosive_ecs_core_macro::corrosive_engine_builder;
            use std::sync::mpsc;

            let mut last_time = Instant::now();
            let mut current_time = Instant::now();
            let delta_time = AtomicU64::new(0.0f64.to_bits());

            let fixed_time = AtomicU64::new(0.1f64.to_bits());
            let mut fixed_delta_time: u64 = 0.0f64 as u64;
            let is_fixed = AtomicBool::new(false);

            let reset: AtomicBool = AtomicBool::new(true);

            #arch_code
        }
    }
    fn generate_app_overwrite(arch_types: &ArchTypes) -> TokenStream {
        let mut overwrite_thread_code: TokenStream = TokenStream::new();
        let mut overwrite_join_code: TokenStream = TokenStream::new();
        let mut signal_code: TokenStream = TokenStream::new();

        for i in 0..arch_types.arch_types.len() {
            let thread_name: TokenStream = parse_str(format!("m_{}", i).as_str()).unwrap();
            let arch_name: TokenStream = parse_str(format!("a{}", i).as_str()).unwrap();
            let overwrite_name: TokenStream = parse_str(format!("o{}", i).as_str()).unwrap();
            let remove_name: TokenStream = parse_str(format!("or{}", i).as_str()).unwrap();
            let lock_name: TokenStream = parse_str(format!("la{}", i).as_str()).unwrap();
            let mut expire: TokenStream = TokenStream::new();

            for j in 0..arch_types.arch_types[i].len() {
                if arch_types.arch_types[i][j].starts_with("Ref<")
                    || arch_types.arch_types[i][j].starts_with("LockedRef<")
                {
                    let index: TokenStream = parse_str(format!("{}", j).as_str()).unwrap();
                    expire.extend(quote! {item.#index.expire();})
                }
            }

            overwrite_thread_code.extend(quote! {
                let #thread_name = s.spawn(|| {
                    if #lock_name.load(Ordering::SeqCst) > 0 {
                        return;
                        }
                    let mut write = #arch_name.write().unwrap();
                    let vlen = write.len();

                    if vlen > 0 {
                        let indices_to_remove = take(&mut *#remove_name.write().unwrap());
                        let mut new = Vec::with_capacity(vlen);

                        for (i, mut item) in write.drain(..).enumerate() {
                            if !indices_to_remove.contains(&i) {
                                new.push(item);
                                continue;
                            }
                            #expire
                        }

                        *write = new;
                    }
                    write.extend(#overwrite_name.write().unwrap().drain(..));
                });
            });

            let error: LitStr = LitStr::new(
                format!(
                    "Failed to update archetype of type -> {:?}",
                    arch_types.arch_types[i]
                )
                .as_str(),
                Span::call_site(),
            );
            overwrite_join_code.extend(quote! {#thread_name.join().expect(#error);});
        }

        for signal in &arch_types.signals {
            let signal_name: TokenStream = parse_str(format!("s_{}", signal).as_str()).unwrap();
            let overwrite_name: TokenStream = parse_str(format!("so_{}", signal).as_str()).unwrap();

            signal_code.extend(quote! {
                #signal_name.store(#overwrite_name.load(Ordering::Relaxed), Ordering::Relaxed);
                #overwrite_name.store(false, Ordering::Relaxed);
            })
        }

        quote! {
            #overwrite_thread_code
            #signal_code
            #overwrite_join_code
        }
    }
}
