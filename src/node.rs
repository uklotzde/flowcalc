use super::port::{PortIndex, PortState};

use std::fmt;

/// TODO
pub trait Node<T>: fmt::Debug {
    /// Number of input ports
    fn num_inputs(&self) -> usize;

    /// Number of output ports
    fn num_outputs(&self) -> usize;

    /// Query input state
    fn input_state(&self, port: PortIndex) -> PortState;

    /// Query output state
    fn output_state(&self, port: PortIndex) -> PortState;

    /// Modify output state
    fn set_input_state(&mut self, port: PortIndex, state: PortState);

    /// Modify output state
    fn set_output_state(&mut self, port: PortIndex, state: PortState);

    /// Modify input value
    fn set_input_value(&mut self, port: PortIndex, value: Option<T>);

    /// Consume output value
    fn take_output_value(&mut self, port: PortIndex) -> Option<T>;

    /// Backward pass
    ///
    /// Propagate port states from outputs backwards to all inputs
    /// that are affecting the corresponding output values.
    fn activate_inputs_from_outputs(&mut self);

    /// Forward pass
    ///
    /// Recalculate the values of active outputs from the current
    /// values at the input ports. Values of inactive outputs are
    /// supposed to be reset to prevent reading stale values.
    fn update_outputs_from_inputs(&mut self);
}
