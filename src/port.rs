/// TODO
#[derive(Default, Debug, Clone, Copy)]
pub struct Port<T> {
    state: PortState,
    value: Option<T>,
}

/// TODO
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum PortState {
    /// TODO
    Inactive,

    /// TODO
    Active,
}

impl Default for PortState {
    fn default() -> Self {
        Self::Inactive
    }
}

/// TODO
impl PortState {
    /// TODO
    pub fn is_active(self) -> bool {
        self == Self::Active
    }
}

impl<T> Port<T> {
    /// TODO
    pub const fn new() -> Self {
        Self {
            state: PortState::Inactive,
            value: None,
        }
    }

    /// TODO
    pub const fn state(&self) -> PortState {
        self.state
    }

    /// TODO
    pub fn is_active(&self) -> bool {
        self.state().is_active()
    }

    /// TODO
    pub fn activate(&mut self, is_active: bool) {
        if is_active {
            self.state = PortState::Active;
        } else {
            self.state = PortState::Inactive;
        }
    }

    /// TODO
    pub fn get_value(&self) -> &Option<T> {
        &self.value
    }

    /// TODO
    pub fn take_value(&mut self) -> Option<T> {
        self.value.take()
    }

    /// TODO
    pub fn set_value(&mut self, value: Option<T>) {
        self.value = value;
    }

    /// TODO
    pub fn reset_value(&mut self) {
        self.value = None;
    }

    /// TODO
    pub fn set_value_from<O>(&mut self, source: &mut Port<O>)
    where
        O: Into<T>,
    {
        debug_assert!(self.is_active());
        debug_assert!(source.is_active());
        self.value = source.value.take().map(Into::into)
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
