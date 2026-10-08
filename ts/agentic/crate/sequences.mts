// The part of rust/src/sequences/ that `crate::algorithm_discovery` reaches:
// `SequenceStore` (store.rs), `SymbolTable::marker` (symbols.rs),
// `balanced_convert` (converter.rs) and `compress` with its lossless check
// (compression.rs). Link addresses are 1-based numbers; 0 is the null link.

/** Mirrors `NULL_LINK` in rust/src/sequences/store.rs. */
export const NULL_LINK = 0;

/** Mirrors `struct SequenceStore` in rust/src/sequences/store.rs (append-only doublets). */
export class SequenceStore {
  /** Mirrors `SequenceStore::new`. */
  constructor() {
    this.links = [];
    this.index = new Map();
  }

  /** Mirrors `SequenceStore::create_point`. */
  createPoint() {
    const address = this.links.length + 1;
    this.links.push([address, address]);
    return address;
  }

  /** Mirrors `SequenceStore::get_or_create`. */
  getOrCreate(source, target) {
    const key = `${source},${target}`;
    const existing = this.index.get(key);
    if (existing !== undefined) return existing;
    const address = this.links.length + 1;
    this.links.push([source, target]);
    this.index.set(key, address);
    return address;
  }

  /** Mirrors `SequenceStore::get`: the `[source, target]` doublet or undefined. */
  get(address) {
    return address >= 1 && address <= this.links.length ? this.links[address - 1] : undefined;
  }

  /** Mirrors `SequenceStore::expand`. */
  expand(address) {
    const output = [];
    const into = (link) => {
      if (link === NULL_LINK) return;
      const doublet = this.get(link);
      if (doublet === undefined || (doublet[0] === link && doublet[1] === link)) {
        output.push(link);
        return;
      }
      into(doublet[0]);
      into(doublet[1]);
    };
    into(address);
    return output;
  }
}

/** Mirrors `struct SymbolTable` in rust/src/sequences/symbols.rs. */
export class SymbolTable {
  constructor() {
    this.markers = new Map();
  }

  /** Mirrors `SymbolTable::marker`. */
  marker(store, name) {
    if (!this.markers.has(name)) this.markers.set(name, store.createPoint());
    return this.markers.get(name);
  }
}

/** Mirrors `fn halve_sequence` in rust/src/sequences/converter.rs. */
function halveSequence(store, source) {
  const destination = [];
  const looped = source.length - (source.length % 2);
  for (let index = 0; index < looped; index += 2) destination.push(store.getOrCreate(source[index], source[index + 1]));
  if (source.length > looped) destination.push(source[source.length - 1]);
  return destination;
}

/** Mirrors `fn balanced_convert` in rust/src/sequences/converter.rs. */
export function balancedConvert(store, sequence) {
  if (sequence.length === 0) return NULL_LINK;
  if (sequence.length === 1) return sequence[0];
  if (sequence.length === 2) return store.getOrCreate(sequence[0], sequence[1]);
  let current = halveSequence(store, sequence);
  while (current.length > 2) current = halveSequence(store, current);
  return store.getOrCreate(current[0], current[1]);
}

/** Mirrors `fn count_non_overlapping` in rust/src/sequences/compression.rs. */
function countNonOverlapping(sequence, [first, second]) {
  let count = 0;
  let index = 0;
  while (index + 1 < sequence.length) {
    if (sequence[index] === first && sequence[index + 1] === second) {
      count += 1;
      index += 2;
    } else {
      index += 1;
    }
  }
  return count;
}

/** Mirrors `fn most_frequent_pair` in rust/src/sequences/compression.rs (ties: smallest pair). */
function mostFrequentPair(sequence) {
  const seen = new Map();
  for (let index = 0; index + 1 < sequence.length; index += 1) {
    seen.set(`${sequence[index]},${sequence[index + 1]}`, [sequence[index], sequence[index + 1]]);
  }
  let best = null;
  for (const pair of seen.values()) {
    const count = countNonOverlapping(sequence, pair);
    if (count < 2) continue;
    if (best && (best.count > count || (best.count === count
      && (best.pair[0] < pair[0] || (best.pair[0] === pair[0] && best.pair[1] < pair[1]))))) continue;
    best = { pair, count };
  }
  return best;
}

/** Mirrors `fn replace_pair` in rust/src/sequences/compression.rs. */
function replacePair(sequence, [first, second], replacement) {
  const output = [];
  let index = 0;
  while (index < sequence.length) {
    if (index + 1 < sequence.length && sequence[index] === first && sequence[index + 1] === second) {
      output.push(replacement);
      index += 2;
    } else {
      output.push(sequence[index]);
      index += 1;
    }
  }
  return output;
}

/**
 * Mirrors `fn compress` in rust/src/sequences/compression.rs: a
 * `CompressionResult` `{original, sequence, steps}` with `isLossless(store)`.
 */
export function compress(store, sequence) {
  const original = sequence.slice();
  let current = original.slice();
  const steps = [];
  for (let best = mostFrequentPair(current); best; best = mostFrequentPair(current)) {
    const replacement = store.getOrCreate(best.pair[0], best.pair[1]);
    current = replacePair(current, best.pair, replacement);
    steps.push({ source: best.pair[0], target: best.pair[1], replacement, occurrences: best.count });
  }
  return {
    original,
    sequence: current,
    steps,
    /** Mirrors `CompressionResult::is_lossless`. */
    isLossless(target) {
      const expanded = this.sequence.flatMap((link) => target.expand(link));
      return expanded.length === this.original.length && expanded.every((link, index) => link === this.original[index]);
    },
  };
}
