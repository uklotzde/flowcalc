use crate::{node::*, port::*, SealedTag};

use std::{collections::HashMap, marker::PhantomData};

/// Private access token of `Flow` to access sealed
/// methods in `Node`.
///
/// This is a workaround for restricting the visibility of
/// methods in traits to the crate that defines the trait.
#[derive(Debug)]
pub struct AccessToken {
    tag: SealedTag,
}

impl AccessToken {
    fn new() -> Self {
        Self { tag: SealedTag }
    }
}

/// Node identifier in a flow graph
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodeId(usize);

impl NodeId {
    /// TODO
    fn new(index: usize) -> Self {
        Self(index)
    }
}

impl From<NodeId> for usize {
    fn from(from: NodeId) -> Self {
        from.0
    }
}

/// Address of an input or output at a node
///
/// The type of port (input or output) is unspecified
/// and implicitly follows from the context.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Socket {
    /// The node identifier
    pub node: NodeId,

    /// The port index
    pub port: PortIndex,
}

#[derive(Debug)]
struct FlowNode<N> {
    node: N,
    connected_inputs: HashMap<PortIndex, Socket>,
    connected_outputs: HashMap<PortIndex, Socket>,
}

/// Directed acyclic graph (DAG) of computational nodes
#[derive(Debug, Default)]
pub struct Flow<N, T> {
    nodes: Vec<FlowNode<N>>,
    phantom: PhantomData<T>,
}

/// Detected cycle
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Cycle {
    node: NodeId,
}

impl Cycle {
    /// A node within the cycle
    pub fn node(self) -> NodeId {
        self.node
    }
}

