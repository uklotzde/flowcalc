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

#[derive(Debug)]
struct SealedTag;
