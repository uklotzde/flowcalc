use flowcalc::{flow::*, node::*, port::*};

use std::{cell::RefCell, rc::Rc, time::Instant};

use rand::{self, rngs::ThreadRng, Rng as _};

#[derive(Debug, Clone)]
enum Value {
    Text(String),
    GrayImage(image::GrayImage),
}

#[derive(Debug)]
struct RandomAsciiTextSource {
    num_chars: usize,
    rng: ThreadRng,
    output: Port<(), Value>,
}

impl RandomAsciiTextSource {
    fn new(num_chars: usize) -> Self {
        Self {
            num_chars,
            rng: rand::thread_rng(),
            output: Port::new(),
        }
    }

    fn gen_text(&mut self) -> String {
        let num_chars = self.num_chars;
        std::iter::repeat(())
            .map(|()| self.rng.sample(rand::distributions::Alphanumeric))
            .take(num_chars)
            .collect()
    }
}

impl Node<(), Value> for RandomAsciiTextSource {}

impl NodeInputs<(), Value> for RandomAsciiTextSource {
    fn num_inputs(&self) -> usize {
        0
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        _packet: Packet<Value, ()>,
    ) {
        unimplemented!();
    }

    fn try_dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Option<Packet<(), Value>> {
        unimplemented!();
    }
}

impl NodeOutputs<(), Value> for RandomAsciiTextSource {
    fn num_outputs(&self) -> usize {
        1
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        packet: Packet<(), Value>,
    ) {
        self.output.accept_packet(packet);
    }

    fn try_dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Option<Packet<Value, ()>> {
        self.output.try_dispatch_packet()
    }
}

impl NodeProcessor for RandomAsciiTextSource {
    fn process_inputs(&mut self, _: AccessToken) {
        if self.output.incoming.is_none() {
            return;
        }
        let text = self.gen_text();
        self.output.outgoing = Some(Value::Text(text));
    }

    fn process_outputs(&mut self, _: AccessToken) {
        // nothing to do
    }
}

#[derive(Debug)]
struct TextQrEncoder {
    input: Port<Value, ()>,
    output: Port<(), Value>,
}

impl TextQrEncoder {
    fn new() -> Self {
        Self {
            input: Port::new(),
            output: Port::new(),
        }
    }
}

impl Node<(), Value> for TextQrEncoder {}

impl NodeInputs<(), Value> for TextQrEncoder {
    fn num_inputs(&self) -> usize {
        1
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<Value, ()>,
    ) {
        self.input.accept_packet(packet);
    }

    fn try_dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Option<Packet<(), Value>> {
        self.input.try_dispatch_packet()
    }
}

impl NodeOutputs<(), Value> for TextQrEncoder {
    fn num_outputs(&self) -> usize {
        1
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        packet: Packet<(), Value>,
    ) {
        self.output.accept_packet(packet);
    }

    fn try_dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Option<Packet<Value, ()>> {
        self.output.try_dispatch_packet()
    }
}

impl NodeProcessor for TextQrEncoder {
    fn process_inputs(&mut self, _: AccessToken) {
        if self.output.incoming.is_none() {
            return;
        }
        let input_value = self.input.incoming.take();
        if let Some(Value::Text(text)) = input_value {
            let code = qrcode::QrCode::new(text.as_bytes()).expect("QR code");
            let image = code.render::<image::Luma<_>>().build();
            self.output.outgoing = Some(Value::GrayImage(image));
        } else {
            panic!("Missing input string");
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.outgoing = self.output.incoming;
    }
}

struct QrTextDecoder {
    input: Port<Value, ()>,
    output: Port<(), Value>,
    decoder: bardecoder::Decoder<image::DynamicImage, image::GrayImage>,
}

impl QrTextDecoder {
    fn new() -> Self {
        Self {
            input: Port::new(),
            output: Port::new(),
            decoder: bardecoder::default_decoder(),
        }
    }
}

impl Node<(), Value> for QrTextDecoder {}

impl NodeInputs<(), Value> for QrTextDecoder {
    fn num_inputs(&self) -> usize {
        1
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<Value, ()>,
    ) {
        self.input.accept_packet(packet);
    }

