use super::{ActivityState, Packet};

use std::fmt;

/// TODO
#[derive(Default, Debug, Clone, Copy)]
pub struct Port<T> {
    pub(crate) state: PortState,

    /// A slot (= a buffer with capacity 1) for storing the
    /// next or current value
    ///
    /// This slot is dedicated to receive input values
    /// from preceding update stages and for sending
    /// output values to subsequent updates stages when
    /// updating the flow graph.
    ///
    /// Values of input ports are consumed when updating
    /// the output ports of a node. Values of output ports
    /// are consumed when transferring the value to connected
    /// input ports of subsequent nodes.
    pub slot: Option<T>,
}

/// TODO
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct PortState {
    pub(crate) activity: ActivityState,
}

impl PortState {
    /// TODO
    pub const fn new() -> Self {
        Self {
            activity: ActivityState::new(),
        }
    }

    /// TODO
    pub fn is_active(self) -> bool {
        self.activity == ActivityState::Active
    }
}

impl ActivityState {
    /// TODO
    pub const fn new() -> Self {
        Self::Inactive
    }
}

impl Default for ActivityState {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Port<T> {
    /// TODO
    pub const fn new() -> Self {
        Self {
            state: PortState::new(),
            slot: None,
        }
    }

    /// TODO
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    /// TODO
    pub fn activate(&mut self, is_active: bool) {
        let activity = if is_active {
            ActivityState::Active
        } else {
            ActivityState::Inactive
        };
        self.state = PortState { activity };
    }

    /// TODO
    pub fn accept_packet(&mut self, packet: Packet<T>) {
        let Packet {
            state: activity,
            value,
        } = packet;
        self.state = PortState {
            activity,
            ..self.state
        };
        // The capacity must never be exhausted, i.e. the
        // slot must be unoccupied when receiving a packet.
        debug_assert!(self.slot.is_none());
        self.slot = value;
    }

    /// TODO
    pub fn dispatch_packet(&mut self) -> Packet<T> {
        Packet {
            state: self.state.activity,
            value: self.slot.take(),
        }
    }
}

/// TODO
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortIndex(usize);

/// TODO
impl PortIndex {
    /// TODO
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
pub trait PortBay<T>: fmt::Debug {
    /// The number of ports in this bay
    ///
    /// Port indexes are 0-based in the range 0..num_ports().
    fn num_ports(&self) -> usize;

    /// Receive an incoming packet and store it into the given port
    fn accept_packet(&mut self, port_index: PortIndex, packet: Packet<T>);

    /// Fetch and dispatch an outgoing packet from the given port
    fn dispatch_packet(&mut self, port_index: PortIndex) -> Packet<T>;
}

/// TODO
#[derive(Default, Debug, Clone)]
pub struct VecPortBay<T> {
    ports: Vec<Port<T>>,
}

impl<T> VecPortBay<T> {
    /// TODO
    pub fn new(num_ports: usize) -> Self {
        let mut ports = Vec::with_capacity(num_ports);
        for _ in 0..num_ports {
            ports.push(Port::new());
        }
        Self { ports }
    }

    /// TODO
    pub fn ports(&self) -> impl Iterator<Item = &Port<T>> {
        self.ports.iter()
    }

    /// TODO
    pub fn ports_mut(&mut self) -> impl Iterator<Item = &mut Port<T>> {
        self.ports.iter_mut()
    }

    /// TODO
    pub fn port(&self, port_index: PortIndex) -> &Port<T> {
        let index = usize::from(port_index);
        debug_assert!(index < self.ports.len());
        &self.ports[index]
    }

    /// TODO
    pub fn port_mut(&mut self, port_index: PortIndex) -> &mut Port<T> {
        let index = usize::from(port_index);
        debug_assert!(index < self.ports.len());
        &mut self.ports[index]
    }
}

impl<T> PortBay<T> for VecPortBay<T>
where
    T: fmt::Debug,
{
    fn num_ports(&self) -> usize {
        self.ports.len()
    }

    /// Receive an incoming packet and store it into the given port
    fn accept_packet(&mut self, port_index: PortIndex, packet: Packet<T>) {
        self.port_mut(port_index).accept_packet(packet);
    }

    /// Fetch and dispatch an outgoing packet from the given port
    fn dispatch_packet(&mut self, port_index: PortIndex) -> Packet<T> {
        self.port_mut(port_index).dispatch_packet()
    }
}
