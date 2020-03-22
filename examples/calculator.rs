use flowcalc::{flow::*, node::*, port::*};

use std::{cell::RefCell, fmt, rc::Rc};

#[derive(Default, Debug, Clone)]
struct OneToManySplitterNode {
    input: Port<f64>,
    outputs: Vec<Port<f64>>,
}

impl OneToManySplitterNode {
    pub fn new(num_outputs: usize) -> Self {
        let outputs = std::iter::repeat(Default::default())
            .take(num_outputs)
            .collect();
        Self {
            outputs,
            ..Default::default()
        }
    }

    pub fn input(&self, _port: PortIndex) -> &Port<f64> {
        debug_assert!(_port < PortIndex::new(self.num_inputs()));
        &self.input
    }

    pub fn input_mut(&mut self, _port: PortIndex) -> &mut Port<f64> {
        debug_assert!(_port < PortIndex::new(self.num_inputs()));
        &mut self.input
    }

    pub fn output(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &self.outputs[usize::from(port)]
    }

    pub fn output_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &mut self.outputs[usize::from(port)]
    }
}

impl Node<f64> for OneToManySplitterNode {
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    fn input_state(&self, port: PortIndex) -> PortState {
        self.input(port).state()
    }

    fn set_output_state(&mut self, port: PortIndex, state: PortState) {
        self.output_mut(port).set_state(state);
    }

    fn set_input_value(&mut self, port: PortIndex, value: Option<f64>) {
        self.input_mut(port).set_value(value)
    }

    fn take_output_value(&mut self, port: PortIndex) -> Option<f64> {
        self.output_mut(port).take_value()
    }

    fn activate_inputs_from_outputs(&mut self) {
        self.input
            .set_state(if self.outputs.iter().any(|p| p.state().is_active()) {
                PortState::Active
            } else {
                PortState::Inactive
            });
    }

    fn update_outputs_from_inputs(&mut self) {
        for output in &mut self.outputs {
            if !output.state().is_active() {
                output.reset_value();
                continue;
            }
            output.set_value(*self.input.get_value());
        }
    }
}

#[derive(Default, Debug, Clone)]
struct CalculatorNode {
    inputs: [Port<f64>; 2],
    outputs: [Port<f64>; 5],
    multiplier: Option<f64>,
}

impl CalculatorNode {
    fn input_state(&self, port: PortIndex) -> PortState {
        self.input(port).state()
    }

    fn set_output_state(&mut self, port: PortIndex, state: PortState) {
        self.output_mut(port).set_state(state);
    }

    fn set_input_value(&mut self, port: PortIndex, value: Option<f64>) {
        self.input_mut(port).set_value(value)
    }

    fn take_output_value(&mut self, port: PortIndex) -> Option<f64> {
        self.output_mut(port).take_value()
    }

    pub const fn input_index_lhs() -> usize {
        0
    }

    pub const fn input_index_rhs() -> usize {
        1
    }

    pub const fn output_index_lhs_neg() -> usize {
        0
    }

    pub const fn output_index_rhs_neg() -> usize {
        1
    }

    pub const fn output_index_sum() -> usize {
        2
    }

    pub const fn output_index_diff() -> usize {
        3
    }

    pub const fn output_index_prod() -> usize {
        4
    }

    pub fn set_multiplier(&mut self, multiplier: Option<f64>) {
        if self.multiplier == multiplier {
            return;
        }
        self.multiplier = multiplier;
        // The multiplier affects all outputs
        for out_port in &mut self.outputs {
            out_port.reset_value()
        }
    }

    pub fn input(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &self.inputs[usize::from(port)]
    }

    pub fn input_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &mut self.inputs[usize::from(port)]
    }

    pub fn output(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &self.outputs[usize::from(port)]
    }

    pub fn output_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &mut self.outputs[usize::from(port)]
    }
}

