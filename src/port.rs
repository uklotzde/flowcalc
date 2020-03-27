/// An input or output port of a processing node.
///
/// Ports have two generic parameters for packet data:
///   - `C`: Control data
///   - `D`: Payload data
///
/// Request control data `C` is passed backward in the flow
/// graph from input to connected output ports.
///
/// Payload response data `D` is passed forward in the flow
/// graph from output to connected input ports.
#[derive(Default, Debug, Clone, Copy)]
pub struct Port<C, D> {
    /// A slot (= a buffer with capacity 1) for a control message
    pub ctrl: Option<C>,

    /// A slot (= a buffer with capacity 1) for a data message
    pub data: Option<D>,
}

impl<C, D> Port<C, D> {
    /// Create an empty port
    pub const fn new() -> Self {
        Self {
            ctrl: None,
            data: None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Ctrlgram<C, D> {
    /// A control message
    pub ctrl: C,

    /// Optional data buffer to be recycled
    pub data: Option<D>,
}

#[derive(Debug, Clone, Copy)]
pub struct Datagram<C, D> {
    /// A data message
    pub data: D,

    /// Optional control buffer to be recycled
    pub ctrl: Option<C>,
}

impl<C, D> Port<C, D> {
    /// Accept a ctrl packet from a connected input port
    pub fn accept_ctrlgram(&mut self, packet: Ctrlgram<C, D>) {
        debug_assert!(self.ctrl.is_none());
        debug_assert!(self.data.is_none());
        self.ctrl = Some(packet.ctrl);
        self.data = packet.data;
    }

    /// Accept a data packet from a connected output port
    pub fn accept_datagram(&mut self, packet: Datagram<C, D>) {
        debug_assert!(self.ctrl.is_none());
        debug_assert!(self.data.is_none());
        self.ctrl = packet.ctrl;
        self.data = Some(packet.data);
    }

    /// Dispatch a ctrl packet for a connected input port
    pub fn dispatch_ctrlgram(&mut self) -> Option<Ctrlgram<C, D>> {
        if let Some(ctrl) = self.ctrl.take() {
            Some(Ctrlgram {
                ctrl,
                data: self.data.take(),
            })
        } else {
            None
        }
    }

    /// Dispatch a data packet for a connected output port
    pub fn dispatch_datagram(&mut self) -> Option<Datagram<C, D>> {
        if let Some(data) = self.data.take() {
            Some(Datagram {
                ctrl: self.ctrl.take(),
                data,
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortIndex(usize);

impl PortIndex {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
}

impl From<PortIndex> for usize {
    fn from(from: PortIndex) -> Self {
        from.0
    }
}

/// An indexed collection of ports
pub trait PortBay<C, D> {
    /// The number of ports in this bay
    ///
    /// Port indexes are 0-based in the range 0..num_ports().
    fn num_ports(&self) -> usize;

    /// Fetch and dispatch a ctrl packet from the given port
    fn dispatch_ctrlgram(&mut self, port_index: PortIndex) -> Option<Ctrlgram<C, D>>;

    /// Fetch and dispatch a data packet from the given port
    fn dispatch_datagram(&mut self, port_index: PortIndex) -> Option<Datagram<C, D>>;

    /// Receive an incoming ctrl packet for the given port
    fn accept_ctrlgram(&mut self, port_index: PortIndex, packet: Ctrlgram<C, D>);

    /// Receive an incoming data packet for the given port
    fn accept_datagram(&mut self, port_index: PortIndex, packet: Datagram<C, D>);
}

#[derive(Default, Debug, Clone)]
pub struct VecPortBay<C, D> {
    ports: Vec<Port<C, D>>,
}

impl<C, D> VecPortBay<C, D> {
    pub fn new(num_ports: usize) -> Self {
        let mut ports = Vec::with_capacity(num_ports);
        for _ in 0..num_ports {
            ports.push(Port::new());
        }
        Self { ports }
    }

    pub fn ports(&self) -> impl Iterator<Item = &Port<C, D>> {
        self.ports.iter()
    }

    pub fn ports_mut(&mut self) -> impl Iterator<Item = &mut Port<C, D>> {
        self.ports.iter_mut()
    }

    pub fn port(&self, port_index: PortIndex) -> &Port<C, D> {
        let index = usize::from(port_index);
        debug_assert!(index < self.ports.len());
        &self.ports[index]
    }

    pub fn port_mut(&mut self, port_index: PortIndex) -> &mut Port<C, D> {
        let index = usize::from(port_index);
        debug_assert!(index < self.ports.len());
        &mut self.ports[index]
    }
}

impl<C, D> PortBay<C, D> for VecPortBay<C, D> {
    fn num_ports(&self) -> usize {
        self.ports.len()
    }

    fn dispatch_ctrlgram(&mut self, port_index: PortIndex) -> Option<Ctrlgram<C, D>> {
        self.port_mut(port_index).dispatch_ctrlgram()
    }

    fn dispatch_datagram(&mut self, port_index: PortIndex) -> Option<Datagram<C, D>> {
        self.port_mut(port_index).dispatch_datagram()
    }

    fn accept_ctrlgram(&mut self, port_index: PortIndex, packet: Ctrlgram<C, D>) {
        self.port_mut(port_index).accept_ctrlgram(packet);
    }

    fn accept_datagram(&mut self, port_index: PortIndex, packet: Datagram<C, D>) {
        self.port_mut(port_index).accept_datagram(packet);
    }
}
