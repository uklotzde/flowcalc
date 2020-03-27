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
    pub node_id: NodeId,

    /// The port index
    pub port_index: PortIndex,
}

#[derive(Debug)]
struct FlowNode<N> {
    node: N,
    connected_inputs: HashMap<PortIndex, Socket>,
    connected_outputs: HashMap<PortIndex, Socket>,
}

/// Directed acyclic graph (DAG) of computational nodes
#[derive(Debug, Default)]
pub struct Flow<N, S, P> {
    nodes: Vec<FlowNode<N>>,
    phantom1: PhantomData<S>,
    phantom2: PhantomData<P>,
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

impl<N, S, P> Flow<N, S, P>
where
    N: Node<S, P>,
{
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            phantom1: PhantomData,
            phantom2: PhantomData,
        }
    }

    pub fn add_node(&mut self, node: N) -> NodeId {
        let new_node = FlowNode {
            node,
            connected_inputs: Default::default(),
            connected_outputs: Default::default(),
        };
        self.nodes.push(new_node);
        NodeId::new(self.nodes.len() - 1)
    }

    pub fn node(&self, node_id: NodeId) -> &N {
        &self.flow_node(node_id).node
    }

    pub fn node_mut(&mut self, node_id: NodeId) -> &mut N {
        &mut self.flow_node_mut(node_id).node
    }

    fn flow_node(&self, node_id: NodeId) -> &FlowNode<N> {
        &self.nodes[usize::from(node_id)]
    }

    fn flow_node_mut(&mut self, node_id: NodeId) -> &mut FlowNode<N> {
        &mut self.nodes[usize::from(node_id)]
    }

    /// Remove a connection from an output socket
    ///
    /// Returns the input socket of the subsequent node
    /// at the opposite end of the connection.
    ///
    /// State or values of disconnected ports on both
    /// ends of a removed connection are not modified.
    pub fn disconnect_output(&mut self, output: Socket) -> Option<Socket> {
        let Socket {
            node_id,
            port_index,
        } = output;
        let node = self.flow_node_mut(node_id);
        let connected_input = node.connected_outputs.remove(&port_index);
        if let Some(input) = connected_input {
            let Socket {
                node_id,
                port_index,
            } = input;
            let node = self.flow_node_mut(node_id);
            let _connected_output = node.connected_inputs.remove(&port_index);
            debug_assert_eq!(_connected_output, Some(output));
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
        let Socket {
            node_id,
            port_index,
        } = input;
        let node = self.flow_node_mut(node_id);
        let connected_output = node.connected_inputs.remove(&port_index);
        if let Some(output) = connected_output {
            let Socket {
                node_id,
                port_index,
            } = output;
            let node = self.flow_node_mut(node_id);
            let _connected_input = node.connected_outputs.remove(&port_index);
            debug_assert_eq!(_connected_input, Some(input));
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
        debug_assert_ne!(output.node_id, input.node_id);
        // Connect output port
        let output_node = self.flow_node_mut(output.node_id);
        let output_index = output.port_index;
        output_node.connected_outputs.insert(output_index, input);
        // Connect input port
        let input_node = self.flow_node_mut(input.node_id);
        let input_index = input.port_index;
        input_node.connected_inputs.insert(input_index, output);
        // Check for no cycles
        debug_assert!(self.topological_nodes().is_ok());
    }

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
                .map(|socket| socket.node_id)
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
    pub fn process_outputs(&mut self, node_id: NodeId) {
        let flow_node_ptr = {
            let flow_node = self.flow_node_mut(node_id);
            flow_node.node.process_outputs(AccessToken::new());
            flow_node as *mut FlowNode<N>
        };
        // The 2nd mutable borrow is safe, because both nodes
        // are guaranteed to be disjunct and the flow graph
        // itself is not modified.
        #[allow(unused_unsafe)]
        unsafe {
            let node = &mut (*flow_node_ptr).node;
            for (input_index, incoming) in &(*flow_node_ptr).connected_inputs {
                let packet = node.try_dispatch_input_packet(AccessToken::new(), *input_index);
                if let Some(packet) = packet {
                    let Socket {
                        node_id: predecessor_node_id,
                        port_index: predecessor_port_index,
                    } = *incoming;
                    debug_assert_ne!(node_id, predecessor_node_id); // disjunct nodes!
                    let predecessor_node = &mut self.flow_node_mut(predecessor_node_id).node;
                    predecessor_node.accept_output_packet(
                        AccessToken::new(),
                        predecessor_port_index,
                        packet,
                    );
                }
            }
        }
    }

    /// Execute forward pass for a single node
    ///
    /// Update the output values of a selected node and then
    /// pass those values along the output connections to all
    /// inputs of subsequent nodes.
    pub fn process_inputs(&mut self, node_id: NodeId) {
        let flow_node_ptr = {
            let flow_node_ptr = self.flow_node_mut(node_id);
            flow_node_ptr.node.process_inputs(AccessToken::new());
            flow_node_ptr as *mut FlowNode<N>
        };
        // The 2nd mutable borrow is safe, because both nodes
        // are guaranteed to be disjunct and the flow graph
        // itself is not modified.
        #[allow(unused_unsafe)]
        unsafe {
            let node = &mut (*flow_node_ptr).node;
            for (output_index, outgoing) in &(*flow_node_ptr).connected_outputs {
                let packet = node.try_dispatch_output_packet(AccessToken::new(), *output_index);
                if let Some(packet) = packet {
                    let Socket {
                        node_id: successor_node_id,
                        port_index: successor_port_index,
                    } = *outgoing;
                    debug_assert_ne!(node_id, successor_node_id); // disjunct nodes!
                    let successor_node = &mut self.flow_node_mut(successor_node_id).node;
                    successor_node.accept_input_packet(
                        AccessToken::new(),
                        successor_port_index,
                        packet,
                    );
                }
            }
        }
    }
}
