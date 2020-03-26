//#![deny(missing_docs)]
//#![deny(missing_debug_implementations)]
#![deny(intra_doc_link_resolution_failure)]
#![cfg_attr(test, deny(warnings))]

//! # flowcalc
//!
//! Components for building and executing simple _flow-based programming_ graphs.

/// The crate's prelude
///
/// A biased set of imports to ease usage of this crate.
pub mod prelude;

pub mod flow;

pub mod node;

pub mod port;

// Restricts the visibility of trait methods
#[derive(Debug)]
struct SealedTag;