impl<N, T> Flow<N, T>
where
    N: Node<T>,
{
    /// TODO
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            phantom: PhantomData,
        }
    }

    /// TODO
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            phantom: PhantomData,
        }
    }

    /// TODO
    pub fn add_node(&mut self, node: N) -> NodeId {
        let new_node = FlowNode {
            node,
            connected_inputs: Default::default(),
            connected_outputs: Default::default(),
        };
        self.nodes.push(new_node);
        NodeId::new(self.nodes.len() - 1)
    }

    /// TODO
    pub fn node(&self, index: NodeId) -> &N {
        &self.flow_node(index).node
    }

    /// TODO
    pub fn node_mut(&mut self, index: NodeId) -> &mut N {
        &mut self.flow_node_mut(index).node
    }

    fn flow_node(&self, index: NodeId) -> &FlowNode<N> {
        &self.nodes[usize::from(index)]
    }

    fn flow_node_mut(&mut self, index: NodeId) -> &mut FlowNode<N> {
        &mut self.nodes[usize::from(index)]
    }

    /// Remove a connection from an output socket
    ///
    /// Returns the input socket of the subsequent node
    /// at the opposite end of the connection.
    ///
    /// State or values of disconnected ports on both
    /// ends of a removed connection are not modified.
    pub fn disconnect_output(&mut self, output: Socket) -> Option<Socket> {
        let Socket { node, port } = output;
        let node = self.flow_node_mut(node);
        let connected_input = node.connected_outputs.remove(&port);
        node.node.set_output_state(
            AccessToken::new(),
            port,
            PortState {
                connectivity: ConnectivityState::Disconnected,
                ..node.node.output_state(port)
            },
        );
        if let Some(input) = connected_input {
            let Socket { node, port } = input;
            let node = self.flow_node_mut(node);
            let _connected_output = node.connected_inputs.remove(&port);
            debug_assert_eq!(_connected_output, Some(output));
            node.node.set_input_state(
                AccessToken::new(),
                port,
                PortState {
                    connectivity: ConnectivityState::Disconnected,
                    ..node.node.input_state(port)
                },
            );
        }
        connected_input
    }

    /// Remove a connection from an input socket
    ///
    /// Returns the output socket of the preceding node
    /// at the opposite end of the connection.
    ///
    /// State or values of disconnected ports on both
    /// ends of a removed connection are not modified.
    pub fn disconnect_input(&mut self, input: Socket) -> Option<Socket> {
        let Socket { node, port } = input;
        let node = self.flow_node_mut(node);
        let connected_output = node.connected_inputs.remove(&port);
        let input_state = PortState {
            connectivity: ConnectivityState::Disconnected,
            ..node.node.input_state(port)
        };
        node.node
            .set_input_state(AccessToken::new(), port, input_state);
        if let Some(output) = connected_output {
            let Socket { node, port } = output;
            let node = self.flow_node_mut(node);
            let _connected_input = node.connected_outputs.remove(&port);
            debug_assert_eq!(_connected_input, Some(input));
            let output_state = PortState {
                connectivity: ConnectivityState::Disconnected,
                ..node.node.output_state(port)
            };
            node.node
                .set_output_state(AccessToken::new(), port, output_state);
        }
        connected_output
    }

    /// Establish a connection between an output port of a
    /// preceding node and an input port of a subsequent node
    ///
    /// Connections are only permitted between distinct nodes,
    /// i.e. neither reflexive connections nor cycles are allowed.
    ///
    /// The caller is responsible to ensure that no cycles are
    /// introduced by the new connection! Otherwise a debug
    /// assertion is triggered.
    pub fn connect(&mut self, output: Socket, input: Socket) {
        // Check for reflexive connections upfront
        debug_assert_ne!(output.node, input.node);
        // Connect output port
        let output_node = self.flow_node_mut(output.node);
        let output_port = output.port;
        output_node.connected_outputs.insert(output_port, input);
        let output_state = PortState {
            connectivity: ConnectivityState::Disconnected,
            ..output_node.node.output_state(output_port)
        };
        output_node
            .node
            .set_output_state(AccessToken::new(), output_port, output_state);
        // Connect input port
        let input_node = self.flow_node_mut(input.node);
        let input_port = input.port;
        input_node.connected_inputs.insert(input_port, output);
        let input_state = PortState {
            connectivity: ConnectivityState::Disconnected,
            ..input_node.node.input_state(input_port)
        };
        input_node
            .node
            .set_input_state(AccessToken::new(), input_port, input_state);
        // Check for no cycles
        debug_assert!(self.topological_nodes().is_ok());
    }

    /// TODO
    pub fn reconnect(&mut self, output: Socket, input: Socket) {
        self.disconnect_output(output);
        self.disconnect_input(input);
        self.connect(output, input);
    }

    /// Precompute a topological ordering of all nodes
    /// in the flow graph.
    ///
    /// The returned array of node indexes can be used to
    /// traverse the nodes in the graph either forward or
    /// backward in reverse order.
    pub fn topological_nodes(&self) -> Result<Vec<NodeId>, Cycle> {
        let mut candidates = Vec::with_capacity(self.nodes.len());
        let mut done = 0;
        let mut none = 0; // no predecessors
        for (index, node) in self
            .nodes
            .iter()
            .enumerate()
            .map(|(i, node)| (NodeId::new(i), node))
        {
            let mut predecessors: Vec<_> = node
                .connected_inputs
                .values()
                .map(|socket| socket.node)
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
                return Err(Cycle { node: index });
            }
            // Remove index from remaining predecessors
            #[allow(clippy::mut_range_bound)]
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
        Ok(candidates[0..none].iter().map(|(node, _)| *node).collect())
    }

    /// Execute backward pass for a single node
    ///
    /// Propagate the output states of a select node to its
    /// inputs and then along the input connections to all
    /// outputs of preceding nodes.
    pub fn propagate_output_states_backward(&mut self, in_node: NodeId) {
        // 1st immutable borrow
        let in_node_ptr = {
            let in_node = self.flow_node_mut(in_node);
            in_node.node.refresh_input_states(AccessToken::new());
            in_node as *const FlowNode<N>
        };
        // The 2nd mutable borrow is safe, because both nodes
        // are guaranteed to be disjunct and the flow graph
        // itself is not modified.
        #[allow(unused_unsafe)]
        unsafe {
            for (in_port, out_socket) in &(*in_node_ptr).connected_inputs {
                let in_node = &(*in_node_ptr).node;
                let state = in_node.input_state(*in_port);
                // 2nd mutable borrow
                let out_node = &mut self.flow_node_mut(out_socket.node).node as *mut N;
                (*out_node).set_output_state(AccessToken::new(), out_socket.port, state);
            }
        }
    }

    /// Execute forward pass for a single node
    ///
    /// Update the output values of a selected node and then
    /// pass those values along the output connections to all
    /// inputs of subsequent nodes.
    pub fn propagate_input_values_forward(&mut self, out_node: NodeId) {
        // 1st mutable borrow
        let out_node_ptr = {
            let out_node = self.flow_node_mut(out_node);
            out_node.node.update_output_values(AccessToken::new());
            out_node as *mut FlowNode<N>
        };
        // The 2nd mutable borrow is safe, because both nodes
        // are guaranteed to be disjunct and the flow graph
        // itself is not modified.
        #[allow(unused_unsafe)]
        unsafe {
            for (out_port, in_socket) in &(*out_node_ptr).connected_outputs {
                // 2nd mutable borrow
                let out_node = &mut (*out_node_ptr).node;
                let value = out_node.take_output_value(AccessToken::new(), *out_port);
                let in_node = &mut self.flow_node_mut(in_socket.node).node;
                in_node.put_input_value(AccessToken::new(), in_socket.port, value);
            }
        }
    }
}
