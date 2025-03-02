use crate::comp::RenderGraph;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// A trait representing a render graph node.
/// Each node can record commands into a given command encoder.
pub type CommandEncoder = wgpu::CommandEncoder;
pub type Device = wgpu::Device;
pub type Queue = wgpu::Queue;

pub trait RenderNode: Send + Sync {
    /// Returns the unique name of this node.
    fn name(&self) -> &str;

    /// Records the commands for this node into the provided encoder.
    fn execute(&self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder);
}

/// A struct representing a node in the render graph.
pub(crate) struct GraphNode {
    node: Box<dyn RenderNode>,
}

/// A simple render graph that holds nodes and dependency relationships.

impl RenderGraph {
    /// Create a new, empty render graph.
    pub fn new() -> Self {
        Self {
            pass_names: HashMap::new(),
            pass_nodes: HashMap::new(),
            edges: Vec::new(),
            sorted: Vec::new(),
        }
    }

    /// Adds a node to the graph.
    pub fn add_node(&mut self, node: Box<dyn RenderNode>) {
        let key = if self.pass_names.contains_key(node.name()) {
            self.pass_names.get(node.name()).unwrap().clone()
        } else {
            let i = self.pass_names.len();
            self.pass_names.insert(node.name().to_string(), i);
            i
        };
        self.pass_nodes.insert(key, GraphNode { node });
    }

    pub fn add_dependency(&mut self, parent: &str, child: &str) {
        self.edges
            .push((self.pass_names[parent], self.pass_names[child]));
    }

    /// Executes the render graph in parallel for independent nodes.
    pub fn execute(&self, device: &Device, queue: &wgpu::Queue) {
        let mut execution_levels: Vec<Vec<usize>> = Vec::new();
        let mut visited = HashSet::new();

        // Group nodes into levels of independent execution.
        for node in &self.sorted {
            if !visited.contains(node) {
                let mut level = vec![node.clone()];
                visited.insert(node.clone());

                for other in &self.sorted {
                    if &other != &node && !self.depends_on_index(other, &node) {
                        level.push(other.clone());
                        visited.insert(other.clone());
                    }
                }
                execution_levels.push(level);
            }
        }

        let command_buffers_mutex = Arc::new(Mutex::new(Vec::new()));

        for level in execution_levels {
            level.par_iter().for_each(|node_name| {
                if let Some(graph_node) = self.pass_nodes.get(&node_name) {
                    let mut local_encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some(&format!("Encoder for {}", node_name)),
                        });

                    graph_node.node.execute(device, queue, &mut local_encoder);

                    let commands = local_encoder.finish();
                    command_buffers_mutex.lock().unwrap().push(commands);
                }
            });
        }

        // Submit all command buffers in one batch
        let command_buffers = Arc::try_unwrap(command_buffers_mutex)
            .unwrap()
            .into_inner()
            .unwrap();
        queue.submit(command_buffers);
    }

    /// Performs a topological sort using Kahn's algorithm.
    /// Returns the names of nodes in execution order.
    fn topological_sort(&self) -> Vec<usize> {
        let mut in_degree: HashMap<usize, usize> =
            self.pass_nodes.keys().map(|k| (k.clone(), 0)).collect();
        for (_parent, child) in &self.edges {
            if let Some(count) = in_degree.get_mut(child) {
                *count += 1;
            }
        }

        let mut queue: Vec<usize> = in_degree
            .iter()
            .filter(|(_, &deg)| deg == 0)
            .map(|(k, _)| k.clone())
            .collect();
        let mut order = Vec::new();
        let mut local_in_degree = in_degree.clone();

        while let Some(n) = queue.pop() {
            order.push(n.clone());
            for (parent, child) in &self.edges {
                if parent == &n {
                    if let Some(count) = local_in_degree.get_mut(child) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push(child.clone());
                        }
                    }
                }
            }
        }
        order
    }

    /// Checks if `child` depends on `parent` in the graph.
    fn depends_on(&self, child: &String, parent: &String) -> bool {
        let child = &self.pass_names[child];
        let parent = &self.pass_names[parent];
        self.edges.iter().any(|(p, c)| p == parent && c == child)
    }
    fn depends_on_index(&self, child: &usize, parent: &usize) -> bool {
        self.edges.iter().any(|(p, c)| p == parent && c == child)
    }

    pub fn prepare(&mut self) {
        self.sorted = self.topological_sort();
    }
}
