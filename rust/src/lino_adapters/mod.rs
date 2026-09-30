//! Adapters that front our maintained dependencies behind the interfaces
//! this repository already uses, so adopting a dependency release is a
//! one-line change at the call site instead of a rewrite (issue #1182).
//!
//! - [`links_notation`] — the `links-notation` crate (installed 0.16.1) behind the
//!   seed parser's [`crate::seed::parser::LinoNode`] /
//!   [`crate::seed::parser::parse_lino`] interface.

pub mod links_notation;
