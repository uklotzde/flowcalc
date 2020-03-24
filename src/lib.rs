#![deny(missing_docs)]
#![deny(missing_debug_implementations)]
#![deny(intra_doc_link_resolution_failure)]
#![cfg_attr(test, deny(warnings))]

//! # flowcalc
//!
//! Components for building and executing simple _flow-based programming_ graphs.

/// The crate's prelude
///
/// A biased set of imports to ease usage of this crate.
pub mod prelude;

/// TODO
pub mod flow;

/// TODO
pub mod node;

/// TODO
pub mod port;

// Restricts the visibility of trait methods
#[derive(Debug)]
struct SealedTag;

/// TODO
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum ActivityState {
    /// TODO
    Inactive,

    /// TODO
    Active,
}

/// A chunk of data that is moved between connected ports
#[derive(Default, Debug, Clone, Copy)]
pub struct Packet<T> {
    /// The state of the sender
    pub state: ActivityState,

    /// The value that is moved from sender to receiver
    pub value: Option<T>,
}
