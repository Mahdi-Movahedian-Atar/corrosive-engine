
use crate::build::general_scan::{ModifiedState, PathMap};
    use quote::ToTokens;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::{fs, io};
    use syn::punctuated::Punctuated;
    use syn::token::Comma;
    use syn::{Attribute, File, FnArg, GenericArgument, Item, ItemFn, Pat, PathArguments, ReturnType, Type, TypeTuple};

    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
    pub struct Task {
        pub name: String,
        pub inputs: Vec<TaskInput>,
        pub outputs: Vec<TaskOutput>,
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
    pub enum MemberType {
        Normal(String),
        Trait(String),
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
    pub enum TaskInput {
        Arch(String, Vec<MemberType>),
        Resources(String, String),
        Hierarchy(String, String),
        State(String, String),
        DeltaTime(String),
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug, Clone, Eq, Hash, PartialEq)]
    pub enum TaskOutput {
        Arch(Vec<String>),
        Signal,
        Reset,
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
                                attrs,
                                block: _,
                                sig,
                                ..
                            }) = item
            {
                if has_task_attr(attrs) {
                    let outputs = get_task_output(sig.output);
                    let inputs = get_task_input(sig.inputs);
                    tasks.push(Task {
                        name: sig.ident.to_string(),
                        inputs,
                        outputs,
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

    pub fn get_task_input(token_stream: Punctuated<FnArg, Comma>) -> Vec<TaskInput> {
        let mut inputs: Vec<TaskInput> = Vec::new();

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
                                        let elems: Vec<MemberType> = elems
                                            .iter()
                                            .map(|elem| {
                                                let val = elem.to_token_stream().to_string();
                                                if val.starts_with("& dyn ") {
                                                    MemberType::Trait(
                                                        val.replace("dyn ", "")
                                                            .replace(" ", "")
                                                            .replace("&", ""),
                                                    )
                                                } else {
                                                    MemberType::Normal(
                                                        val.replace(" ", "").replace("&", ""),
                                                    )
                                                }
                                            })
                                            .collect();
                                        inputs.push(TaskInput::Arch(name, elems));
                                    } else {
                                        let val = inner_type.to_token_stream().to_string();
                                        let new_val: MemberType;
                                        if val.starts_with("& dyn ") {
                                            new_val = MemberType::Trait(
                                                val.replace("dyn ", "")
                                                    .replace(" ", "")
                                                    .replace("&", ""),
                                            )
                                        } else {
                                            new_val = MemberType::Normal(
                                                val.replace(" ", "").replace("&", ""),
                                            )
                                        }
                                        inputs.push(TaskInput::Arch(name, vec![new_val]));
                                    }
                                    continue;
                                }

                                if segment.ident == "Res" {
                                    inputs.push(TaskInput::Resources(
                                        name,
                                        inner_type.to_token_stream().to_string().replace(" ", ""),
                                    ));
                                    continue;
                                }

                                if segment.ident == "State" {
                                    inputs.push(TaskInput::State(
                                        name,
                                        inner_type.to_token_stream().to_string().replace(" ", ""),
                                    ));
                                    continue;
                                }
                                if segment.ident == "Hierarchy" {
                                    inputs.push(TaskInput::Hierarchy(
                                        name,
                                        inner_type.to_token_stream().to_string().replace(" ", ""),
                                    ));
                                    continue;
                                }
                            }
                        }
                    }
                    if type_path.to_token_stream().to_string() == "DeltaTime" {
                        inputs.push(TaskInput::DeltaTime(name));
                        continue;
                    }
                }
            }
        }
        inputs
    }

    fn get_task_output(return_type: ReturnType) -> Vec<TaskOutput> {
        let mut outputs: Vec<TaskOutput> = Vec::new();

        if let ReturnType::Type(_, t) = return_type {
            if let Type::Tuple(t) = *t {
                for elem in t.elems {
                    if let Type::Path(t) = elem {
                        for segment in t.path.segments {
                            match segment.ident.to_string().as_str() {
                                "RArch" => {
                                    if let PathArguments::AngleBracketed(a) = segment.arguments {
                                        for arg in a.args {
                                            if let GenericArgument::Type(Type::Tuple(elem)) = arg {
                                                outputs.push(TaskOutput::Arch(
                                                    elem.elems
                                                        .iter()
                                                        .map(|x| {
                                                            x.to_token_stream()
                                                                .to_string()
                                                                .replace(" ", "")
                                                        })
                                                        .collect::<Vec<String>>(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                "Signal" => outputs.push(TaskOutput::Signal),
                                "Reset" => outputs.push(TaskOutput::Reset),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        outputs
    }