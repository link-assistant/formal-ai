//! Embedded Links Notation seed files and the file registry.
//!
//! Every registered `data/seed/*.lino` file is compiled into the binary with
//! [`include_str!`] so even offline builds expose the same data the browser
//! fetches at runtime. [`seed_files`] returns them in registry order -- sorted
//! by name -- so callers can render the merged bundle deterministically, and
//! [`MEANING_FILES`] names the subset that make up the language-independent
//! meaning lexicon (see [`super::lexicon`]).
//!
//! The inventory itself is generated. Issue #991: this file needed manual
//! conflict resolution 27 times because every branch that added a seed file
//! appended to the same three lists here. The lists now live in
//! `data/meta/seed-registry.lino`, which is `merge=union`, and
//! `rust-script scripts/generate-seed-registry.rs --write` writes
//! `embedded_registry.rs` from it. `include!` rather than `mod` keeps the
//! generated file outside rustfmt's reach, so the generator owns every byte of
//! it and a regeneration is always byte-identical.

include!("embedded_registry.rs");
