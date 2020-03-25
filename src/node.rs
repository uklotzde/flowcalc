use super::{
    flow::AccessToken,
    port::{Port, PortIndex},
    Packet,
};

use std::{cell::RefCell, fmt, rc::Rc};

/// TODO
pub trait SemiNode<T>: fmt::Debug {
    /// Query the number of ports
    fn num_ports(&self) -> usize;

    /// TODO
    fn receive_packet(&mut self, token: AccessToken, port_index: PortIndex, packet: Packet<T>);

    /// TODO
    fn dispatch_packet(&mut self, token: AccessToken, port_index: PortIndex) -> Packet<T>;

    /// TODO
    fn process_packets(&mut self, token: AccessToken);
}

/// TODO
pub trait FullNode<T>: fmt::Debug {
    /// TODO
    type Input: SemiNode<T>;

    /// TODO
    type Output: SemiNode<T>;

    /// TODO
    fn input(&self) -> &Self::Input;

    /// TODO
    fn input_mut(&self) -> &mut Self::Input;

    /// TODO
    fn output(&self) -> &Self::Input;

    /// TODO
    fn output_mut(&self) -> &mut Self::Input;
}

/// TODO
pub trait Processor: fmt::Debug {
    /// Backward pass: Refresh the state of all inputs
    ///
    /// Propagate port states from outputs backwards to all inputs
    /// that are affecting the corresponding output values.
    ///
    /// This decision is made independent of whether the node needs
    /// to be updated or not. It must only take into account the pure
    /// functional dependencies between inputs and outputs.
    fn process_outputs(&mut self, token: AccessToken);

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
    fn process_inputs(&mut self, token: AccessToken);
}

/// TODO
pub trait Node<T>: Processor + fmt::Debug {
    /// Query the number of input ports
    fn num_inputs(&self) -> usize;

    /// Query the number of output ports
    fn num_outputs(&self) -> usize;

    /// TODO
    fn receive_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Packet<T>,
    );

    /// TODO
    fn receive_output_packet(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
        packet: Packet<T>,
    );

    /// TODO
    fn dispatch_input_packet(&mut self, token: AccessToken, input_index: PortIndex) -> Packet<T>;

    /// TODO
    fn dispatch_output_packet(&mut self, token: AccessToken, output_index: PortIndex) -> Packet<T>;
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

    fn receive_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.node
            .borrow_mut()
            .receive_input_packet(token, input_index, packet)
    }

    fn receive_output_packet(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.node
            .borrow_mut()
            .receive_output_packet(token, output_index, packet)
    }

    fn dispatch_input_packet(&mut self, token: AccessToken, input_index: PortIndex) -> Packet<T> {
        self.node
            .borrow_mut()
            .dispatch_input_packet(token, input_index)
    }

    fn dispatch_output_packet(&mut self, token: AccessToken, output_index: PortIndex) -> Packet<T> {
        self.node
            .borrow_mut()
            .dispatch_output_packet(token, output_index)
    }
}

impl<T> Processor for RcProxyNode<T>
where
    T: fmt::Debug,
{
    fn process_outputs(&mut self, token: AccessToken) {
        self.node.borrow_mut().process_outputs(token);
    }

    fn process_inputs(&mut self, token: AccessToken) {
        self.node.borrow_mut().process_inputs(token);
    }
}

/// TODO
#[derive(Default, Debug, Clone)]
pub struct OneToManySplitterNode<T> {
    input: Port<T>,
    outputs: Vec<Port<T>>,
}

impl<T> OneToManySplitterNode<T> {
    /// TODO
    pub fn new(num_outputs: usize) -> Self {
        let mut outputs = Vec::with_capacity(num_outputs);
        for _ in 0..num_outputs {
            outputs.push(Port::new());
        }
        Self {
            input: Port::new(),
            outputs,
        }
    }

    /// TODO
    pub fn input(&self) -> &Port<T> {
        &self.input
    }

    /// TODO
    pub fn input_mut(&mut self) -> &mut Port<T> {
        &mut self.input
    }

    /// TODO
    pub fn output(&self, output_index: PortIndex) -> &Port<T> {
        debug_assert!(output_index < PortIndex::new(self.outputs.len()));
        &self.outputs[usize::from(output_index)]
    }

    /// TODO
    pub fn output_mut(&mut self, output_index: PortIndex) -> &mut Port<T> {
        debug_assert!(output_index < PortIndex::new(self.outputs.len()));
        &mut self.outputs[usize::from(output_index)]
    }
}

impl<T> Node<T> for OneToManySplitterNode<T>
where
    T: Clone + fmt::Debug,
{
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    fn dispatch_input_packet(&mut self, _token: AccessToken, _input_index: PortIndex) -> Packet<T> {
        debug_assert_eq!(PortIndex::new(0), _input_index);
        self.input.dispatch_packet()
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
    ) -> Packet<T> {
        self.output_mut(output_index).dispatch_packet()
    }

    fn receive_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<T>,
    ) {
        debug_assert_eq!(PortIndex::new(0), _input_index);
        self.input.receive_packet(packet);
    }

    fn receive_output_packet(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.output_mut(output_index).receive_packet(packet);
    }
}

impl<T> Processor for OneToManySplitterNode<T>
where
    T: Clone + fmt::Debug,
{
    fn process_inputs(&mut self, _token: AccessToken) {
        debug_assert!(self.input.is_active());
        let input_value = self.input.slot.take();
        for output in &mut self.outputs {
            if !output.is_active() {
                continue;
            }
            output.slot = input_value.clone();
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input
            .activate(self.outputs.iter().any(|output| output.is_active()));
    }
}

#[derive(Debug, Default, Clone)]
/// TODO
pub struct DebugPrinterSinkNode<T> {
    inputs: Vec<Port<T>>,
}

impl<T> DebugPrinterSinkNode<T> {
    /// TODO
    pub fn new(num_inputs: usize) -> Self {
        let mut inputs = Vec::with_capacity(num_inputs);
        for _ in 0..num_inputs {
            inputs.push(Port::new());
        }
        Self { inputs }
    }

    /// TODO
    pub fn input(&self, input_index: PortIndex) -> &Port<T> {
        &self.inputs[usize::from(input_index)]
    }

    /// TODO
    pub fn input_mut(&mut self, input_index: PortIndex) -> &mut Port<T> {
        &mut self.inputs[usize::from(input_index)]
    }
}

impl<T> Node<T> for DebugPrinterSinkNode<T>
where
    T: fmt::Debug,
{
    fn num_inputs(&self) -> usize {
        self.inputs.len()
    }

    fn num_outputs(&self) -> usize {
        0
    }

    fn dispatch_input_packet(&mut self, _token: AccessToken, input_index: PortIndex) -> Packet<T> {
        self.input_mut(input_index).dispatch_packet()
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Packet<T> {
        unimplemented!();
    }

    fn receive_input_packet(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.input_mut(input_index).receive_packet(packet);
    }

    fn receive_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        _packet: Packet<T>,
    ) {
        unimplemented!();
    }
}

impl<T> Processor for DebugPrinterSinkNode<T>
where
    T: fmt::Debug,
{
    fn process_inputs(&mut self, _: AccessToken) {
        // No outputs, just a side-effect
        println!(
            "{:?}",
            self.inputs
                .iter_mut()
                .map(|input| input.slot.take())
                .collect::<Vec<_>>()
        );
    }

    fn process_outputs(&mut self, _: AccessToken) {
        // No outputs, nothing to do
    }
}