    fn try_dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Option<Packet<(), Value>> {
        self.input.try_dispatch_packet()
    }
}

impl NodeOutputs<(), Value> for QrTextDecoder {
    fn num_outputs(&self) -> usize {
        1
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        packet: Packet<(), Value>,
    ) {
        self.output.accept_packet(packet);
    }

    fn try_dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Option<Packet<Value, ()>> {
        self.output.try_dispatch_packet()
    }
}

impl NodeProcessor for QrTextDecoder {
    fn process_inputs(&mut self, _: AccessToken) {
        if self.output.incoming.is_none() {
            // Not output requested
            return;
        }
        let input_value = self.input.incoming.take();
        if let Some(Value::GrayImage(image)) = input_value {
            let results = self.decoder.decode(&image::DynamicImage::ImageLuma8(image));
            debug_assert!(results.len() <= 1);
            let first_result = results.into_iter().next();
            if let Some(first_result) = first_result {
                let text = first_result.expect("decoded text");
                self.output.outgoing = Some(Value::Text(text));
            } else {
                eprintln!("No QR codes found in image");
                self.output.outgoing = None;
            }
        } else {
            panic!("Missing image");
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.outgoing = self.output.incoming;
    }
}

fn main() {
    let source = Rc::new(RefCell::new(RandomAsciiTextSource::new(20)));
    let encoder = Rc::new(RefCell::new(TextQrEncoder::new()));
    let decoder = Rc::new(RefCell::new(QrTextDecoder::new()));
    let splitter = Rc::new(RefCell::new(OneToManySplitter::<(), Value>::new(2)));
    let printer = Rc::new(RefCell::new(DebugPrinterSink::<(), Value>::new(2)));

    let mut flow: Flow<RcProxyNode<_, _>, _, _> = Flow::new();
    let source_id = flow.add_node(RcProxyNode::new(Rc::clone(&source) as _));
    let encoder_id = flow.add_node(RcProxyNode::new(Rc::clone(&encoder) as _));
    let decoder_id = flow.add_node(RcProxyNode::new(Rc::clone(&decoder) as _));
    let splitter_id = flow.add_node(RcProxyNode::new(Rc::clone(&splitter) as _));
    let printer_id = flow.add_node(RcProxyNode::new(Rc::clone(&printer) as _));
    flow.connect(
        Socket {
            node_id: source_id,
            port_index: PortIndex::new(0),
        },
        Socket {
            node_id: splitter_id,
            port_index: PortIndex::new(0),
        },
    );
    flow.connect(
        Socket {
            node_id: splitter_id,
            port_index: PortIndex::new(0),
        },
        Socket {
            node_id: encoder_id,
            port_index: PortIndex::new(0),
        },
    );
    flow.connect(
        Socket {
            node_id: splitter_id,
            port_index: PortIndex::new(1),
        },
        Socket {
            node_id: printer_id,
            port_index: PortIndex::new(0),
        },
    );
    flow.connect(
        Socket {
            node_id: encoder_id,
            port_index: PortIndex::new(0),
        },
        Socket {
            node_id: decoder_id,
            port_index: PortIndex::new(0),
        },
    );
    flow.connect(
        Socket {
            node_id: decoder_id,
            port_index: PortIndex::new(0),
        },
        Socket {
            node_id: printer_id,
            port_index: PortIndex::new(1),
        },
    );

    // Activate all sink inputs
    for port in printer.borrow_mut().inputs.ports_mut() {
        port.outgoing = Some(());
    }

    let topo_nodes = flow.topological_nodes().unwrap();

    for _ in 0..10 {
        let now = Instant::now();

        // Backward pass
        for node in topo_nodes.iter().rev() {
            flow.process_outputs(*node);
        }

        // Forward pass
        for node in topo_nodes.iter() {
            flow.process_inputs(*node);
        }

        println!("Duration: {} ms", now.elapsed().as_micros() as f64 / 1000.0);
    }
}
