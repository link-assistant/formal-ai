//! Self-contained associative doublet store for link-native sequences.
//!
//! Issue #531 asks us to reimplement the `linksplatform/Data.Doublets.Sequences`
//! machinery in Rust so pattern inference can be grounded in links rather than
//! ad hoc strings. The upstream converters are written against the `ILinks`
//! interface whose two load-bearing primitives are `GetOrCreate(source, target)`
//! and `SearchOrDefault(source, target)`. The crate-wide [`crate::link_store`]
//! abstraction deliberately does not expose raw doublet CRUD (it is a
//! memory-event projection), so this module provides a small, dependency-free,
//! deterministic doublet store that the sequence algorithms build on.
//!
//! The store follows Links meta-theory conventions:
//!
//! - address `0` is the reserved null/empty link (`Zero` in the C# sources);
//! - a *point* is a self-referential link whose source and target both equal its
//!   own address (`get(p) == (p, p)`); points model unique atomic symbols;
//! - a *doublet* (composite link) references two existing links as its source
//!   and target; `get_or_create` guarantees structural uniqueness so identical
//!   composite pairs always resolve to the same address (associative
//!   deduplication at the storage layer).
//!
//! A point and the self-pairing composite `(a, a)` are deliberately *different*
//! links: the point is an atomic identity, whereas `(a, a)` is the length-two
//! sequence "`a` followed by `a`". Keeping them distinct is what lets a run of
//! repeated atoms (e.g. `A A A A`) be represented and expanded losslessly; if
//! the point shadowed the composite, pairing an atom with itself would collapse
//! back to the atom and silently drop an element. Points are therefore
//! deduplicated by their allocator (the symbol table), while composites are
//! deduplicated by this store's structural index.
//!
//! Everything is content-addressed by structure, which is what lets the
//! sequence converters deduplicate repeated sub-structure for free: converting
//! the same pair twice returns the same link.

use std::collections::HashMap;

/// Address of a link inside a [`SequenceStore`]. `0` is the reserved null link.
pub type LinkAddress = u64;

/// The reserved null / empty link address (the `Zero` constant upstream).
pub const NULL_LINK: LinkAddress = 0;

/// An ordered pair of link addresses: the fundamental unit of the store.
///
/// A point stores its own address in both fields; a composite stores the two
/// links it connects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Doublet {
    /// The referenced source link.
    pub source: LinkAddress,
    /// The referenced target link.
    pub target: LinkAddress,
}

impl Doublet {
    /// Construct a doublet from a source and target address.
    #[must_use]
    pub const fn new(source: LinkAddress, target: LinkAddress) -> Self {
        Self { source, target }
    }
}

/// An append-only associative doublet store.
///
/// The store never mutates or deletes existing links, mirroring the
/// append-only, content-addressed discipline of the crate's event log and
/// keeping every derived address stable for the lifetime of a conversion.
#[derive(Debug, Clone, Default)]
pub struct SequenceStore {
    /// `links[address - 1]` holds the doublet stored at `address`. Address `0`
    /// is the reserved null link and is never present in this vector.
    links: Vec<Doublet>,
    /// Structural index from `(source, target)` to the address that stores it,
    /// so repeated pairs deduplicate to a single link.
    index: HashMap<(LinkAddress, LinkAddress), LinkAddress>,
}