impl Node<f64> for CalculatorNode {
    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        5
    }

    fn input_state(&self, port: PortIndex) -> PortState {
        self.input(port).state()
    }

    fn set_output_state(&mut self, port: PortIndex, state: PortState) {
        self.output_mut(port).set_state(state);
    }

    fn set_input_value(&mut self, port: PortIndex, value: Option<f64>) {
        self.input_mut(port).set_value(value)
    }

    fn take_output_value(&mut self, port: PortIndex) -> Option<f64> {
        self.output_mut(port).take_value()
    }

    fn activate_inputs_from_outputs(&mut self) {
        self.inputs[Self::input_index_lhs()].set_state(
            // Needed for all outputs except the negation of the rhs input
            if self
                .outputs
                .iter()
                .enumerate()
                .any(|(i, output)| i != Self::output_index_rhs_neg() && output.state().is_active())
            {
                PortState::Active
            } else {
                PortState::Inactive
            },
        );
        self.inputs[Self::input_index_rhs()].set_state(
            // Needed for all outputs except the negation of the lhs input
            if self
                .outputs
                .iter()
                .enumerate()
                .any(|(i, output)| i != Self::output_index_lhs_neg() && output.state().is_active())
            {
                PortState::Active
            } else {
                PortState::Inactive
            },
        );
    }

    fn update_outputs_from_inputs(&mut self) {
        for (index, output) in self.outputs.iter_mut().enumerate() {
            if !output.state().is_active() || self.multiplier.is_none() {
                output.reset_value();
                continue;
            }
            let value = match index {
                0 => {
                    debug_assert_eq!(index, Self::output_index_lhs_neg());
                    if let Some(lhs) = self.inputs[Self::input_index_lhs()].get_value() {
                        Some(-*lhs)
                    } else {
                        None
                    }
                }
                1 => {
                    debug_assert_eq!(index, Self::output_index_rhs_neg());
                    if let Some(rhs) = self.inputs[Self::input_index_rhs()].get_value() {
                        Some(-*rhs)
                    } else {
                        None
                    }
                }
                2 => {
                    debug_assert_eq!(index, Self::output_index_sum());
                    if let (Some(lhs), Some(rhs)) = (
                        self.inputs[Self::input_index_rhs()].get_value(),
                        self.inputs[Self::input_index_rhs()].get_value(),
                    ) {
                        Some(lhs + rhs)
                    } else {
                        None
                    }
                }
                3 => {
                    debug_assert_eq!(index, Self::output_index_diff());
                    if let (Some(lhs), Some(rhs)) = (
                        self.inputs[Self::input_index_rhs()].get_value(),
                        self.inputs[Self::input_index_rhs()].get_value(),
                    ) {
                        Some(lhs - rhs)
                    } else {
                        None
                    }
                }
                4 => {
                    debug_assert_eq!(index, Self::output_index_prod());
                    if let (Some(lhs), Some(rhs)) = (
                        self.inputs[Self::input_index_rhs()].get_value(),
                        self.inputs[Self::input_index_rhs()].get_value(),
                    ) {
                        Some(lhs * rhs)
                    } else {
                        None
                    }
                }
                _ => panic!("invalid output index"),
            };
            debug_assert!(self.multiplier.is_some());
            if let Some(value) = value {
                output.set_value(self.multiplier.map(|mult| mult * value));
            }
        }
    }
}

#[derive(Debug, Clone)]
struct DebugPrinterSinkNode {
    inputs: Vec<Port<f64>>,
}

impl DebugPrinterSinkNode {
    pub fn new(num_inputs: usize) -> Self {
        let inputs = std::iter::repeat(Port::new()).take(num_inputs).collect();
        Self { inputs }
    }

    pub fn input(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &self.inputs[usize::from(port)]
    }

    pub fn input_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &mut self.inputs[usize::from(port)]
    }

    pub fn output(&self, _: PortIndex) -> &Port<f64> {
        unimplemented!("no outputs available");
    }

    pub fn output_mut(&mut self, _: PortIndex) -> &mut Port<f64> {
        unimplemented!("no outputs available");
    }
}

impl Node<f64> for DebugPrinterSinkNode {
    fn num_inputs(&self) -> usize {
        self.inputs.len()
    }

    fn num_outputs(&self) -> usize {
        0
    }

