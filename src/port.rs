/// TODO
#[derive(Default, Debug, Clone, Copy)]
pub struct Port<T> {
    state: PortState,

    /// The current value sent to input and input port or
    /// received by an output port.
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
    value: Option<T>,
}

/// TODO
#[derive(Debug, Default, Clone, Copy, Eq, PartialEq)]
pub struct PortState {
    /// TODO
    pub activity: ActivityState,

    pub(crate) connectivity: ConnectivityState,
}

impl PortState {
    /// TODO
    pub const fn new() -> Self {
        Self {
            activity: ActivityState::new(),
            connectivity: ConnectivityState::new(),
        }
    }

    /// TODO
    pub fn is_active(self) -> bool {
        self.activity == ActivityState::Active
    }

    /// TODO
    pub fn is_connected(self) -> bool {
        self.connectivity == ConnectivityState::Connected
    }
}

/// TODO
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActivityState {
    /// TODO
    Inactive,

    /// TODO
    Active,
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

/// TODO
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ConnectivityState {
    /// TODO
    Disconnected,

    /// TODO
    Connected,
}

impl ConnectivityState {
    /// TODO
    pub const fn new() -> Self {
        Self::Disconnected
    }
}

impl Default for ConnectivityState {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Port<T> {
    /// TODO
    pub const fn new() -> Self {
        Self {
            state: PortState::new(),
            value: None,
        }
    }

    /// TODO
    pub const fn state(&self) -> PortState {
        self.state
    }

    /// TODO
    pub fn set_state(&mut self, new_state: PortState) {
        self.state = new_state;
    }

    /// TODO
    pub fn activate(&mut self, is_active: bool) {
        let new_activity = if is_active {
            ActivityState::Active
        } else {
            ActivityState::Inactive
        };
        self.state = PortState {
            activity: new_activity,
            ..self.state
        }
    }

    /// TODO
    pub fn put_value(&mut self, value: Option<T>) {
        debug_assert!(self.state().is_active() == value.is_some());
        self.value = value;
    }

    /// TODO
    pub fn take_value(&mut self) -> Option<T> {
        debug_assert!(self.state().is_active() == self.value.is_some());
        self.value.take()
    }

    /// TODO
    pub fn reset_value(&mut self) {
        debug_assert!(!self.state().is_active());
        self.value = None;
    }
}

/// TODO
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PortIndex(usize);

/// TODO
impl PortIndex {
    /// TODO
    pub fn new(index: usize) -> Self {
        Self(index)
    }
}

impl From<PortIndex> for usize {
    fn from(from: PortIndex) -> Self {
        from.0
    }
}
