//! Ports of the deterministic `Data.Doublets.Sequences` building blocks:
//! `BalancedVariantConverter`, `SequenceIndex`, and `LinkFrequenciesCache`.
//!
//! These three are the low-risk, dependency-order-first pieces of the upstream
//! library (Phase 2 of the issue #531 solution plan). They are pure functions
//! over a [`SequenceStore`] with no combinatorial search, so their behaviour can
//! be pinned exactly against the C# sources with small fixtures.

use std::collections::HashMap;

use super::store::{Doublet, LinkAddress, SequenceStore, NULL_LINK};

/// Fold a flat element sequence into a balanced binary doublet tree, mirroring
/// `BalancedVariantConverter<TLinkAddress>.Convert`.
///
/// The algorithm halves the working sequence layer by layer, pairing adjacent
/// elements through [`SequenceStore::get_or_create`] and carrying a trailing odd
/// element unchanged, until two elements remain; those become the root doublet.
/// Because pairing goes through `get_or_create`, equal sub-sequences collapse to
/// the same links — deduplication is a side effect of the balanced fold.
///
/// - an empty sequence converts to the null link;
/// - a single element converts to itself (already a valid link);
/// - otherwise the result is the balanced root whose [`SequenceStore::expand`]
///   reproduces the input exactly.
#[must_use]
pub fn balanced_convert(store: &mut SequenceStore, sequence: &[LinkAddress]) -> LinkAddress {
    match sequence.len() {
        0 => NULL_LINK,
        1 => sequence[0],
        _ => {
            let mut current = halve_sequence(store, sequence);
            while current.len() > 2 {
                current = halve_sequence(store, &current);
            }
            store.get_or_create(current[0], current[1])
        }
    }
}

/// Pair adjacent elements of `source` through `get_or_create`, carrying a
/// trailing odd element unchanged. The result is `ceil(len / 2)` long. This is
/// the `HalveSequence` helper of the balanced converter.
fn halve_sequence(store: &mut SequenceStore, source: &[LinkAddress]) -> Vec<LinkAddress> {
    let length = source.len();
    let looped_length = length - (length % 2);
    let mut destination = Vec::with_capacity(length / 2 + length % 2);
    let mut index = 0;
    while index < looped_length {
        destination.push(store.get_or_create(source[index], source[index + 1]));
        index += 2;
    }
    if length > looped_length {
        destination.push(source[length - 1]);
    }
    destination
}

/// Adjacent-pair index over sequences, mirroring `SequenceIndex<TLinkAddress>`.
///
/// Indexing a sequence records every adjacent `(previous, current)` pair as a
/// link. [`SequenceIndex::might_contain`] then answers, without materialising a
/// root, whether a candidate sequence *could* already be stored: it can only be
/// present if all of its adjacent pairs are already indexed.
#[derive(Debug, Default)]
pub struct SequenceIndex;

impl SequenceIndex {
    /// Index every adjacent pair of `sequence`, returning whether the whole
    /// sequence was already fully indexed beforehand.
    ///
    /// Following the upstream scan, this walks from the end while pairs are
    /// already present, then creates the remaining prefix pairs. The return
    /// value reports the pre-existing state, matching `Add`'s semantics.
    pub fn add(store: &mut SequenceStore, sequence: &[LinkAddress]) -> bool {
        if sequence.len() < 2 {
            return true;
        }
        let mut index = sequence.len() - 1;
        let mut indexed = true;
        while index >= 1 {
            if store.search(sequence[index - 1], sequence[index]).is_some() {
                index -= 1;
            } else {
                indexed = false;
                break;
            }
        }
        if !indexed {
            for lower in (1..=index).rev() {
                store.get_or_create(sequence[lower - 1], sequence[lower]);
            }
        }
        indexed
    }

    /// Whether `sequence` might already be stored: true only if every adjacent
    /// pair is already indexed. This never mutates the store.
    #[must_use]
    pub fn might_contain(store: &SequenceStore, sequence: &[LinkAddress]) -> bool {
        sequence
            .windows(2)
            .all(|pair| store.search(pair[0], pair[1]).is_some())
    }
}

/// Frequency of an adjacent doublet, plus the link that stores it (if any).
///
/// Mirrors `LinkFrequency<TLinkAddress>`: the count of observations and the
/// backing link address (`0`/null when the pair has not been materialised).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkFrequency {
    /// Number of times the doublet has been observed.
    pub frequency: u64,
    /// The link storing the doublet, or the null link when not yet created.
    pub link: LinkAddress,
}

/// Cache of adjacent-doublet frequencies, mirroring `LinkFrequenciesCache`.
///
/// The compressing deduplicator uses it to find the most frequent adjacent pair
/// to replace. Keeping it separate (as upstream does) lets frequencies persist
/// across multiple compression passes.
#[derive(Debug, Clone, Default)]
pub struct LinkFrequenciesCache {
    cache: HashMap<Doublet, LinkFrequency>,
}

