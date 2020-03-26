use super::{
    flow::AccessToken,
    port::{Ctrlgram, Datagram, Port, PortBay, PortIndex, PortStatus, VecPortBay},
};

use std::{cell::RefCell, fmt, rc::Rc};

pub trait Processor {
    /// Process the current contents from all ports
    fn process(&mut self, token: AccessToken);
}

pub trait SemiNode<C, D>: Processor + PortBay<C, D> {}

pub trait FullNode<C, D> {
    type Input: SemiNode<C, D>;

    type Output: SemiNode<C, D>;

    fn input(&self) -> &Self::Input;

    fn input_mut(&self) -> &mut Self::Input;

    fn output(&self) -> &Self::Input;

    fn output_mut(&self) -> &mut Self::Input;
}

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

pub trait Node<C, D>: NodeProcessor {
    /// Query the number of input ports
    fn num_inputs(&self) -> usize;

    /// Query the number of output ports
    fn num_outputs(&self) -> usize;

    fn accept_input_datagram(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Datagram<C, D>,
    );

    fn accept_output_ctrlgram(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
        packet: Ctrlgram<C, D>,
    );

    fn dispatch_input_ctrlgram(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
    ) -> Option<Ctrlgram<C, D>>;

    fn dispatch_output_datagram(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
    ) -> Option<Datagram<C, D>>;
}

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

impl<C, D> Node<C, D> for RcProxyNode<C, D> {
    fn num_inputs(&self) -> usize {
        self.node.borrow().num_inputs()
    }

    fn num_outputs(&self) -> usize {
        self.node.borrow().num_outputs()
    }

    fn accept_input_datagram(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
        packet: Datagram<C, D>,
    ) {
        self.node
            .borrow_mut()
            .accept_input_datagram(token, input_index, packet)
    }

    fn accept_output_ctrlgram(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
        packet: Ctrlgram<C, D>,
    ) {
        self.node
            .borrow_mut()
            .accept_output_ctrlgram(token, output_index, packet)
    }

    fn dispatch_input_ctrlgram(
        &mut self,
        token: AccessToken,
        input_index: PortIndex,
    ) -> Option<Ctrlgram<C, D>> {
        self.node
            .borrow_mut()
            .dispatch_input_ctrlgram(token, input_index)
    }

    fn dispatch_output_datagram(
        &mut self,
        token: AccessToken,
        output_index: PortIndex,
    ) -> Option<Datagram<C, D>> {
        self.node
            .borrow_mut()
            .dispatch_output_datagram(token, output_index)
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
    input: Port<C, D>,
    outputs: VecPortBay<C, D>,
}

impl<C, D> OneToManySplitter<C, D> {
    pub fn new(num_outputs: usize) -> Self {
        Self {
            input: Port::new(),
            outputs: VecPortBay::new(num_outputs),
        }
    }

    pub fn input(&self) -> &Port<C, D> {
        &self.input
    }

    pub fn input_mut(&mut self) -> &mut Port<C, D> {
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
    C: PortStatus + JoinablePortControl + Clone,
    D: Clone,
{
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        self.outputs.num_ports()
    }

    fn accept_output_ctrlgram(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
        packet: Ctrlgram<C, D>,
    ) {
        self.outputs.accept_ctrlgram(output_index, packet)
    }

    fn accept_input_datagram(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Datagram<C, D>,
    ) {
        debug_assert_eq!(PortIndex::new(0), _input_index);
        self.input.accept_datagram(packet);
    }

    fn dispatch_input_ctrlgram(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Option<Ctrlgram<C, D>> {
        debug_assert_eq!(PortIndex::new(0), _input_index);
        self.input.dispatch_ctrlgram()
    }

    fn dispatch_output_datagram(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
    ) -> Option<Datagram<C, D>> {
        self.outputs.dispatch_datagram(output_index)
    }
}

impl<C, D> NodeProcessor for OneToManySplitter<C, D>
where
    C: PortStatus + JoinablePortControl + Clone,
    D: Clone,
{
    fn process_inputs(&mut self, _token: AccessToken) {
        debug_assert!(self.input.is_active());
        for output_port in self.outputs.ports_mut() {
            if !output_port.is_active() {
                continue;
            }
            output_port.data = self.input.data.as_ref().map(|data| data.clone());
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.ctrl = self.outputs.ports().fold(None, |ctrl, port| {
            if let Some(ctrl) = ctrl {
                Some(ctrl.join_next_port_control(port.ctrl.as_ref()))
            } else {
                port.ctrl.as_ref().map(Clone::clone)
            }
        })
    }
}

#[derive(Debug, Default, Clone)]

pub struct DebugPrinterSink<C, D> {
    pub inputs: VecPortBay<C, D>,
}

impl<C, D> DebugPrinterSink<C, D> {
    pub fn new(num_inputs: usize) -> Self {
        Self {
            inputs: VecPortBay::new(num_inputs),
        }
    }

    pub fn input(&self, input_index: PortIndex) -> &Port<C, D> {
        self.inputs.port(input_index)
    }

    pub fn input_mut(&mut self, input_index: PortIndex) -> &mut Port<C, D> {
        self.inputs.port_mut(input_index)
    }
}

impl<C, D> Node<C, D> for DebugPrinterSink<C, D>
where
    C: PortStatus,
    D: fmt::Debug,
{
    fn num_inputs(&self) -> usize {
        self.inputs.num_ports()
    }

    fn num_outputs(&self) -> usize {
        0
    }

    fn accept_input_datagram(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
        packet: Datagram<C, D>,
    ) {
        self.inputs.accept_datagram(input_index, packet);
    }

    fn accept_output_ctrlgram(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        _packet: Ctrlgram<C, D>,
    ) {
        unimplemented!();
    }

    fn dispatch_input_ctrlgram(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
    ) -> Option<Ctrlgram<C, D>> {
        self.inputs.dispatch_ctrlgram(input_index)
    }

    fn dispatch_output_datagram(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Option<Datagram<C, D>> {
        unimplemented!();
    }
}

impl<C, D> NodeProcessor for DebugPrinterSink<C, D>
where
    C: PortStatus,
    D: fmt::Debug,
{
    fn process_inputs(&mut self, _: AccessToken) {
        // No outputs, just a side-effect
        println!(
            "{:?}",
            self.inputs
                .ports_mut()
                .map(|port| port.data.as_ref())
                .collect::<Vec<_>>()
        );
    }

    fn process_outputs(&mut self, _: AccessToken) {
        // No outputs, nothing to do
    }
}