impl SequenceStore {
    /// Create an empty store containing only the reserved null link.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of real links created so far (excluding the null link).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.links.len()
    }

    /// Whether the store holds no real links yet.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.links.is_empty()
    }

    /// The reserved null / empty link address.
    #[must_use]
    pub const fn null(&self) -> LinkAddress {
        NULL_LINK
    }

    /// Allocate a fresh unique point: a self-referential link `(a, a)` whose
    /// address `a` is returned. Points model atomic sequence elements (scalars,
    /// unicode code points, grid colours) that carry identity but no internal
    /// structure.
    pub fn create_point(&mut self) -> LinkAddress {
        let address = self.next_address();
        self.links.push(Doublet::new(address, address));
        // Points are intentionally not placed in the composite index: the point
        // `a` and the self-pairing composite `(a, a)` are distinct links (an
        // atom versus the length-two sequence `a a`). Indexing the point under
        // `(a, a)` would make `get_or_create(a, a)` collapse the pair back to the
        // atom and break lossless expansion of repeated-element runs.
        address
    }

    /// Look up the address of the link storing `(source, target)`, if any.
    ///
    /// This is the `SearchOrDefault(source, target)` primitive the upstream
    /// converters rely on; a missing pair returns [`None`] (the null link).
    #[must_use]
    pub fn search(&self, source: LinkAddress, target: LinkAddress) -> Option<LinkAddress> {
        self.index.get(&(source, target)).copied()
    }

    /// Return the address of the link storing `(source, target)`, creating it if
    /// it does not exist. This is the associative `GetOrCreate(source, target)`
    /// primitive: identical pairs always resolve to the same address, so shared
    /// sub-structure is stored exactly once. Pairing an atom with itself,
    /// `get_or_create(a, a)`, yields a fresh composite distinct from the point
    /// `a` (see the module documentation) so the two-element sequence `a a` is
    /// represented faithfully.
    ///
    /// Both `source` and `target` must be existing links (or the null link);
    /// referencing an out-of-range address is a programming error and panics in
    /// debug builds via the [`debug_assert`]s below.
    pub fn get_or_create(&mut self, source: LinkAddress, target: LinkAddress) -> LinkAddress {
        debug_assert!(self.is_valid(source), "source link must exist");
        debug_assert!(self.is_valid(target), "target link must exist");
        if let Some(existing) = self.search(source, target) {
            return existing;
        }
        let address = self.next_address();
        self.links.push(Doublet::new(source, target));
        self.index.insert((source, target), address);
        address
    }

    /// Fetch the doublet stored at `address`.
    ///
    /// The null link resolves to `(0, 0)`. Out-of-range addresses return
    /// [`None`].
    #[must_use]
    pub fn get(&self, address: LinkAddress) -> Option<Doublet> {
        if address == NULL_LINK {
            return Some(Doublet::new(NULL_LINK, NULL_LINK));
        }
        let idx = usize::try_from(address - 1).ok()?;
        self.links.get(idx).copied()
    }

    /// Whether `address` is the null link or a real allocated link.
    #[must_use]
    pub fn is_valid(&self, address: LinkAddress) -> bool {
        address == NULL_LINK || usize::try_from(address - 1).is_ok_and(|idx| idx < self.links.len())
    }

    /// Whether `address` is a point (self-referential atomic symbol).
    #[must_use]
    pub fn is_point(&self, address: LinkAddress) -> bool {
        address != NULL_LINK
            && self
                .get(address)
                .is_some_and(|doublet| doublet.source == address && doublet.target == address)
    }

    /// Flatten a link back into the ordered sequence of point addresses it was
    /// built from. Points expand to themselves; composites expand to the
    /// concatenation of their source and target expansions.
    ///
    /// This is the exact inverse of the balanced/compressing converters and is
    /// what makes every deduplication step verifiably lossless: `expand` after
    /// any conversion reproduces the original element sequence.
    #[must_use]
    pub fn expand(&self, address: LinkAddress) -> Vec<LinkAddress> {
        let mut output = Vec::new();
        self.expand_into(address, &mut output);
        output
    }

    fn expand_into(&self, address: LinkAddress, output: &mut Vec<LinkAddress>) {
        if address == NULL_LINK {
            return;
        }
        // Guard against a pathological cycle: a well-formed store built only
        // through `create_point`/`get_or_create` can never contain one because
        // both fields of a composite reference strictly-earlier addresses, but
        // the explicit check keeps `expand` total.
        match self.get(address) {
            Some(doublet) if doublet.source == address && doublet.target == address => {
                output.push(address);
            }
            Some(doublet) => {
                self.expand_into(doublet.source, output);
                self.expand_into(doublet.target, output);
            }
            None => output.push(address),
        }
    }

    const fn next_address(&self) -> LinkAddress {
        // Addresses are 1-based; the null link occupies the conceptual slot 0.
        self.links.len() as LinkAddress + 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_link_resolves_to_zero_doublet() {
        let store = SequenceStore::new();
        assert!(store.is_empty());
        assert_eq!(store.null(), NULL_LINK);
        assert_eq!(store.get(NULL_LINK), Some(Doublet::new(0, 0)));
        assert!(store.is_valid(NULL_LINK));
        assert!(!store.is_point(NULL_LINK));
    }

    #[test]
    fn points_are_self_referential_and_unique() {
        let mut store = SequenceStore::new();
        let a = store.create_point();
        let b = store.create_point();
        assert_ne!(a, b);
        assert!(store.is_point(a));
        assert!(store.is_point(b));
        assert_eq!(store.get(a), Some(Doublet::new(a, a)));
        assert_eq!(store.expand(a), vec![a]);
    }

    #[test]
    fn get_or_create_deduplicates_pairs() {
        let mut store = SequenceStore::new();
        let a = store.create_point();
        let b = store.create_point();
        let first = store.get_or_create(a, b);
        let second = store.get_or_create(a, b);
        assert_eq!(first, second, "identical pairs must resolve to one address");
        assert_eq!(store.len(), 3, "two points and one composite");
        assert_eq!(store.search(a, b), Some(first));
        assert_eq!(store.search(b, a), None);
    }

    #[test]
    fn expand_reproduces_the_original_elements() {
        let mut store = SequenceStore::new();
        let a = store.create_point();
        let b = store.create_point();
        let c = store.create_point();
        // Build ((a b) c) and confirm it flattens back to a, b, c.
        let ab = store.get_or_create(a, b);
        let abc = store.get_or_create(ab, c);
        assert_eq!(store.expand(abc), vec![a, b, c]);
    }

    #[test]
    fn self_pairing_a_point_is_a_distinct_composite() {
        let mut store = SequenceStore::new();
        let a = store.create_point();
        // Pairing an atom with itself is the length-two sequence `a a`, a
        // different link from the atom `a`, and it must expand losslessly.
        let aa = store.get_or_create(a, a);
        assert_ne!(aa, a, "the self-pair is not the point");
        assert!(
            !store.is_point(aa),
            "the self-pair is a composite, not a point"
        );
        assert!(store.is_point(a));
        assert_eq!(store.expand(aa), vec![a, a]);
        // Repeated requests still deduplicate to the same composite.
        assert_eq!(store.get_or_create(a, a), aa);
        assert_eq!(store.len(), 2, "one point and one composite");
    }
}