impl LinkFrequenciesCache {
    /// Create an empty frequency cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of distinct doublets tracked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Whether no doublets have been counted yet.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Return the recorded frequency of `(source, target)`, or a zero-count
    /// entry when the pair has never been counted.
    #[must_use]
    pub fn get(&self, source: LinkAddress, target: LinkAddress) -> LinkFrequency {
        self.cache
            .get(&Doublet::new(source, target))
            .copied()
            .unwrap_or(LinkFrequency {
                frequency: 0,
                link: NULL_LINK,
            })
    }

    /// Increment the frequency of `(source, target)` by one, seeding the backing
    /// link from the store when the pair already exists there. Returns the
    /// updated entry.
    pub fn increment(
        &mut self,
        store: &SequenceStore,
        source: LinkAddress,
        target: LinkAddress,
    ) -> LinkFrequency {
        let doublet = Doublet::new(source, target);
        let entry = self.cache.entry(doublet).or_insert_with(|| LinkFrequency {
            frequency: 0,
            link: store.search(source, target).unwrap_or(NULL_LINK),
        });
        entry.frequency += 1;
        *entry
    }

    /// Increment every adjacent pair of `sequence`, mirroring
    /// `IncrementFrequencies`.
    pub fn increment_sequence(&mut self, store: &SequenceStore, sequence: &[LinkAddress]) {
        for pair in sequence.windows(2) {
            self.increment(store, pair[0], pair[1]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sequences::symbols::SymbolTable;

    fn points(
        store: &mut SequenceStore,
        symbols: &mut SymbolTable,
        values: &[u64],
    ) -> Vec<LinkAddress> {
        values.iter().map(|&v| symbols.scalar(store, v)).collect()
    }

    #[test]
    fn balanced_convert_edge_cases() {
        let mut store = SequenceStore::new();
        assert_eq!(balanced_convert(&mut store, &[]), NULL_LINK);
        let a = store.create_point();
        assert_eq!(balanced_convert(&mut store, &[a]), a);
    }

    #[test]
    fn balanced_convert_round_trips_and_deduplicates() {
        let mut store = SequenceStore::new();
        let mut symbols = SymbolTable::new();
        let seq = points(&mut store, &mut symbols, &[1, 2, 3, 4, 5]);
        let root = balanced_convert(&mut store, &seq);
        assert_eq!(
            store.expand(root),
            seq,
            "balanced tree expands to the input"
        );

        // Converting the same sequence again yields the same root (structural
        // deduplication), allocating no new links.
        let before = store.len();
        let root_again = balanced_convert(&mut store, &seq);
        assert_eq!(root, root_again);
        assert_eq!(store.len(), before, "second conversion reuses every link");
    }

    #[test]
    fn balanced_convert_pairs_before_carrying_odd_tail() {
        let mut store = SequenceStore::new();
        let mut symbols = SymbolTable::new();
        let seq = points(&mut store, &mut symbols, &[1, 2, 3]);
        let root = balanced_convert(&mut store, &seq);
        // Layer one pairs (1,2) and carries 3 -> [(1 2), 3]; root = ((1 2) 3).
        let root_doublet = store.get(root).unwrap();
        assert_eq!(root_doublet.target, seq[2]);
        assert_eq!(store.expand(root_doublet.source), vec![seq[0], seq[1]]);
    }

    #[test]
    fn sequence_index_add_and_might_contain() {
        let mut store = SequenceStore::new();
        let mut symbols = SymbolTable::new();
        let seq = points(&mut store, &mut symbols, &[1, 2, 3]);
        assert!(!SequenceIndex::might_contain(&store, &seq));
        let already = SequenceIndex::add(&mut store, &seq);
        assert!(!already, "first add reports the sequence was not indexed");
        assert!(SequenceIndex::might_contain(&store, &seq));
        assert!(
            SequenceIndex::add(&mut store, &seq),
            "second add reports it was already indexed"
        );
    }

    #[test]
    fn frequencies_count_adjacent_pairs() {
        let mut store = SequenceStore::new();
        let mut symbols = SymbolTable::new();
        // A B A B: the pair (A,B) occurs twice, (B,A) once.
        let seq = points(&mut store, &mut symbols, &[10, 20, 10, 20]);
        let mut cache = LinkFrequenciesCache::new();
        cache.increment_sequence(&store, &seq);
        assert_eq!(cache.get(seq[0], seq[1]).frequency, 2);
        assert_eq!(cache.get(seq[1], seq[2]).frequency, 1);
        assert_eq!(cache.get(seq[2], seq[3]).frequency, 2, "same (A,B) doublet");
        assert_eq!(cache.get(seq[1], seq[0]).frequency, 1);
        // Unobserved pair reports a zero entry.
        assert_eq!(cache.get(seq[0], seq[0]).frequency, 0);
    }
}
