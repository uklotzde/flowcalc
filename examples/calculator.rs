use flowcalc::{flow::*, node::*, port::*, Packet};

use std::{cell::RefCell, rc::Rc};

#[derive(Default, Debug, Clone)]
struct CalculatorNode {
    inputs: [Port<f64>; 2],
    outputs: [Port<f64>; 5],
    multiplier: f64,
}

impl CalculatorNode {
    pub const fn input_index_lhs() -> PortIndex {
        PortIndex::new(0)
    }

    pub const fn input_index_rhs() -> PortIndex {
        PortIndex::new(1)
    }

    pub const fn output_index_lhs_neg() -> PortIndex {
        PortIndex::new(0)
    }

    pub const fn output_index_rhs_neg() -> PortIndex {
        PortIndex::new(1)
    }

    pub const fn output_index_sum() -> PortIndex {
        PortIndex::new(2)
    }

    pub const fn output_index_diff() -> PortIndex {
        PortIndex::new(3)
    }

    pub const fn output_index_prod() -> PortIndex {
        PortIndex::new(4)
    }

    pub fn set_multiplier(&mut self, multiplier: f64) {
        if self.multiplier == multiplier {
            return;
        }
        self.multiplier = multiplier;
    }

    pub fn input(&self, input_index: PortIndex) -> &Port<f64> {
        debug_assert!(input_index < PortIndex::new(self.num_inputs()));
        &self.inputs[usize::from(input_index)]
    }

    pub fn input_mut(&mut self, input_index: PortIndex) -> &mut Port<f64> {
        debug_assert!(input_index < PortIndex::new(self.num_inputs()));
        &mut self.inputs[usize::from(input_index)]
    }

    pub fn output(&self, output_index: PortIndex) -> &Port<f64> {
        debug_assert!(output_index < PortIndex::new(self.num_outputs()));
        &self.outputs[usize::from(output_index)]
    }

    pub fn output_mut(&mut self, output_index: PortIndex) -> &mut Port<f64> {
        debug_assert!(output_index < PortIndex::new(self.num_outputs()));
        &mut self.outputs[usize::from(output_index)]
    }
}

impl Node<f64> for CalculatorNode {
    fn num_inputs(&self) -> usize {
        2
    }

    fn num_outputs(&self) -> usize {
        5
    }

    fn dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
    ) -> Packet<f64> {
        self.input_mut(input_index).dispatch_packet()
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
    ) -> Packet<f64> {
        self.output_mut(output_index).dispatch_packet()
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        input_index: PortIndex,
        packet: Packet<f64>,
    ) {
        self.input_mut(input_index).accept_packet(packet);
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        output_index: PortIndex,
        packet: Packet<f64>,
    ) {
        self.output_mut(output_index).accept_packet(packet);
    }
}

impl NodeProcessor for CalculatorNode {
    fn process_inputs(&mut self, _: AccessToken) {
        let lhs_input_value = self.input_mut(Self::input_index_lhs()).slot.take();
        let rhs_input_value = self.input_mut(Self::input_index_rhs()).slot.take();
        for (index, output) in self.outputs.iter_mut().enumerate() {
            if !output.is_active() {
                continue;
            }
            let value = match index {
                0 => {
                    debug_assert_eq!(index, Self::output_index_lhs_neg().into());
                    if let Some(value) = lhs_input_value {
                        Some(-value)
                    } else {
                        None
                    }
                }
                1 => {
                    debug_assert_eq!(index, Self::output_index_rhs_neg().into());
                    if let Some(value) = rhs_input_value {
                        Some(-value)
                    } else {
                        None
                    }
                }
                2 => {
                    debug_assert_eq!(index, Self::output_index_sum().into());
                    if let (Some(lhs), Some(rhs)) = (lhs_input_value, rhs_input_value) {
                        Some(lhs + rhs)
                    } else {
                        None
                    }
                }
                3 => {
                    debug_assert_eq!(index, Self::output_index_diff().into());
                    if let (Some(lhs), Some(rhs)) = (lhs_input_value, rhs_input_value) {
                        Some(lhs - rhs)
                    } else {
                        None
                    }
                }
                4 => {
                    debug_assert_eq!(index, Self::output_index_prod().into());
                    if let (Some(lhs), Some(rhs)) = (lhs_input_value, rhs_input_value) {
                        Some(lhs * rhs)
                    } else {
                        None
                    }
                }
                _ => panic!("invalid output index"),
            };
            let multiplier = self.multiplier;
            output.slot = value.map(|value| multiplier * value);
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        // Needed for all outputs except the negation of the rhs input
        let lhs_active = self
            .outputs
            .iter()
            .enumerate()
            .any(|(i, output)| i != Self::output_index_rhs_neg().into() && output.is_active());
        self.input_mut(Self::input_index_lhs()).activate(lhs_active);
        // Needed for all outputs except the negation of the lhs input
        let rhs_active = self
            .outputs
            .iter()
            .enumerate()
            .any(|(i, output)| i != Self::output_index_lhs_neg().into() && output.is_active());
        self.input_mut(Self::input_index_rhs()).activate(rhs_active);
    }
}

fn main() {
    let calculator = Rc::new(RefCell::new(CalculatorNode::default()));
    let splitter = Rc::new(RefCell::new(OneToManySplitterNode::<f64>::new(
        calculator.borrow().num_inputs(),
    )));
    let printer = Rc::new(RefCell::new(DebugPrinterSinkNode::<f64>::new(
        calculator.borrow().num_outputs(),
    )));
    // Print only selected outputs from the calculator
    printer
        .borrow_mut()
        .input_mut(CalculatorNode::output_index_sum())
        .activate(true);
    printer
        .borrow_mut()
        .input_mut(CalculatorNode::output_index_prod())
        .activate(true);

    let mut flow: Flow<RcProxyNode<f64>, f64> = Flow::new();
    let printer_id = flow.add_node(RcProxyNode::new(Rc::clone(&printer) as _));
    let splitter_id = flow.add_node(RcProxyNode::new(Rc::clone(&splitter) as _));
    let calculator_id = flow.add_node(RcProxyNode::new(Rc::clone(&calculator) as _));
    // Connect splitter -> calculator
    let num_splitter_outputs = splitter.borrow().num_outputs();
    for port_index in (0..num_splitter_outputs).map(PortIndex::new) {
        flow.connect(
            Socket {
                node_id: splitter_id,
                port_index,
            },
            Socket {
                node_id: calculator_id,
                port_index,
            },
        );
    }
    // Connect calculator -> printer
    let num_calculator_outputs = calculator.borrow().num_outputs();
    for port_index in (0..num_calculator_outputs).map(PortIndex::new) {
        flow.connect(
            Socket {
                node_id: calculator_id,
                port_index,
            },
            Socket {
                node_id: printer_id,
                port_index,
            },
        );
    }

    //println!("flow = {:#?}", flow);

    let topo_nodes = flow.topological_nodes().unwrap();
    //println!("topo_nodes = {:?}", topo_nodes);

    for i in 0..10 {
        // Backward pass
        for node in topo_nodes.iter().rev() {
            flow.process_outputs(*node);
        }

        // Inject input values...
        {
            let mut splitter_node = splitter.borrow_mut();
            let single_input = splitter_node.input_mut();
            if single_input.is_active() {
                single_input.slot = Some(f64::from(i));
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
            flow.process_inputs(*node);
        }
    }
}