    fn input_state(&self, port: PortIndex) -> PortState {
        self.input(port).state()
    }

    fn set_output_state(&mut self, _port: PortIndex, _state: PortState) {
        // No outputs, never invoked
        unimplemented!();
    }

    fn set_input_value(&mut self, port: PortIndex, value: Option<f64>) {
        self.input_mut(port).set_value(value)
    }

    fn take_output_value(&mut self, _port: PortIndex) -> Option<f64> {
        // No outputs, never invoked
        unimplemented!();
    }

    fn activate_inputs_from_outputs(&mut self) {
        // No outputs, nothing to do
    }

    fn update_outputs_from_inputs(&mut self) {
        // No outputs, just a side-effect
        println!(
            "{:?}",
            self.inputs
                .iter()
                .map(|input| input.get_value())
                .collect::<Vec<_>>()
        );
    }
}

#[derive(Debug, Clone)]
struct NodeProxy<T> {
    node: Rc<RefCell<dyn Node<T>>>,
}

impl<T> NodeProxy<T> {
    pub fn new(node: Rc<RefCell<dyn Node<T>>>) -> Self {
        Self { node }
    }
}

impl<T> Node<T> for NodeProxy<T>
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

    fn set_output_state(&mut self, port: PortIndex, state: PortState) {
        self.node.borrow_mut().set_output_state(port, state)
    }

    fn set_input_value(&mut self, port: PortIndex, value: Option<T>) {
        self.node.borrow_mut().set_input_value(port, value)
    }

    fn take_output_value(&mut self, port: PortIndex) -> Option<T> {
        self.node.borrow_mut().take_output_value(port)
    }

    fn activate_inputs_from_outputs(&mut self) {
        self.node.borrow_mut().activate_inputs_from_outputs();
    }

    fn update_outputs_from_inputs(&mut self) {
        self.node.borrow_mut().update_outputs_from_inputs();
    }
}

fn main() {
    let calculator = Rc::new(RefCell::new(CalculatorNode::default()));
    let splitter = Rc::new(RefCell::new(OneToManySplitterNode::new(
        calculator.borrow().num_inputs(),
    )));
    let printer = Rc::new(RefCell::new(DebugPrinterSinkNode::new(
        calculator.borrow().num_outputs(),
    )));
    // Print only selected outputs from the calculator
    printer.borrow_mut().inputs[CalculatorNode::output_index_sum()].set_state(PortState::Active);
    printer.borrow_mut().inputs[CalculatorNode::output_index_prod()].set_state(PortState::Active);

    let mut flow: Flow<NodeProxy<f64>, f64> = Flow::new();
    let printer_node = flow.add_node(NodeProxy::new(Rc::clone(&printer) as _));
    let splitter_node = flow.add_node(NodeProxy::new(Rc::clone(&splitter) as _));
    let calculator_node = flow.add_node(NodeProxy::new(Rc::clone(&calculator) as _));
    // Connect splitter -> calculator
    for port in (0..splitter.borrow().num_outputs()).map(PortIndex::new) {
        flow.connect(Socket(splitter_node, port), Socket(calculator_node, port));
    }
    // Connect calculator -> printer
    for port in (0..calculator.borrow().num_outputs()).map(PortIndex::new) {
        flow.connect(Socket(calculator_node, port), Socket(printer_node, port));
    }

    //println!("flow = {:#?}", flow);

    let topo_nodes = flow.topological_nodes();
    //println!("topo_nodes = {:?}", topo_nodes);

    for i in 0..10 {
        // Backward pass
        for node in topo_nodes.iter().rev() {
            flow.propagate_output_states_backward(*node);
        }

        // Inject input values...
        {
            let single_input = &mut splitter.borrow_mut().input;
            if single_input.state().is_active() {
                single_input.set_value(Some(f64::from(i)));
            }
            // release mutable borrow at runtime
        }
        // ...and update parameters
        calculator
            .borrow_mut()
            .set_multiplier(Some(f64::from(i) / 2.0));

        // Forward pass
        for node in topo_nodes.iter() {
            flow.propagate_input_values_forward(*node);
        }
    }
}
