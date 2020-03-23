use flowcalc::{flow::*, node::*, port::*};

use std::{cell::RefCell, rc::Rc};

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

    fn output_state(&self, port: PortIndex) -> PortState {
        self.output(port).state()
    }

    fn set_input_state(&mut self, _: AccessToken, port: PortIndex, state: PortState) {
        self.input_mut(port).set_state(state);
    }

    fn set_output_state(&mut self, _: AccessToken, port: PortIndex, state: PortState) {
        self.output_mut(port).set_state(state);
    }

    fn put_input_value(&mut self, _: AccessToken, port: PortIndex, value: Option<f64>) {
        self.input_mut(port).put_value(value)
    }

    fn take_output_value(&mut self, _: AccessToken, port: PortIndex) -> Option<f64> {
        self.output_mut(port).take_value()
    }

    fn refresh_input_states(&mut self, _: AccessToken) {
        self.input
            .activate(self.outputs.iter().any(|p| p.state().is_active()));
    }

    fn update_output_values(&mut self, _: AccessToken) {
        let input_value = self.input.take_value();
        for output in &mut self.outputs {
            if !output.state().is_active() {
                output.reset_value();
                continue;
            }
            output.put_value(input_value);
        }
    }
}

#[derive(Default, Debug, Clone)]
struct CalculatorNode {
    inputs: [Port<f64>; 2],
    outputs: [Port<f64>; 5],
    multiplier: f64,
}

impl CalculatorNode {
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

    pub fn set_multiplier(&mut self, multiplier: f64) {
        if self.multiplier == multiplier {
            return;
        }
        self.multiplier = multiplier;
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

    fn output_state(&self, port: PortIndex) -> PortState {
        self.output(port).state()
    }

    fn set_input_state(&mut self, _: AccessToken, port: PortIndex, state: PortState) {
        self.input_mut(port).set_state(state);
    }

    fn set_output_state(&mut self, _: AccessToken, port: PortIndex, state: PortState) {
        self.output_mut(port).set_state(state);
    }

    fn put_input_value(&mut self, _: AccessToken, port: PortIndex, value: Option<f64>) {
        self.input_mut(port).put_value(value)
    }

    fn take_output_value(&mut self, _: AccessToken, port: PortIndex) -> Option<f64> {
        self.output_mut(port).take_value()
    }

    fn refresh_input_states(&mut self, _: AccessToken) {
        // Needed for all outputs except the negation of the rhs input
        self.inputs[Self::input_index_lhs()].activate(
            self.outputs
                .iter()
                .enumerate()
                .any(|(i, output)| i != Self::output_index_rhs_neg() && output.state().is_active()),
        );
        // Needed for all outputs except the negation of the lhs input
        self.inputs[Self::input_index_rhs()].activate(
            self.outputs
                .iter()
                .enumerate()
                .any(|(i, output)| i != Self::output_index_lhs_neg() && output.state().is_active()),
        );
    }

    fn update_output_values(&mut self, _: AccessToken) {
        let lhs_input_value = self.inputs[Self::input_index_lhs()].take_value();
        let rhs_input_value = self.inputs[Self::input_index_rhs()].take_value();
        for (index, output) in self.outputs.iter_mut().enumerate() {
            if !output.state().is_active() {
                output.reset_value();
                continue;
            }
            let value = match index {
                0 => {
                    debug_assert_eq!(index, Self::output_index_lhs_neg());
                    if let Some(value) = lhs_input_value {
                        Some(-value)
                    } else {
                        None
                    }
                }
                1 => {
                    debug_assert_eq!(index, Self::output_index_rhs_neg());
                    if let Some(value) = rhs_input_value {
                        Some(-value)
                    } else {
                        None
                    }
                }
                2 => {
                    debug_assert_eq!(index, Self::output_index_sum());
                    if let (Some(lhs), Some(rhs)) = (lhs_input_value, rhs_input_value) {
                        Some(lhs + rhs)
                    } else {
                        None
                    }
                }
                3 => {
                    debug_assert_eq!(index, Self::output_index_diff());
                    if let (Some(lhs), Some(rhs)) = (lhs_input_value, rhs_input_value) {
                        Some(lhs - rhs)
                    } else {
                        None
                    }
                }
                4 => {
                    debug_assert_eq!(index, Self::output_index_prod());
                    if let (Some(lhs), Some(rhs)) = (lhs_input_value, rhs_input_value) {
                        Some(lhs * rhs)
                    } else {
                        None
                    }
                }
                _ => panic!("invalid output index"),
            };
            let multiplier = self.multiplier;
            output.put_value(value.map(|value| multiplier * value));
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

    fn output_state(&self, _port: PortIndex) -> PortState {
        // No outputs, never invoked
        unimplemented!();
    }

    fn set_input_state(&mut self, _: AccessToken, port: PortIndex, state: PortState) {
        self.input_mut(port).set_state(state);
    }

    fn set_output_state(&mut self, _: AccessToken, _port: PortIndex, _state: PortState) {
        // No outputs, never invoked
        unimplemented!();
    }

    fn put_input_value(&mut self, _: AccessToken, port: PortIndex, value: Option<f64>) {
        self.input_mut(port).put_value(value)
    }

    fn take_output_value(&mut self, _: AccessToken, _port: PortIndex) -> Option<f64> {
        // No outputs, never invoked
        unimplemented!();
    }

    fn refresh_input_states(&mut self, _: AccessToken) {
        // No outputs, nothing to do
    }

    fn update_output_values(&mut self, _: AccessToken) {
        // No outputs, just a side-effect
        println!(
            "{:?}",
            self.inputs
                .iter_mut()
                .map(|input| input.take_value())
                .collect::<Vec<_>>()
        );
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
    printer.borrow_mut().inputs[CalculatorNode::output_index_sum()].activate(true);
    printer.borrow_mut().inputs[CalculatorNode::output_index_prod()].activate(true);

    let mut flow: Flow<RcProxyNode<f64>, f64> = Flow::new();
    let printer_node = flow.add_node(RcProxyNode::new(Rc::clone(&printer) as _));
    let splitter_node = flow.add_node(RcProxyNode::new(Rc::clone(&splitter) as _));
    let calculator_node = flow.add_node(RcProxyNode::new(Rc::clone(&calculator) as _));
    // Connect splitter -> calculator
    let num_splitter_outputs = splitter.borrow().num_outputs();
    for port in (0..num_splitter_outputs).map(PortIndex::new) {
        flow.connect(
            Socket {
                node: splitter_node,
                port,
            },
            Socket {
                node: calculator_node,
                port,
            },
        );
    }
    // Connect calculator -> printer
    let num_calculator_outputs = calculator.borrow().num_outputs();
    for port in (0..num_calculator_outputs).map(PortIndex::new) {
        flow.connect(
            Socket {
                node: calculator_node,
                port,
            },
            Socket {
                node: printer_node,
                port,
            },
        );
    }

    //println!("flow = {:#?}", flow);

    let topo_nodes = flow.topological_nodes().unwrap();
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
                single_input.put_value(Some(f64::from(i)));
            }
            // release mutable borrow at runtime
        }
        {
            // ...and update parameters
            calculator.borrow_mut().set_multiplier(f64::from(i) / 2.0);
            // release mutable borrow at runtime
        }

        // Forward pass
        for node in topo_nodes.iter() {
            flow.propagate_input_values_forward(*node);
        }
    }
}
