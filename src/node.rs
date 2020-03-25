use super::{
    flow::AccessToken,
    port::{Port, PortBay, PortIndex, VecPortBay},
    Packet,
};

use std::{cell::RefCell, fmt, rc::Rc};

/// TODO
pub trait Processor: fmt::Debug {
    /// Process the current contents from all ports
    fn process(&mut self, token: AccessToken);
}

/// TODO
pub trait SemiNode<T>: Processor + PortBay<T> + fmt::Debug {}

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
pub trait NodeProcessor: fmt::Debug {
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
pub trait Node<T>: NodeProcessor + fmt::Debug {
    /// Query the number of input ports
    fn num_inputs(&self) -> usize;

    /// Query the number of output ports
    fn num_outputs(&self) -> usize;

    /// TODO
    fn accept_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Packet<T>,
    );

    /// TODO
    fn accept_output_packet(
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

    fn accept_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.node
            .borrow_mut()
            .accept_input_packet(token, input_index, packet)
    }

    fn accept_output_packet(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.node
            .borrow_mut()
            .accept_output_packet(token, output_index, packet)
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

impl<T> NodeProcessor for RcProxyNode<T>
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
    outputs: VecPortBay<T>,
}

impl<T> OneToManySplitterNode<T> {
    /// TODO
    pub fn new(num_outputs: usize) -> Self {
        Self {
            input: Port::new(),
            outputs: VecPortBay::new(num_outputs),
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
        self.outputs.port(output_index)
    }

    /// TODO
    pub fn output_mut(&mut self, output_index: PortIndex) -> &mut Port<T> {
        self.outputs.port_mut(output_index)
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
        self.outputs.num_ports()
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<T>,
    ) {
        debug_assert_eq!(PortIndex::new(0), _input_index);
        self.input.accept_packet(packet);
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.outputs.accept_packet(output_index, packet)
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
        self.outputs.dispatch_packet(output_index)
    }
}

impl<T> NodeProcessor for OneToManySplitterNode<T>
where
    T: Clone + fmt::Debug,
{
    fn process_inputs(&mut self, _token: AccessToken) {
        debug_assert!(self.input.is_active());
        let input_value = self.input.slot.take();
        for output_port in self.outputs.ports_mut() {
            if !output_port.is_active() {
                continue;
            }
            output_port.slot = input_value.clone();
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.activate(
            self.outputs
                .ports()
                .any(|output_port| output_port.is_active()),
        );
    }
}

#[derive(Debug, Default, Clone)]
/// TODO
pub struct DebugPrinterSinkNode<T> {
    inputs: VecPortBay<T>,
}

impl<T> DebugPrinterSinkNode<T> {
    /// TODO
    pub fn new(num_inputs: usize) -> Self {
        Self {
            inputs: VecPortBay::new(num_inputs),
        }
    }

    /// TODO
    pub fn input(&self, input_index: PortIndex) -> &Port<T> {
        self.inputs.port(input_index)
    }

    /// TODO
    pub fn input_mut(&mut self, input_index: PortIndex) -> &mut Port<T> {
        self.inputs.port_mut(input_index)
    }
}

impl<T> Node<T> for DebugPrinterSinkNode<T>
where
    T: fmt::Debug,
{
    fn num_inputs(&self) -> usize {
        self.inputs.num_ports()
    }

    fn num_outputs(&self) -> usize {
        0
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
        packet: Packet<T>,
    ) {
        self.inputs.accept_packet(input_index, packet);
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        _packet: Packet<T>,
    ) {
        unimplemented!();
    }

    fn dispatch_input_packet(&mut self, _token: AccessToken, input_index: PortIndex) -> Packet<T> {
        self.inputs.dispatch_packet(input_index)
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Packet<T> {
        unimplemented!();
    }
}

impl<T> NodeProcessor for DebugPrinterSinkNode<T>
where
    T: fmt::Debug,
{
    fn process_inputs(&mut self, _: AccessToken) {
        // No outputs, just a side-effect
        println!(
            "{:?}",
            self.inputs
                .ports_mut()
                .map(|port| port.slot.take())
                .collect::<Vec<_>>()
        );
    }

    fn process_outputs(&mut self, _: AccessToken) {
        // No outputs, nothing to do
    }
}
