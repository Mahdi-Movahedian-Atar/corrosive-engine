
    use crate::build::general_scan::{ModifiedState, PathMap};
    use proc_macro2::Ident;
    use quote::ToTokens;
    use std::collections::{HashMap, HashSet};
    use std::path::{Path, PathBuf};
    use std::{fs, io};
    use syn::parse::{Parse, ParseStream};
    use syn::{parse2, File, Item, ItemEnum, ItemStruct, ItemTrait, ItemType, Token, Type};

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub enum ComponentType {
        Component(String),
        Trait(String),
        TraitFor(String, HashSet<String>),
    }
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct ComponentMap {
        pub path: PathBuf,
        pub sub_maps: Vec<ComponentMap>,
        pub components: Vec<ComponentType>,
    }

    impl ComponentMap {
        pub fn get_trait_to_components(&self) -> HashMap<String, HashSet<String>> {
            let mut data: HashMap<String, HashSet<String>> = HashMap::new();
            for i in &self.components {
                match i {
                    ComponentType::TraitFor(a, b) => {
                        if let Some(t) = data.get_mut(a) {
                            t.extend(b.clone());
                        } else {
                            data.insert(a.clone(), b.clone());
                        }
                    }
                    _ => {}
                }
            }
            for i in &self.sub_maps {
                for i in i.get_trait_to_components() {
                    if let Some(t) = data.get_mut(&i.0) {
                        t.extend(i.1);
                    } else {
                        data.insert(i.0, i.1);
                    }
                }
            }
            data
        }
        pub fn get_all(&self) -> HashMap<String, String> {
            let mut data: HashMap<String, String> = HashMap::new();
            let path = self.path.as_path().iter().last().unwrap().to_str().unwrap();
            for i in &self.components {
                match i {
                    ComponentType::Component(i) => {
                        data.insert(i.clone(), format!("{}::{}", path, i).to_string());
                    }
                    ComponentType::Trait(i) => {
                        data.insert(i.clone(), format!("{}::{}", path, i).to_string());
                    }
                    ComponentType::TraitFor(_, _) => {}
                }
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

    fn find_structs_with_component(file_path: &Path) -> Vec<ComponentType> {
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

        let mut names: Vec<ComponentType> = Vec::new();

        for item in syntax.items {
            match item {
                Item::Struct(ItemStruct { attrs, ident, .. }) => {
                    for attr in attrs {
                        if attr.path().is_ident("derive") {
                            let tokens = attr.to_token_stream().to_string();
                            if tokens.contains("Component")
                                || tokens.contains("State")
                                || tokens.contains("Resource")
                            {
                                names.push(ComponentType::Component(ident.to_string()));
                            }
                        }
                    }
                }
                Item::Enum(ItemEnum { attrs, ident, .. }) => {
                    for attr in attrs {
                        if attr.path().is_ident("derive") {
                            let tokens = attr.to_token_stream().to_string();
                            if tokens.contains("Component")
                                || tokens.contains("State")
                                || tokens.contains("Resource")
                            {
                                names.push(ComponentType::Component(ident.to_string()));
                            }
                        }
                    }
                }
                Item::Type(ItemType { attrs, ident, .. }) => {
                    for attr in attrs {
                        if attr.path().is_ident("derive") {
                            let tokens = attr.to_token_stream().to_string();
                            if tokens.contains("Component")
                                || tokens.contains("State")
                                || tokens.contains("Resource")
                            {
                                names.push(ComponentType::Component(ident.to_string()));
                            }
                        }
                    }
                }
                Item::Trait(ItemTrait { attrs, ident, .. }) => {
                    for attr in attrs {
                        if attr.path().is_ident("trait_bound") {
                            names.push(ComponentType::Trait(ident.to_string()));
                            break;
                        }
                    }
                }
                Item::Macro(ref macro_item) => {
                    if macro_item.mac.path.segments.last().unwrap().ident == "trait_for" {
                        let tokens = macro_item.mac.tokens.clone();
                        let data: HelperParser =
                            parse2(tokens).expect("Failed to parse trait_for input");
                        let trait_name = data.trait_name.to_string();
                        let types = data
                            .types
                            .iter()
                            .map(|ty| ty.to_token_stream().to_string().replace(" ", ""))
                            .collect::<HashSet<String>>();
                        names.push(ComponentType::TraitFor(trait_name, types));
                    }
                }
                _ => {}
            }
        }

        names
    }
    struct HelperParser {
        _trait_kw: Token![trait],
        trait_name: Ident,
        _fat_arrow: Token![=>],
        types: syn::punctuated::Punctuated<Type, Token![,]>,
    }

    impl Parse for HelperParser {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            Ok(Self {
                _trait_kw: input.parse()?,
                trait_name: input.parse()?,
                _fat_arrow: input.parse()?,
                types: syn::punctuated::Punctuated::parse_terminated(input)?,
            })
        }
    }