# Issue 531 Architecture Inventory

This inventory records where pattern inference should attach to the current
Rust codebase and what the upstream Doublets sequence package contributes.

## Current Formal AI Boundaries

### Link storage

`src/link_store.rs` is the central storage boundary. It defines `DoubletLink`,
`LinkRecord`, `LinkStoreBackend`, the `LinkStore` trait, import/export methods,
and native doublets backend selection through the default `doublets-native`
feature. Future sequence support should be expressed through this boundary so
Links Notation, in-memory stores, and native doublets remain interchangeable.

### Link-pattern substitution

`src/substitution.rs` already implements data-driven link-pattern substitution.
It defines substitution links, link patterns, rules, and rule sets over source
and target nodes. This is not sequence inference, but it is the closest local
pattern surface and should be reused for later "replace repeated structure"
behavior where possible.

### Solver and meta-core

`src/solver.rs` describes the universal reasoning loop, including simplification
by collapsing meaning-preserving redundancies. `src/meta_core.rs` records issue
#559 style recursive work units, evidence, method selection, and skill ledgers.
Pattern inference should become a method in this loop, with traces recorded as
data, not a side-channel algorithm hidden from the meta-core.

### Existing text deduplication

`src/solver_handlers/text_manipulation.rs` includes a `DeduplicateLines`
operation. It is useful as a simple user-facing operation, but it does not
discover repeated sub-sequences, build a link-native compression tree, or
generalize to transformations. Future work can use it as a baseline behavior
only.

### Dependency surface

`Cargo.toml` already depends on `doublets = "0.4.0"` behind the default
`doublets-native` feature and on `meta-language = "0.45.0"`. A sequence module
should not add a new graph storage dependency until it proves the existing
doublets boundary cannot support the required operations.

## Upstream Data.Doublets.Sequences Inventory

The raw first-pass source excerpts are saved under `raw-data/`.

- `BalancedVariantConverter` recursively halves the symbol list and creates
  doublets until one root doublet remains. It is the safest first converter to
  port because it has deterministic structure and a narrow dependency surface.
- `OptimalVariantConverter` uses local element levels/frequencies to choose
  pairings. It should be implemented after the frequency cache behavior is
  reproduced by tests.
- `CompressingConverter` repeatedly substitutes selected adjacent pairs with a
  created/reused link. It matches the issue's "associative deduplication" goal,
  but its max-frequency condition must be verified before copying behavior.
- `LinkFrequenciesCache` tracks pair frequency and existing link counters.
  Formal AI should keep the first Rust version explicit and testable before
  attempting storage-level frequency persistence.
- `SequenceIndex` indexes adjacent sequence pairs with `GetOrCreate`.
  A Rust `SequenceIndex` can be a small bridge between sequence ingestion and
  later pattern lookup.
- `StringToUnicodeSequenceConverter` shows the expected layering:
  string -> unicode symbols -> sequence converter -> optional sequence index.

## Gaps To Close

- There is no `src/sequences.rs` or equivalent link-native sequence API.
- There is no stable symbol initialization layer for sequence elements,
  unicode symbols, or grid colors.
- There is no balanced, optimal, or compressing sequence-tree builder.
- There is no frequency cache for adjacent doublets.
- There is no compression trace that can expand back to the original sequence.
- There is no transformed matcher for reverse, shift, substitution, rotation,
  reflection, or translation.
- There is no 2D grid projection layer for rows, columns, diagonals, boundaries,
  or relative coordinates.
- There are not yet seed meanings for the pattern-inference vocabulary.

## Recommended Integration Shape

The smallest coherent implementation unit is a `sequence` module that stores
only typed link IDs and leaves storage behind existing `LinkStore` interfaces.
It should expose:

- symbol initialization and lookup;
- sequence markers and roots;
- balanced sequence construction;
- adjacent-pair indexing;
- optional frequency cache and compression trace;
- exact expansion back to original symbols;
- fixtures for text and grid sequences.

Once that substrate exists, solver and meta-core integration should call it as a
method and record the resulting sequence tree, compression trace, candidate
patterns, and verification evidence as data.
