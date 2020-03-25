use flowcalc::{flow::*, node::*, port::*, Packet};

use std::{cell::RefCell, fmt, rc::Rc, path::{Path, PathBuf}, time::Instant};

use rand::{self, rngs::ThreadRng, Rng as _};

use resvg::Render as SvgRender;
use usvg::Tree as SvgTree;

#[derive(Clone)]
pub struct QrCode(qrcodegen::QrCode);

impl fmt::Debug for QrCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.write_str(&self.0.to_svg_string(0))
    }
}

#[derive(Debug, Clone)]
enum Value {
    Text(String),
    QrCode(QrCode),
    FilePath(PathBuf),
}

#[derive(Debug)]
struct RandomAsciiTextSource {
    num_chars: usize,
    rng: ThreadRng,
    output: Port<Value>,
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

impl Node<Value> for RandomAsciiTextSource {
    fn num_inputs(&self) -> usize {
        0
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Packet<Value> {
        unimplemented!();
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Packet<Value> {
        self.output.dispatch_packet()
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        _packet: Packet<Value>,
    ) {
        unimplemented!();
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        packet: Packet<Value>,
    ) {
        self.output.accept_packet(packet);
    }
}

impl NodeProcessor for RandomAsciiTextSource {
    fn process_inputs(&mut self, _: AccessToken) {
        if !self.output.is_active() {
            return;
        }
        let text = self.gen_text();
        self.output.slot = Some(Value::Text(text));
    }

    fn process_outputs(&mut self, _: AccessToken) {
        // nothing to do
    }
}

#[derive(Debug)]
struct TextQrEncoder {
    input: Port<Value>,
    output: Port<Value>,
}

impl TextQrEncoder {
    fn new() -> Self {
        Self {
            input: Port::new(),
            output: Port::new(),
        }
    }
}

impl Node<Value> for TextQrEncoder {
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Packet<Value> {
        self.input.dispatch_packet()
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Packet<Value> {
        self.output.dispatch_packet()
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<Value>,
    ) {
        self.input.accept_packet(packet);
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        packet: Packet<Value>,
    ) {
        self.output.accept_packet(packet);
    }
}

impl NodeProcessor for TextQrEncoder {
    fn process_inputs(&mut self, _: AccessToken) {
        if !self.output.is_active() {
            return;
        }
        let input_value = self.input.slot.take();
        if let Some(Value::Text(text)) = input_value {
            let code = qrcodegen::QrCode::encode_text(&text, qrcodegen::QrCodeEcc::Quartile)
                .expect("Input string too long");
            self.output.slot = Some(Value::QrCode(QrCode(code)));
        } else {
            panic!("Missing input string");
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.activate(self.output.is_active());
    }
}

struct QrCodeRender {
    input: Port<Value>,
    output: Port<Value>,
    render: Box<dyn SvgRender>,
}

impl QrCodeRender {
    fn new() -> Self {
        Self {
            input: Port::new(),
            output: Port::new(),
            render: resvg::default_backend(),
        }
    }
}

impl Node<Value> for QrCodeRender {
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Packet<Value> {
        self.input.dispatch_packet()
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Packet<Value> {
        self.output.dispatch_packet()
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<Value>,
    ) {
        self.input.accept_packet(packet);
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        packet: Packet<Value>,
    ) {
        self.output.accept_packet(packet);
    }
}

impl NodeProcessor for QrCodeRender {
    fn process_inputs(&mut self, _: AccessToken) {
        if !self.output.is_active() {
            return;
        }
        let input_value = self.input.slot.take();
        if let Some(Value::QrCode(code)) = input_value {
            let svg_tree =
                SvgTree::from_str(&code.0.to_svg_string(2), &Default::default()).expect("SVG tree");
            let mut output_image = self.render.render_to_image(&svg_tree, &resvg::Options {
                background: Some(usvg::Color::white()),
                fit_to: resvg::FitTo::Width(200),
                ..Default::default()
            }).expect("output image");
            // FIXME: Don't write to disk!
            let path = Path::new("/tmp/qrcode.png");
            output_image.save_png(path);
            self.output.slot = Some(Value::FilePath(path.into()));
        } else {
            panic!("Missing QR code input");
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.activate(self.output.is_active());
    }
}

struct QrTextDecoder {
    input: Port<Value>,
    output: Port<Value>,
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

impl Node<Value> for QrTextDecoder {
    fn num_inputs(&self) -> usize {
        1
    }

    fn num_outputs(&self) -> usize {
        1
    }

    fn dispatch_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
    ) -> Packet<Value> {
        self.input.dispatch_packet()
    }

    fn dispatch_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
    ) -> Packet<Value> {
        self.output.dispatch_packet()
    }

    fn accept_input_packet(
        &mut self,
        _token: AccessToken,
        _input_index: PortIndex,
        packet: Packet<Value>,
    ) {
        self.input.accept_packet(packet);
    }

    fn accept_output_packet(
        &mut self,
        _token: AccessToken,
        _output_index: PortIndex,
        packet: Packet<Value>,
    ) {
        self.output.accept_packet(packet);
    }
}

impl NodeProcessor for QrTextDecoder {
    fn process_inputs(&mut self, _: AccessToken) {
        if !self.output.is_active() {
            return;
        }
        let input_value = self.input.slot.take();
        if let Some(Value::FilePath(path)) = input_value {
            let reader = image::io::Reader::open(path).expect("PNG file");
            let image = reader.decode().expect("PNG image");
            let results = self.decoder.decode(&image);
            debug_assert!(results.len() <= 1);
            let first_result = results.into_iter().next();
            if let Some(first_result) = first_result {
                let text = first_result.expect("decoded text");
                self.output.slot = Some(Value::Text(text));
            } else {
                eprintln!("No QR codes found in image");
                self.output.slot = None;
            }
        } else {
            panic!("Missing image");
        }
    }

    fn process_outputs(&mut self, _: AccessToken) {
        self.input.activate(self.output.is_active());
    }
}

fn main() {
    let source = Rc::new(RefCell::new(RandomAsciiTextSource::new(20)));
    let encoder = Rc::new(RefCell::new(TextQrEncoder::new()));
    let decoder = Rc::new(RefCell::new(QrTextDecoder::new()));
    let render = Rc::new(RefCell::new(QrCodeRender::new()));
    let splitter = Rc::new(RefCell::new(OneToManySplitter::<Value>::new(2)));
    let printer = Rc::new(RefCell::new(DebugPrinterSink::<Value>::new(2)));

    let mut flow: Flow<RcProxyNode<_>, _> = Flow::new();
    let source_id = flow.add_node(RcProxyNode::new(Rc::clone(&source) as _));
    let encoder_id = flow.add_node(RcProxyNode::new(Rc::clone(&encoder) as _));
    let decoder_id = flow.add_node(RcProxyNode::new(Rc::clone(&decoder) as _));
    let render_id = flow.add_node(RcProxyNode::new(Rc::clone(&render) as _));
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
            node_id: render_id,
            port_index: PortIndex::new(0),
        },
    );
    flow.connect(
        Socket {
            node_id: render_id,
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
        port.activate(true);
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
