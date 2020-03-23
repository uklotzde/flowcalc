use super::port::{PortIndex, PortState};

use std::{cell::RefCell, fmt, rc::Rc};

/// TODO
pub trait Node<T>: fmt::Debug {
    /// Query the number of input ports
    fn num_inputs(&self) -> usize;

    /// Query the number of output ports
    fn num_outputs(&self) -> usize;

    /// Query the state of an input port
    fn input_state(&self, port: PortIndex) -> PortState;

    /// Query the state of an output port
    fn output_state(&self, port: PortIndex) -> PortState;

    /// Modify the state of an input port
    fn set_input_state(&mut self, port: PortIndex, state: PortState);

    /// Modify the state of an output port
    fn set_output_state(&mut self, port: PortIndex, state: PortState);

    /// Place an input value into the port
    ///
    /// The previous values (if any) is overwritten.
    fn put_input_value(&mut self, port: PortIndex, value: Option<T>);

    /// Consume an output value from the port
    ///
    /// Only a single invocation is permitted after the outputs
    /// have been updated. Results of subsequent invocations are
    /// undefined and supposed to return `None`.
    fn take_output_value(&mut self, port: PortIndex) -> Option<T>;

    /// Backward pass: Refresh the state of all inputs
    ///
    /// Propagate port states from outputs backwards to all inputs
    /// that are affecting the corresponding output values.
    ///
    /// This decision is made independent of whether the node needs
    /// to be updated or not. It must only take into account the pure
    /// functional dependencies between inputs and outputs.
    fn refresh_input_states(&mut self);

    /// Forward pass: Update the values of all outputs
    ///
    /// Consume active input values and recalculate the value of
    /// the corresponding active outputs. Values of inactive outputs
    /// are supposed to be reset to `None` to prevent reading stale
    /// values.
    ///
    /// It is the responsibility of each node to determine and decide
    /// if and how the output values need to be recalculated, thereby
    /// taking into account both the input values and its internal
    /// state at the time of invocation.
    ///
    /// Nodes are encouraged to reset input values to `None` as
    /// a result of consuming them. The current input values could
    /// still be cached internally for subsequent operations, e.g.
    /// to determine if input values have changed between invocations.
    fn update_output_values(&mut self);
}

/// A reference-counted node proxy
#[derive(Debug, Clone)]
pub struct RcProxyNode<T> {
    node: Rc<RefCell<dyn Node<T>>>,
}

impl<T> RcProxyNode<T> {
    /// Create a new proxy node by wrapping a shared node
    pub fn new(node: Rc<RefCell<dyn Node<T>>>) -> Self {
        Self { node }
    }
}

impl<T> Node<T> for RcProxyNode<T>
where
    T: fmt::Debug,
{
    fn num_inputs(&self) -> usize {
        self.node.borrow().num_inputs()
    }

    fn num_outputs(&self) -> usize {
        self.node.borrow().num_outputs()
    }

    fn input_state(&self, port: PortIndex) -> PortState {
        self.node.borrow().input_state(port)
    }

    fn output_state(&self, port: PortIndex) -> PortState {
        self.node.borrow().output_state(port)
    }

    fn set_input_state(&mut self, port: PortIndex, state: PortState) {
        self.node.borrow_mut().set_input_state(port, state)
    }

    fn set_output_state(&mut self, port: PortIndex, state: PortState) {
        self.node.borrow_mut().set_output_state(port, state)
    }

    fn put_input_value(&mut self, port: PortIndex, value: Option<T>) {
        self.node.borrow_mut().put_input_value(port, value)
    }

    fn take_output_value(&mut self, port: PortIndex) -> Option<T> {
        self.node.borrow_mut().take_output_value(port)
    }

    fn refresh_input_states(&mut self) {
        self.node.borrow_mut().refresh_input_states();
    }

    fn update_output_values(&mut self) {
        self.node.borrow_mut().update_output_values();
    }
}
