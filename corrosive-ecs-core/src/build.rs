use quote::ToTokens;
use std::cmp::PartialEq;
use std::io::Write;

pub const IGNORE_DIR: &str = ".corrosive-components";

pub mod general_helper {
    use std::fs::OpenOptions;

    pub fn log_message(message: String) {}
}

pub mod general_scan {
    use crate::build::IGNORE_DIR;
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
            for mut m in &mut self.sub_maps {
                m.modified_state = ModifiedState::Removed;
                m.remove()
            }
        }

        fn none(&mut self) {
            self.modified_state = ModifiedState::None;
            for mut m in &mut self.sub_maps {
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
        if path_map.path.as_path().is_dir() && !path_map.path.as_path().ends_with(IGNORE_DIR) {
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
                .for_each(|mut item| item.remove());
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

pub mod tasks_scan {
    use crate::build::general_scan::{ModifiedState, PathMap};
    use quote::ToTokens;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};
    use std::{fs, io};
    use syn::punctuated::Punctuated;
    use syn::token::Comma;
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

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct TaskMap {
        pub path: PathBuf,
        pub sub_maps: Vec<TaskMap>,
        pub tasks: Vec<Task>,
    }

    impl TaskMap {
        pub fn get_all(&self) -> HashMap<Task, String> {
            let mut data: HashMap<Task, String> = HashMap::new();
            let path = self.path.as_path().iter().last().unwrap().to_str().unwrap();
            for i in &self.tasks {
                data.insert(i.clone(), format!("{}::{}", path, i.name).to_string());
            }
            for i in &self.sub_maps {
                for i in i.get_all() {
                    data.insert(i.0, format!("{}::{}", path, i.1).to_string());
                }
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
                                        let mut elems: Vec<String> = elems
                                            .iter()
                                            .map(|elem| {
                                                elem.to_token_stream().to_string().replace(" ", "")
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
    fn get_task_output(stmts: Vec<Stmt>) -> (Vec<Vec<(String, String)>>, Vec<String>, bool) {
        let mut output_arch: Vec<Vec<(String, String)>> = Vec::new();
        let mut output_signals: Vec<String> = Vec::new();
        let mut output_reset: bool = false;

        for st in stmts {
            if let Stmt::Macro(mac) = st {
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
                        for token in mac.mac.tokens {
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

                        if output_arch.contains(&tuples) {
                            break;
                        }

                        output_arch.push(tuples);
                    }
                    "signal" => {
                        let signal: LitStr = mac.mac.parse_body().unwrap();

                        if output_signals.contains(&signal.value()) {
                            break;
                        }

                        output_signals.push(signal.value())
                    }
                    "reset" => {
                        output_reset = true;
                    }
                    _ => {}
                }
            }
        }
        (output_arch, output_signals, output_reset)
    }
}

pub mod codegen {
    use crate::build::tasks_scan::{Task, TaskMap};
    use proc_macro2::{Ident, Punct, Spacing, Span, TokenStream};
    use quote::{quote, quote_spanned, ToTokens, TokenStreamExt};
    use serde_json::to_string;
    use std::collections::{HashMap, HashSet};
    use std::fmt::Debug;
    use std::fs::File;
    use std::io::Write;
    use std::ops::Index;
    use std::{fs, io, vec};
    use syn::spanned::Spanned;
    use syn::{parse, parse2, parse_quote, parse_str, Expr, LitStr, ReturnType, Stmt, Token};

    #[derive(Debug)]
    pub struct ArchTypes {
        arch_types: Vec<Vec<String>>,
        tasks: HashMap<String, Vec<TaskArchType>>,
    }
    #[derive(Debug)]
    pub struct TaskArchType {
        arch_type_type: Vec<String>,
        task_index: usize,
        input_arch_type_indexes: Vec<(usize, Vec<usize>)>,
    }

    pub fn write_rust_file(token_stream: TokenStream, path: &str) -> io::Result<()> {
        let token_stream_str = token_stream.to_string();

        let mut file = File::create(path)?;
        file.write_all(token_stream_str.as_bytes())?;

        Ok(())
    }

    pub fn get_all_archetypes(all_tasks: &HashMap<Task, String>) -> ArchTypes {
        let mut archetypes: ArchTypes = ArchTypes {
            arch_types: vec![],
            tasks: HashMap::new(),
        };

        for task in all_tasks {
            for output_arch in &task.0.output_archs {
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
        }
        for key in all_tasks.keys() {
            let mut index: usize = 0;
            for input_arch in &key.input_archs {
                let Hash_archs: HashSet<&str> = input_arch.1.iter().map(|s| s.as_str()).collect();

                match archetypes.tasks.get_mut(&key.name) {
                    Some(T) => T.push(TaskArchType {
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
                                                if let Some(t) =
                                                    sub_vec.iter().position(|i| i == a.1)
                                                {
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
                    }),
                    _ => {
                        archetypes.tasks.insert(
                            key.name.clone(),
                            vec![TaskArchType {
                                arch_type_type: input_arch.1.clone(),
                                task_index: index,
                                input_arch_type_indexes: archetypes
                                    .arch_types
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(outer_index, sub_vec)| {
                                        if input_arch
                                            .1
                                            .iter()
                                            .all(|b_elem| sub_vec.contains(b_elem))
                                        {
                                            Some((
                                                outer_index,
                                                input_arch
                                                    .1
                                                    .iter()
                                                    .enumerate()
                                                    .filter_map(|a| {
                                                        if let Some(t) =
                                                            sub_vec.iter().position(|i| i == a.1)
                                                        {
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
                            }],
                        );
                    }
                };
                index += 1;
            }
        }
        archetypes
    }

    pub fn generate_prelude(
        all_components: &HashMap<String, String>,
        all_tasks: &HashMap<Task, String>,
    ) -> TokenStream {
        let mut code: TokenStream = TokenStream::new();

        for component in all_components {
            let name: TokenStream =
                parse_str(component.1.as_str()).expect("Failed to parse component map");
            code.extend(quote!(pub use crate::#name;).into_iter());
        }
        for task in all_tasks {
            let name: TokenStream =
                parse_str(task.1.as_str()).expect("Failed to parse component map");
            code.extend(quote!(pub use crate::#name;).into_iter());
        }

        quote! {
            pub mod prelude {
            #code
            pub use crate::corrosive_engine::arch_types::arch_types::*;
            pub use corrosive_ecs_core::ecs_core::{Locked, LockedRef, Ref};
            }
        }
    }
    pub fn generate_arch_types(arch_types: &ArchTypes) -> TokenStream {
        let mut code: TokenStream = TokenStream::new();

        for task in &arch_types.tasks {
            for input_arch_type in task.1 {
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
                        iter_types.extend(quote! {&self.#var_name[current_index].#val,});
                    }
                    iter_code.extend(quote! {
                    if current_index < self.#var_name.len() {
                        return Some((#iter_types));
                    };
                    current_index -= self.#var_name.len();
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
                            pub len: usize,
                            pub v_i: usize,
                        }
                            impl<'a> #arch_type_name<'a> {
                    pub fn new(
                        #members
                    ) -> Self {
                        #arch_type_name {
                            #new_fn
                            len: #new_fn_len
                            v_i: 0,
                        }
                    }

                    pub fn remove(&self, mut index: usize) {
                        #remove_fn
                    }
                }
                impl<'a> Iterator for #arch_type_name<'a> {
                    type Item = (#arch_type_type);

                    fn next(&mut self) -> Option<Self::Item> {
                        let mut current_index = self.v_i.clone();
                        self.v_i += 1;
                        #iter_code
                        None
                    }
                }
                                })
            }
        }

        quote! {
                pub mod arch_types {
        use crate::corrosive_engine::auto_prelude::prelude::*;
        use std::collections::HashSet;
        use std::sync::RwLock;
                #code
                }
            }
    }

    pub fn generate_task_body(stmts: &mut Vec<Stmt>) -> TokenStream {
        let mut var_arch: Vec<Stmt> = Vec::new();
        let mut var_signals: Vec<Stmt> = Vec::new();
        let mut var_reset: Option<Stmt> = None;

        let mut out_type: TokenStream = TokenStream::new();
        let mut bool_num: usize = 0;

        let mut out_arch: TokenStream = TokenStream::new();
        let mut out_signals: TokenStream = TokenStream::new();
        let mut out_reset: TokenStream = TokenStream::new();

        let mut index_arch: HashMap<Vec<(String, String)>, usize> = HashMap::new();
        let mut index_signals: HashMap<String, usize> = HashMap::new();
        let mut is_reset = false;

        for i in 0..stmts.len() {
            if let Stmt::Macro(mac) = &stmts[i] {
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
                        if !index_arch.contains_key(&tuples) {
                            index_arch.insert(tuples.clone(), index_arch.len());
                            is_new = true;
                        }

                        let mut vec_type = TokenStream::new();
                        let mut vec_input = TokenStream::new();
                        let mut vec_name: TokenStream =
                            parse_str(format!("engine_add_arch{}", index_arch[&tuples]).as_str())
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
                            var_arch.push(
                                parse2(quote! {let mut #vec_name: Vec<(#vec_type)> = Vec::new();})
                                    .expect("Failed to parse TokenStream into Stmt"),
                            );
                            out_arch.extend(quote! {#vec_name,});
                            out_type.extend(quote! {Vec<(#vec_type)>,})
                        }

                        stmts[i] = parse2(quote! { #vec_name.push((#vec_input)); })
                            .expect("Failed to parse TokenStream into Stmt");
                    }
                    "signal" => {
                        let signal: LitStr = mac.mac.parse_body().unwrap();

                        let mut is_new = false;
                        if !index_signals.contains_key(&signal.value()) {
                            index_signals.insert(signal.value(), index_signals.len());
                            is_new = true;
                            bool_num += 1;
                        }

                        let mut vec_name: TokenStream = parse_str(
                            format!("engine_trigger_signal{}", index_signals[&signal.value()])
                                .as_str(),
                        )
                        .unwrap();

                        if is_new {
                            var_signals.push(
                                parse2(quote! {let mut #vec_name: bool = false;})
                                    .expect("Failed to parse TokenStream into Stmt"),
                            );
                            out_signals.extend(quote! {#vec_name,});
                        }

                        stmts[i] = parse2(quote! { #vec_name = true; })
                            .expect("Failed to parse TokenStream into Stmt");
                    }
                    "reset" => {
                        if !is_reset {
                            var_reset = Some(
                                parse2(quote! {let mut engine_signal_trigger: bool = false;})
                                    .expect("Failed to parse TokenStream into Stmt"),
                            );
                            out_reset.extend(quote! {engine_signal_trigger,});
                            is_reset = true;
                            bool_num += 1;
                        }

                        stmts[i] = parse2(quote! { engine_signal_trigger = true; })
                            .expect("Failed to parse TokenStream into Stmt");
                    }
                    _ => {}
                }
            }
        }

        let mut var = var_arch;
        var.extend(var_signals);

        if let Some(t) = var_reset {
            var.push(t);
        }

        for s in var {
            stmts.insert(0, s);
        }
        stmts.push(
            parse2(quote! {return (#out_arch #out_signals # out_reset); })
                .expect("Failed to parse TokenStream into Stmt"),
        );

        for _ in 0..bool_num {
            out_type.extend(quote! {bool, })
        }

        out_type
    }
}
