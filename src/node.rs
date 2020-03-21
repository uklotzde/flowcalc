use super::port::{Port, PortIndex};

use std::fmt;

/// TODO
pub trait Node<T>: fmt::Debug {
    /// Number of input ports
    fn num_inputs(&self) -> usize;

    /// Number of output ports
    fn num_outputs(&self) -> usize;

    /// Immutable access to an input port
    fn input(&self, port: PortIndex) -> &Port<T>;

    /// Mutable access to an input port
    fn input_mut(&mut self, port: PortIndex) -> &mut Port<T>;

    /// Immutable access to an output port
    fn output(&self, port: PortIndex) -> &Port<T>;

    /// Mutable access to an output port
    fn output_mut(&mut self, port: PortIndex) -> &mut Port<T>;

    /// Backward pass
    ///
    /// Propagate the activation flags from output ports
    /// backwards to all input ports that are affecting the
    /// corresponding output values.
    fn activate_inputs_from_outputs(&mut self);

    /// Forward pass
    ///
    /// Recalculate the values of active output ports from
    /// the current values of the corresponding input ports.
    /// The values of inactive output ports are supposed to
    /// be reset.
    fn update_outputs_from_inputs(&mut self);
}

/// TODO
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodeIndex(usize);

impl NodeIndex {
    /// TODO
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

impl From<NodeIndex> for usize {
    fn from(from: NodeIndex) -> Self {
        from.0
    }
}
