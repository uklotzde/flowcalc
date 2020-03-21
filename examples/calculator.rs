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
}

impl Node<f64> for OneToManySplitterNode {
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        self.outputs.len()
    }

    fn input(&self, _port: PortIndex) -> &Port<f64> {
        debug_assert!(_port < PortIndex::new(self.num_inputs()));
        &self.input
    }

    fn input_mut(&mut self, _port: PortIndex) -> &mut Port<f64> {
        debug_assert!(_port < PortIndex::new(self.num_inputs()));
        &mut self.input
    }

    fn output(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &self.outputs[usize::from(port)]
    }

    fn output_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &mut self.outputs[usize::from(port)]
    }

    fn activate_inputs_from_outputs(&mut self) {
        self.input
            .activate(self.outputs.iter().any(|p| p.is_active()));
    }

    fn update_outputs_from_inputs(&mut self) {
        for output_port in &mut self.outputs {
            let _ = output_port.take_value();
            if !output_port.is_active() {
                continue;
            }
            output_port.set_value(*self.input.get_value());
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
}

impl Node<f64> for CalculatorNode {
    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        5
    }

    fn input(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &self.inputs[usize::from(port)]
    }

    fn input_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &mut self.inputs[usize::from(port)]
    }

    fn output(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &self.outputs[usize::from(port)]
    }

    fn output_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_outputs()));
        &mut self.outputs[usize::from(port)]
    }

    fn activate_inputs_from_outputs(&mut self) {
        self.inputs[Self::input_index_lhs()].activate(
            // Needed for all outputs except the negation of the rhs input
            self.outputs
                .iter()
                .enumerate()
                .any(|(i, p)| i != Self::output_index_rhs_neg() && p.is_active()),
        );
        self.inputs[Self::input_index_rhs()].activate(
            // Needed for all outputs except the negation of the lhs input
            self.outputs
                .iter()
                .enumerate()
                .any(|(i, p)| i != Self::output_index_lhs_neg() && p.is_active()),
        );
    }

    fn update_outputs_from_inputs(&mut self) {
        for (index, port) in self.outputs.iter_mut().enumerate() {
            let _ = port.take_value();
            if !port.is_active() || self.multiplier.is_none() {
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
                port.set_value(self.multiplier.map(|mult| mult * value));
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
}

impl Node<f64> for DebugPrinterSinkNode {
    fn num_inputs(&self) -> usize {
        self.inputs.len()
    }

    fn num_outputs(&self) -> usize {
        0
    }

    fn input(&self, port: PortIndex) -> &Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &self.inputs[usize::from(port)]
    }

    fn input_mut(&mut self, port: PortIndex) -> &mut Port<f64> {
        debug_assert!(port < PortIndex::new(self.num_inputs()));
        &mut self.inputs[usize::from(port)]
    }

    fn output(&self, _: PortIndex) -> &Port<f64> {
        unimplemented!("no outputs available");
    }

    fn output_mut(&mut self, _: PortIndex) -> &mut Port<f64> {
        unimplemented!("no outputs available");
    }

    fn activate_inputs_from_outputs(&mut self) {
        // No outputs
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

    let mut flow = Flow::new();
    let printer_node = flow.add_node(Rc::clone(&printer) as _);
    let splitter_node = flow.add_node(Rc::clone(&splitter) as _);
    let calculator_node = flow.add_node(Rc::clone(&calculator) as _);
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
            flow.propagate_output_activations_backward(*node);
        }

        // Inject input values...
        {
            let single_input = &mut splitter.borrow_mut().input;
            if single_input.is_active() {
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
