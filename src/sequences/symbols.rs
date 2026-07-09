//! Unique symbol allocation over a [`SequenceStore`].
//!
//! Before any sequence can be built, its atomic elements need stable link
//! identities. Upstream this is handled by dedicated "address to raw number"
//! and unicode-symbol converters plus typed markers. Here a single
//! [`SymbolTable`] provides deterministic, deduplicated points for three kinds
//! of atoms:
//!
//! - **scalar symbols** — arbitrary `u64` element ids (e.g. tokens, colours);
//! - **unicode symbols** — `char` code points, the basis for text sequences;
//! - **named markers** — typed sequence markers such as a "unicode sequence"
//!   or "text" tag that stamps the kind of a sequence root.
//!
//! Requesting the same value twice returns the same point, so equal atoms are
//! stored exactly once — associative deduplication starting at the leaves.

use std::collections::HashMap;

use super::store::{LinkAddress, SequenceStore};

/// Allocates and caches unique points for scalar, unicode, and named atoms.
#[derive(Debug, Clone, Default)]
pub struct SymbolTable {
    scalars: HashMap<u64, LinkAddress>,
    unicode: HashMap<char, LinkAddress>,
    markers: HashMap<String, LinkAddress>,
}

impl SymbolTable {
    /// Create an empty symbol table.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The total number of distinct atoms allocated so far.
    #[must_use]
    pub fn len(&self) -> usize {
        self.scalars.len() + self.unicode.len() + self.markers.len()
    }

    /// Whether no atoms have been allocated yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.scalars.is_empty() && self.unicode.is_empty() && self.markers.is_empty()
    }

    /// Return the point for scalar element `value`, creating it in `store` on
    /// first use. Equal values always map to the same point.
    pub fn scalar(&mut self, store: &mut SequenceStore, value: u64) -> LinkAddress {
        if let Some(&point) = self.scalars.get(&value) {
            return point;
        }
        let point = store.create_point();
        self.scalars.insert(value, point);
        point
    }

    /// Look up the point for scalar `value` without allocating.
    #[must_use]
    pub fn scalar_lookup(&self, value: u64) -> Option<LinkAddress> {
        self.scalars.get(&value).copied()
    }

    /// Return the point for unicode code point `ch`, creating it on first use.
    pub fn unicode(&mut self, store: &mut SequenceStore, ch: char) -> LinkAddress {
        if let Some(&point) = self.unicode.get(&ch) {
            return point;
        }
        let point = store.create_point();
        self.unicode.insert(ch, point);
        point
    }

    /// Return the point for the named marker `name`, creating it on first use.
    /// Markers give sequence roots a typed tag (e.g. `"unicode_sequence"`).
    pub fn marker(&mut self, store: &mut SequenceStore, name: &str) -> LinkAddress {
        if let Some(&point) = self.markers.get(name) {
            return point;
        }
        let point = store.create_point();
        self.markers.insert(name.to_owned(), point);
        point
    }

    /// Convert a string into the ordered list of unicode-symbol points backing
    /// it, allocating new points for previously-unseen code points. This is the
    /// Rust analogue of `StringToUnicodeSymbolsListConverter`.
    pub fn unicode_symbols(&mut self, store: &mut SequenceStore, text: &str) -> Vec<LinkAddress> {
        text.chars().map(|ch| self.unicode(store, ch)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_are_deduplicated() {
        let mut store = SequenceStore::new();
        let mut symbols = SymbolTable::new();
        let a = symbols.scalar(&mut store, 7);
        let b = symbols.scalar(&mut store, 7);
        let c = symbols.scalar(&mut store, 8);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_eq!(symbols.len(), 2);
        assert_eq!(symbols.scalar_lookup(7), Some(a));
        assert_eq!(symbols.scalar_lookup(9), None);
    }

    #[test]
    fn unicode_symbols_reuse_repeated_characters() {
        let mut store = SequenceStore::new();
        let mut symbols = SymbolTable::new();
        let points = symbols.unicode_symbols(&mut store, "abba");
        assert_eq!(points.len(), 4);
        assert_eq!(points[0], points[3], "both 'a's share a point");
        assert_eq!(points[1], points[2], "both 'b's share a point");
        assert_ne!(points[0], points[1]);
        // Only two distinct code points were allocated.
        assert_eq!(symbols.len(), 2);
    }

    #[test]
    fn markers_are_stable_by_name() {
        let mut store = SequenceStore::new();
        let mut symbols = SymbolTable::new();
        let first = symbols.marker(&mut store, "unicode_sequence");
        let second = symbols.marker(&mut store, "unicode_sequence");
        let other = symbols.marker(&mut store, "text");
        assert_eq!(first, second);
        assert_ne!(first, other);
        assert!(store.is_point(first));
    }
}
