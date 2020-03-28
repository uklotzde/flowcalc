#[derive(Debug, Clone, Copy)]
pub struct Packet<P, B> {
    /// The payload
    ///
    /// Within a flow graph the payload in backward direction
    /// during the 1st phase contains a control message. In
    /// forward direction during the 2nd phase it contains
    /// the actual data message.
    pub payload: P,

    /// Optional pre-allocated data to be reused when passing
    /// back a complementary packet into the opposite direction
    ///
    /// Within a flow graph the piggyback in backward direction
    /// during the 1st phase contains a pre-allocated data message
    /// that is supposed to store the results when processing the
    /// actual data during the 2nd phase. In forward direction
    /// during the 2nd phase it contains the previously received
    /// control message that is supposed to be reused on subsequent
    /// invocations.
    pub piggyback: Option<B>,
}

/// An input or output port of a processing node.
///
/// Ports have two generic parameters for packet data:
///   - `I`: Incoming data
///   - `O`: Outgoing data
///
/// Input ports accept packets with an incoming data payload
/// in forward direction and dispatch packets with on outgoing
/// control payload in backward direction.
///
/// Output ports accept packets with an incoming control payload
/// in backward direction and dispatch packets with an outgoing
/// data payload in forward direction.
#[derive(Default, Debug, Clone, Copy)]
pub struct Port<I, O> {
    /// A slot (= a buffer with capacity 1) for the payload of an
    /// acceptable (= incoming) package
    pub incoming: Option<I>,

    /// A slot (= a buffer with capacity 1) for the playload of an
    /// dispatchable (= outgoing) packet
    pub outgoing: Option<O>,
}

impl<I, O> Port<I, O> {
    pub const fn new() -> Self {
        Self {
            incoming: None,
            outgoing: None,
        }
    }

    /// Accept a packet with an incoming payload
    pub fn accept_packet(&mut self, packet: Packet<I, O>) {
        let Packet {
            payload: incoming,
            piggyback: outgoing,
        } = packet;
        self.incoming = Some(incoming);
        self.outgoing = outgoing;
    }

    /// Try to dispatch a packet with an outgoing payload
    pub fn try_dispatch_packet(&mut self) -> Option<Packet<O, I>> {
        if let Some(outgoing) = self.outgoing.take() {
            Some(Packet {
                payload: outgoing,
                piggyback: self.incoming.take(),
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
pub trait PortBay<I, O> {
    /// The number of ports in this bay
    ///
    /// Port indexes are 0-based in the range 0..num_ports().
    fn num_ports(&self) -> usize;

    /// Receive and store an incoming packet for the given port
    fn accept_packet(&mut self, port_index: PortIndex, packet: Packet<I, O>);

    /// Fetch and dispatch an outgoing packet from the given port
    fn try_dispatch_packet(&mut self, port_index: PortIndex) -> Option<Packet<O, I>>;
}

#[derive(Default, Debug, Clone)]
pub struct VecPortBay<I, O> {
    ports: Vec<Port<I, O>>,
}

impl<I, O> VecPortBay<I, O> {
    pub fn new(num_ports: usize) -> Self {
        let mut ports = Vec::with_capacity(num_ports);
        for _ in 0..num_ports {
            ports.push(Port::new());
        }
        Self { ports }
    }

    pub fn ports(&self) -> impl Iterator<Item = &Port<I, O>> {
        self.ports.iter()
    }

    pub fn ports_mut(&mut self) -> impl Iterator<Item = &mut Port<I, O>> {
        self.ports.iter_mut()
    }

    pub fn port(&self, port_index: PortIndex) -> &Port<I, O> {
        let index = usize::from(port_index);
        debug_assert!(index < self.ports.len());
        &self.ports[index]
    }

    pub fn port_mut(&mut self, port_index: PortIndex) -> &mut Port<I, O> {
        let index = usize::from(port_index);
        debug_assert!(index < self.ports.len());
        &mut self.ports[index]
    }
}

impl<I, O> PortBay<I, O> for VecPortBay<I, O> {
    fn num_ports(&self) -> usize {
        self.ports.len()
    }

    fn accept_packet(&mut self, port_index: PortIndex, packet: Packet<I, O>) {
        self.port_mut(port_index).accept_packet(packet);
    }

    fn try_dispatch_packet(&mut self, port_index: PortIndex) -> Option<Packet<O, I>> {
        self.port_mut(port_index).try_dispatch_packet()
    }
}
