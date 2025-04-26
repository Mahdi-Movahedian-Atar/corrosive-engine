
    use crate::build::app_scan::{
        AppPackage, DependencyGraph, DependencyType, LogicalExpression, LogicalOperator, TaskType,
    };
    use crate::build::components_scan::ComponentMap;
    use crate::build::tasks_scan::{MemberType, Task, TaskInput, TaskMap, TaskOutput};
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
        resources: HashSet<String>,
        states: HashSet<String>,
        hierarchy: HashSet<String>,
    }
    #[derive(Debug)]
    pub struct TaskArchType {
        arch_type_type: Vec<MemberType>,
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
        trait_to_components: HashMap<String, HashSet<String>>,
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
        let arch_types =
            get_all_archetypes(tasks.values().collect::<Vec<&Task>>(), trait_to_components);

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

    pub fn get_all_archetypes(
        tasks: Vec<&Task>,
        trait_to_components: HashMap<String, HashSet<String>>,
    ) -> ArchTypes {
        let mut archetypes: ArchTypes = ArchTypes {
            arch_types: vec![],
            tasks: HashMap::new(),
            resources: Default::default(),
            states: Default::default(),
            hierarchy: Default::default(),
        };

        for task in &tasks {
            for output in &task.outputs {
                match output {
                    TaskOutput::Arch(v) => {
                        let mut v = v.clone();
                        v.sort();
                        archetypes.arch_types.push(v)
                    }
                    _ => {}
                }
            }
            for input in &task.inputs {
                match input {
                    TaskInput::Resources(_, v) => {
                        archetypes.resources.insert(v.clone());
                    }
                    TaskInput::State(_, v) => {
                        archetypes.states.insert(v.clone());
                    }
                    TaskInput::Hierarchy(_, v) => {
                        archetypes.hierarchy.insert(v.clone());
                    }
                    _ => {}
                }
            }
        }

        for task in &tasks {
            let mut new = TasksInputOutput {
                input: vec![],
                output: task
                    .outputs
                    .iter()
                    .filter_map(|x| {
                        if let TaskOutput::Arch(v) = x {
                            let mut v = v.clone();
                            v.sort();
                            Some(v)
                        } else {
                            None
                        }
                    })
                    .filter_map(|x| archetypes.arch_types.iter().position(|y| y == &x))
                    .collect::<Vec<usize>>(),
            };

            let mut index: usize = 0;

            for input_arch in &task.inputs {
                if let TaskInput::Arch(_, input_arch) = input_arch {
                    let value_lists: Vec<Vec<String>> = input_arch
                        .iter()
                        .map(|key| match key {
                            MemberType::Normal(t) => {
                                vec![t.clone()]
                            }
                            MemberType::Trait(t) => trait_to_components
                                .get(t)
                                .expect("All keys must be present in the HashMap")
                                .iter()
                                .map(|x| x.clone())
                                .collect(),
                        })
                        .collect();

                    let mut combinations: Vec<Vec<&String>> = vec![vec![]];

                    for values in &value_lists {
                        combinations = combinations
                            .into_iter()
                            .flat_map(|combo| {
                                values.iter().map(move |value| {
                                    let mut new_combo = combo.clone();
                                    new_combo.push(value);
                                    new_combo
                                })
                            })
                            .collect();
                    }

                    let mut index_data: Vec<(usize, Vec<usize>)> = Vec::new();

                    for combination in combinations {
                        index_data.extend(
                            archetypes
                                .arch_types
                                .iter()
                                .enumerate()
                                .filter_map(|(outer_index, sub_vec)| {
                                    if combination.iter().all(|b_elem| sub_vec.contains(b_elem)) {
                                        Some((
                                            outer_index,
                                            combination
                                                .iter()
                                                .enumerate()
                                                .filter_map(|a| {
                                                    if let Some(t) =
                                                        sub_vec.iter().position(|i| &i == a.1)
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
                                .collect::<Vec<_>>(),
                        )
                    }

                    new.input.push(TaskArchType {
                        arch_type_type: input_arch.clone(),
                        task_index: index,
                        input_arch_type_indexes: index_data,
                    });
                    index += 1;
                }
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
            pub use corrosive_ecs_core::ecs_core::{State, Res, Arch, Locked, LockedRef, Ref, Member, Hierarchy};
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
                if input_arch_type.input_arch_type_indexes.is_empty() {
                    panic!("{} does not contain any archetype.", task.0)
                }
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
                    let val: TokenStream = parse_str(
                        match arch_type_name {
                            MemberType::Normal(t) => {
                                format!("{}", t)
                            }
                            MemberType::Trait(t) => {
                                format!("dyn {}", t)
                            }
                        }
                            .as_str(),
                    )
                        .unwrap();
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
                    "Reset" => {
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

            let mut arch_types_index: usize = 0;
            for task in &tasks[task_name].inputs {
                match task {
                    TaskInput::Arch(_, _) => {
                        let t = &arch_types.tasks[task_name].input[arch_types_index];
                        let arch_name: TokenStream =
                            parse_str(format!("{}{}", task_name, t.task_index).as_str()).unwrap();
                        let mut arch_inputs: TokenStream = TokenStream::new();
                        for input_arch_type_index in &t.input_arch_type_indexes {
                            let name: TokenStream =
                                parse_str(format!("a{}", input_arch_type_index.0).as_str())
                                    .unwrap();
                            let remove: TokenStream =
                                parse_str(format!("or{}", input_arch_type_index.0).as_str())
                                    .unwrap();

                            arch_inputs.extend(quote! {&*#name.read().unwrap(),});
                            arch_inputs.extend(quote! {&#remove,});
                        }

                        code.extend(quote! {
                            Arch::new(&mut #arch_name::new(
                            #arch_inputs
                            )),
                        });
                        arch_types_index += 1;
                    }
                    TaskInput::Resources(_, v) => {
                        let resource_name: TokenStream = parse_str(
                            format!("r_{}", v)
                                .replace("<", "")
                                .replace(">", "")
                                .as_str(),
                        )
                            .unwrap();
                        code.extend(quote! {#resource_name.clone(),})
                    }
                    TaskInput::State(_, v) => {
                        let state_name: TokenStream = parse_str(
                            format!("st_{}", v)
                                .replace("<", "")
                                .replace(">", "")
                                .as_str(),
                        )
                            .unwrap();
                        code.extend(quote! {#state_name.clone(),})
                    }
                    TaskInput::Hierarchy(_, v) => {
                        let Hierarch_name: TokenStream =
                            parse_str(format!("h_{}", v).as_str()).unwrap();
                        code.extend(quote! {#Hierarch_name.clone(),})
                    }
                    TaskInput::DeltaTime(_) => {
                        code.extend(quote! {&f64::from_bits(delta_time.load(Ordering::Relaxed)),});
                    }
                }
            }

            code = quote! {
                let o = #task_name_code(
                    #code
                );
            };

            //output
            let mut index: usize = 0;
            let mut arch_types_index: usize = 0;

            for task in &tasks[task_name].outputs {
                match task {
                    TaskOutput::Arch(v) => {
                        let output = &arch_types.tasks[task_name].output[arch_types_index];
                        let name: TokenStream = parse_str(format!("o.{}", index).as_str()).unwrap();
                        let arch_name: TokenStream =
                            parse_str(format!("o{}", output).as_str()).unwrap();

                        let mut map_left: TokenStream = TokenStream::new();
                        let mut map_right: TokenStream = TokenStream::new();
                        let mut map_index: usize = 0;

                        let mut sorted = v.clone();
                        sorted.sort();

                        for val in &sorted {
                            let left_name: TokenStream =
                                parse_str(format!("m{}", map_index).as_str()).unwrap();
                            let right_name: TokenStream = parse_str(
                                format!("m{}", v.iter().position(|x| x == val).unwrap()).as_str(),
                            )
                                .unwrap();
                            map_left.extend(quote! {#left_name,});
                            map_right.extend(quote! {#right_name,});
                            map_index += 1
                        }

                        code.extend(quote! {
                            (&#arch_name).write().unwrap().extend(#name.vec.into_iter().map(|(#map_left)| (#map_right)));
                        });
                        index += 1;
                        arch_types_index += 1;
                    }
                    TaskOutput::Signal => {
                        let name: TokenStream = parse_str(format!("o.{}", index).as_str()).unwrap();

                        code.extend(quote! {
                            o_signals.write().unwrap().extend(#name.vec);
                        });
                        index += 1
                    }
                    TaskOutput::Reset => {
                        let name: TokenStream = parse_str(format!("o.{}", index).as_str()).unwrap();

                        code.extend(quote! {
                            if #name.get() {
                                reset.store(#name.get(), Ordering::Relaxed);
                            }
                        });
                    }
                }
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
                    let v: TokenStream = parse_str(format!("\"{}\"", v).as_str()).unwrap();
                    quote! {signals.read().unwrap().contains(#v)}
                }
                LogicalExpression::State(n, t) => {
                    let n: TokenStream = parse_str(
                        format!("st_{}", n)
                            .replace("<", "")
                            .replace(">", "")
                            .as_str(),
                    )
                        .unwrap();
                    let t: TokenStream = parse_str(t.as_str()).unwrap();
                    quote! {*#n.f_read() == #t}
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
    }
    fn generate_app_variables(
        arch_types: &ArchTypes,
        task_options: &HashMap<&String, &(TaskType, Option<LogicalExpression>)>,
    ) -> TokenStream {
        let mut arch_code = TokenStream::new();
        let mut index: usize = 0;

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
                states.extend(T.get_states());
            }
        }

        for state in &arch_types.states {
            states.insert(state);
        }
        for state in states {
            let name: TokenStream = parse_str(
                format!("st_{}", state)
                    .replace("<", "")
                    .replace(">", "")
                    .as_str(),
            )
                .unwrap();
            let t: TokenStream = parse_str(state.as_str()).unwrap();

            arch_code.extend(quote! {
                let #name: State<#t> = State::new(Default::default());
            });
        }

        for resource in &arch_types.resources {
            let name: TokenStream = parse_str(
                format!("r_{}", resource)
                    .replace("<", "")
                    .replace(">", "")
                    .as_str(),
            )
                .unwrap();
            let t: TokenStream = parse_str(resource.as_str()).unwrap();

            arch_code.extend(quote! {
                let #name: Res<#t> = Res::new(Default::default());
            });
        }

        for hierarchy in &arch_types.hierarchy {
            let name: TokenStream = parse_str(format!("h_{}", hierarchy).as_str()).unwrap();
            let t: TokenStream = parse_str(hierarchy.as_str()).unwrap();

            arch_code.extend(quote! {
                let #name: Hierarchy<#t> = Hierarchy::default();
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

            let mut signals = RwLock::new(HashSet::<String>::new());
            let mut o_signals = RwLock::new(HashSet::<String>::new());
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
                    || arch_types.arch_types[i][j].starts_with("Member<")
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

        quote! {
            #overwrite_thread_code
            signals.write().unwrap().extend(o_signals.write().unwrap().drain());
            *o_signals.write().unwrap() = HashSet::new();
            #overwrite_join_code
        }
    }