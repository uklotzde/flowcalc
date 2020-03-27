use super::{
    flow::AccessToken,
    port::{Packet, Port, PortBay, PortIndex, VecPortBay},
};

use std::{cell::RefCell, fmt, rc::Rc};

pub trait NodeProcessor {
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

pub trait NodeInputs<C, D> {
    /// Query the number of input ports
    fn num_inputs(&self) -> usize;

    fn accept_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Packet<D, C>,
    );

    fn try_dispatch_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
    ) -> Option<Packet<C, D>>;
}

pub trait NodeOutputs<C, D> {
    /// Query the number of output ports
    fn num_outputs(&self) -> usize;

    fn accept_output_packet(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
        packet: Packet<C, D>,
    );

    fn try_dispatch_output_packet(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
    ) -> Option<Packet<D, C>>;
}

pub trait Node<C, D>: NodeInputs<C, D> + NodeOutputs<C, D> + NodeProcessor {}

/// A reference-counted node proxy
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct RcProxyNode<C, D> {
    node: Rc<RefCell<dyn Node<C, D>>>,
}

impl<C, D> RcProxyNode<C, D> {
    /// Create a new proxy node by wrapping a shared node
    pub fn new(node: Rc<RefCell<dyn Node<C, D>>>) -> Self {
        Self { node }
    }
}

impl<C, D> Node<C, D> for RcProxyNode<C, D> {}

impl<C, D> NodeInputs<C, D> for RcProxyNode<C, D> {
    fn num_inputs(&self) -> usize {
        self.node.borrow().num_inputs()
    }

    fn accept_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Packet<D, C>,
    ) {
        self.node
            .borrow_mut()
            .accept_input_packet(token, input_index, packet)
    }

    fn try_dispatch_input_packet(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
    ) -> Option<Packet<C, D>> {
        self.node
            .borrow_mut()
            .try_dispatch_input_packet(token, input_index)
    }
}

impl<C, D> NodeOutputs<C, D> for RcProxyNode<C, D> {
    fn num_outputs(&self) -> usize {
        self.node.borrow().num_outputs()
    }

    fn accept_output_packet(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
        packet: Packet<C, D>,
    ) {
        self.node
            .borrow_mut()
            .accept_output_packet(token, output_index, packet)
    }

    fn try_dispatch_output_packet(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
    ) -> Option<Packet<D, C>> {
        self.node
            .borrow_mut()
            .try_dispatch_output_packet(token, output_index)
    }
}

impl<C, D> NodeProcessor for RcProxyNode<C, D> {
    fn process_outputs(&mut self, token: AccessToken) {
        self.node.borrow_mut().process_outputs(token);
    }

    fn process_inputs(&mut self, token: AccessToken) {
        self.node.borrow_mut().process_inputs(token);
    }
}

pub trait JoinablePortControl {
    fn join_next_port_control(self, next_ctrl: Option<&Self>) -> Self;
}

impl JoinablePortControl for () {
    fn join_next_port_control(self, _next_ctrl: Option<&Self>) -> Self {
        self
    }
}

