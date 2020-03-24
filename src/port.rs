use super::{ActivityState, Packet};

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
    pub fn dispatch_packet(&mut self) -> Packet<T> {
        Packet {
            state: self.state.activity,
            value: self.slot.take(),
        }
    }

    /// TODO
    pub fn receive_packet(&mut self, packet: Packet<T>) {
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
