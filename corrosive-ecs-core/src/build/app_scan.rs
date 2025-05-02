
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
                            Ok(T) => T.to_token_stream(),
                            Err(E) => E.into_compile_error(),
                        },
                        "String literal of the path of the package.\nExample: (path \"./src/lib\")",
                    ));
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
                            Ok(T) => T.to_token_stream(),
                            Err(E) => E.into_compile_error(),
                        },
                        "String literal of name of a task.\nExample: (fixed_update \"fixed_task\")",
                    ));
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
                            Ok(T) => T.to_token_stream(),
                            Err(E) => E.into_compile_error(),
                        },
                        "String literal of name of a task.\nExample: (sync_update \"sync_task\")",
                    ));
                }
            },
            "long_update" => match input.parse::<Lit>() {
                Ok(Lit::Str(T)) => task_name = Some((T.value(), TaskType::Long)),
                T => {
                    return Err(Error::new_spanned(
                        match T {
                            Ok(T) => T.to_token_stream(),
                            Err(E) => E.into_compile_error(),
                        },
                        "String literal of name of a task.\nExample: (long_update \"long_task\")",
                    ));
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
                            Ok(T) => T.to_token_stream(),
                            Err(E) => E.into_compile_error(),
                        },
                        "String literal of name of a package.\nExample: (package \"package_name\")",
                    ));
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
