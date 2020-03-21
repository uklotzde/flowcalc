use crate::{node::*, port::*};

use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// TODO
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Socket(pub NodeIndex, pub PortIndex);

/// TODO
#[derive(Debug)]
pub struct FlowNode<T> {
    node: Rc<RefCell<dyn Node<T>>>,
    connected_inputs: HashMap<PortIndex, Socket>,
    connected_outputs: HashMap<PortIndex, Socket>,
}

impl<T> FlowNode<T> {
    /// TODO
    pub fn node(&self) -> &Rc<RefCell<dyn Node<T>>> {
        &self.node
    }

    /// TODO
    pub fn connected_inputs(&self) -> &HashMap<PortIndex, Socket> {
        &self.connected_inputs
    }

    /// TODO
    pub fn connected_outputs(&self) -> &HashMap<PortIndex, Socket> {
        &self.connected_outputs
    }
}

/// TODO
#[derive(Debug, Default)]
pub struct Flow<T> {
    nodes: Vec<FlowNode<T>>,
}

impl<T> Flow<T> {
    /// TODO
    pub const fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    /// TODO
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
        }
    }

    /// TODO
    pub fn add_node(&mut self, node: Rc<RefCell<dyn Node<T>>>) -> NodeIndex {
        let new_node = FlowNode {
            node,
            connected_inputs: Default::default(),
            connected_outputs: Default::default(),
        };
        self.nodes.push(new_node);
        NodeIndex::new(self.nodes.len() - 1)
    }

    /// TODO
    pub fn node(&self, index: NodeIndex) -> &FlowNode<T> {
        &self.nodes[usize::from(index)]
    }

    /// TODO
    fn node_mut(&mut self, index: NodeIndex) -> &mut FlowNode<T> {
        &mut self.nodes[usize::from(index)]
    }

    /// Remove a connection from an output socket
    pub fn disconnect_output(&mut self, output: Socket) {
        let output_node = self.node_mut(output.0);
        let connected_input = output_node.connected_outputs.remove(&output.1);
        if let Some(input) = connected_input {
            output_node
                .node
                .borrow_mut()
                .output_mut(output.1)
                .activate(false);
            let input_node = self.node_mut(input.0);
            let _connected_output = input_node.connected_inputs.remove(&input.1);
            debug_assert_eq!(_connected_output, Some(output));
        }
    }

    /// Remove a connection from an input socket
    pub fn disconnect_input(&mut self, input: Socket) {
        let input_node = self.node_mut(input.0);
        let connected_output = input_node.connected_inputs.remove(&input.1);
        if let Some(output) = connected_output {
            let output_node = self.node_mut(output.0);
            let _connected_input = output_node.connected_outputs.remove(&output.1);
            debug_assert_eq!(_connected_input, Some(input));
            output_node
                .node
                .borrow_mut()
                .output_mut(output.1)
                .activate(false);
        }
    }

    /// Establish a connection between an output port and
    /// an input port of disjunct nodes
    ///
    /// TODO: Check for cycles
    pub fn connect(&mut self, output: Socket, input: Socket) {
        // Only allowed for disjunct nodes
        debug_assert_ne!(output.0, input.0);
        let output_node = self.node_mut(output.0);
        let output_port_index = output.1;
        debug_assert!(output_port_index < PortIndex::new(output_node.node.borrow().num_outputs()));
        output_node
            .connected_outputs
            .insert(output_port_index, input);
        let input_node = self.node_mut(input.0);
        let input_port_index = input.1;
        debug_assert!(input_port_index < PortIndex::new(input_node.node.borrow().num_inputs()));
        input_node.connected_inputs.insert(input_port_index, output);
    }

    /// TODO
    pub fn reconnect(&mut self, output: Socket, input: Socket) {
        self.disconnect_output(output);
        self.disconnect_input(input);
        self.connect(output, input);
    }

    /// TODO
    pub fn topological_nodes(&self) -> Vec<NodeIndex> {
        let mut candidates = Vec::with_capacity(self.nodes.len());
        let mut done = 0;
        let mut none = 0; // no predecessors
        for (index, node) in self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (NodeIndex::new(i), node))
        {
            let mut predecessors: Vec<_> = node
                .connected_inputs
                .values()
                .map(|socket| socket.0)
                .collect();
            predecessors.sort_unstable();
            predecessors.dedup();
            let no_predecessors = predecessors.is_empty();
            candidates.push((index, predecessors));
            if no_predecessors {
                none += 1;
                if none < candidates.len() {
                    let swap = candidates.swap_remove(none - 1);
                    candidates.push(swap);
                }
            }
        }
        while none < candidates.len() {
            debug_assert!(done <= none);
            let index = candidates[done].0;
            if !candidates[done].1.is_empty() {
                // cycle detected
                break;
            }
            // Remove index from remaining predecessors
            for i in none..candidates.len() {
                let candidate = &mut candidates[i];
                if let Ok(index) = candidate.1.binary_search(&index) {
                    candidate.1.swap_remove(index);
                    if candidate.1.is_empty() {
                        none += 1;
                        if none < candidates.len() {
                            candidates.swap(none - 1, i);
                        }
                    }
                }
            }
            done += 1;
        }
        candidates[0..none].iter().map(|(node, _)| *node).collect()
    }

    /// Backward pass
    pub fn propagate_output_activations_backward(&mut self, in_node: NodeIndex) {
        let in_node = self.node(in_node);
        in_node.node().borrow_mut().activate_inputs_from_outputs();
        for (in_port, out_socket) in in_node.connected_inputs() {
            let mut out_node = self.node(out_socket.0).node().borrow_mut();
            let in_node = in_node.node().borrow();
            out_node
                .output_mut(out_socket.1)
                .activate(in_node.input(*in_port).is_active());
        }
    }

    /// Forward pass
    pub fn propagate_input_values_forward(&mut self, out_node: NodeIndex) {
        let out_node = self.node(out_node);
        out_node.node().borrow_mut().update_outputs_from_inputs();
        for (out_port, in_socket) in out_node.connected_outputs() {
            let mut in_node = self.node(in_socket.0).node().borrow_mut();
            let in_port = in_node.input_mut(in_socket.1);
            if !in_port.is_active() {
                continue;
            }
            let mut out_node = out_node.node().borrow_mut();
            let out_port = out_node.output_mut(*out_port);
            let in_port = in_node.input_mut(in_socket.1);
            in_port.set_value_from(out_port);
        }
    }
}