impl JoinablePortControl for bool {
    fn join_next_port_control(self, next_ctrl: Option<&Self>) -> Self {
        if let Some(next_ctrl) = next_ctrl {
            self || *next_ctrl
        } else {
            self
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct OneToManySplitter<C, D> {
    input: Port<D, C>,
    outputs: VecPortBay<C, D>,
}

impl<C, D> OneToManySplitter<C, D> {
    pub fn new(num_outputs: usize) -> Self {
        Self {
            input: Port::new(),
            outputs: VecPortBay::new(num_outputs),
        }
    }

    pub fn input(&self) -> &Port<D, C> {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut Port<D, C> {
        &mut self.input
    }

    pub fn output(&self, output_index: PortIndex) -> &Port<C, D> {
        self.outputs.port(output_index)
    }

    pub fn output_mut(&mut self, output_index: PortIndex) -> &mut Port<C, D> {
        self.outputs.port_mut(output_index)
    }
}

impl<C, D> Node<C, D> for OneToManySplitter<C, D>
where
    C: Clone + JoinablePortControl,
    D: Clone,
{
}

impl<C, D> NodeInputs<C, D> for OneToManySplitter<C, D>
where
    C: Clone + JoinablePortControl,
    D: Clone,
{
    fn num_inputs(&self) -> usize {
        1
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<D, C>,
    ) {
        debug_assert_eq!(PortIndex::new(0), _input_index);
        self.input.accept_packet(packet);
    }

    fn try_dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Option<Packet<C, D>> {
        debug_assert_eq!(PortIndex::new(0), _input_index);
        self.input.try_dispatch_packet()
    }
}

impl<C, D> NodeOutputs<C, D> for OneToManySplitter<C, D>
where
    C: Clone + JoinablePortControl,
    D: Clone,
{
    fn num_outputs(&self) -> usize {
        self.outputs.num_ports()
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
        packet: Packet<C, D>,
    ) {
        self.outputs.accept_packet(output_index, packet)
    }

    fn try_dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
    ) -> Option<Packet<D, C>> {
        self.outputs.try_dispatch_packet(output_index)
    }
}

impl<C, D> NodeProcessor for OneToManySplitter<C, D>
where
    C: Clone + JoinablePortControl,
    D: Clone,
{
    fn process_inputs(&mut self, _token: AccessToken) {
        for output_port in self.outputs.ports_mut() {
            if output_port.incoming.is_none() {
                continue;
            }
            output_port.outgoing = self
                .input
                .incoming
                .as_ref()
                .map(|incoming| incoming.clone());
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.outgoing = self.outputs.ports().fold(None, |ctrl, port| {
            if let Some(ctrl) = ctrl {
                Some(ctrl.join_next_port_control(port.incoming.as_ref()))
            } else {
                port.incoming.as_ref().map(Clone::clone)
            }
        })
    }
}

#[derive(Debug, Default, Clone)]

pub struct DebugPrinterSink<C, D> {
    pub inputs: VecPortBay<D, C>,
}

impl<C, D> DebugPrinterSink<C, D> {
    pub fn new(num_inputs: usize) -> Self {
        Self {
            inputs: VecPortBay::new(num_inputs),
        }
    }

    pub fn input(&self, input_index: PortIndex) -> &Port<D, C> {
        self.inputs.port(input_index)
    }

    pub fn input_mut(&mut self, input_index: PortIndex) -> &mut Port<D, C> {
        self.inputs.port_mut(input_index)
    }
}

impl<C, D> Node<C, D> for DebugPrinterSink<C, D> where D: fmt::Debug {}

impl<C, D> NodeInputs<C, D> for DebugPrinterSink<C, D>
where
    D: fmt::Debug,
{
    fn num_inputs(&self) -> usize {
        self.inputs.num_ports()
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
        packet: Packet<D, C>,
    ) {
        self.inputs.accept_packet(input_index, packet);
    }

    fn try_dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
    ) -> Option<Packet<C, D>> {
        self.inputs.try_dispatch_packet(input_index)
    }
}

impl<C, D> NodeOutputs<C, D> for DebugPrinterSink<C, D> {
    fn num_outputs(&self) -> usize {
        0
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        _packet: Packet<C, D>,
    ) {
        unimplemented!();
    }

    fn try_dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Option<Packet<D, C>> {
        unimplemented!();
    }
}

impl<C, D> NodeProcessor for DebugPrinterSink<C, D>
where
    D: fmt::Debug,
{
    fn process_inputs(&mut self, _: AccessToken) {
        // No outputs, just a side-effect
        println!(
            "{:?}",
            self.inputs
                .ports_mut()
                .map(|port| port.incoming.as_ref())
                .collect::<Vec<_>>()
        );
    }

    fn process_outputs(&mut self, _: AccessToken) {
        // No outputs, nothing to do
    }
}
