// Canonical indexed Merkle topology: hosts execute hash instructions without copying tree policy.
/** @typedef {{ $: 'shape', total: number, root: number }} SnapshotShape */
/** @typedef {{ $: 'pair', left: number, right: number } | { $: 'not-pair' }} SnapshotPair */
/** @param {number} count @returns {SnapshotShape} */
export function snapshotTreeShape(count) {
  let width = count;
  let total = count;
  while (width > 1) {
    width = Math.ceil(width / 2);
    total += width;
  }
  return {$: 'shape', total, root: total - 1};
}
/** @param {number} count @param {number} node @returns {SnapshotPair} */
export function snapshotTreePair(count, node) {
  let width = count;
  let offset = 0;
  let following = count;
  while (width > 1) {
    const nextWidth = Math.ceil(width / 2);
    if (node >= following && node < following + nextWidth) {
      const position = (node - following) * 2;
      const left = offset + position;
      const right = position + 1 < width ? left + 1 : left;
      return {$: 'pair', left, right};
    }
    offset = following;
    following += nextWidth;
    width = nextWidth;
  }
  return {$: 'not-pair'};
}
/** @param {number} count @param {number} selected @returns {number[]} */
export function snapshotProofRecipe(count, selected) {
  let siblings = [];
  if (selected < 0 || selected >= count) return siblings;
  let offset = 0;
  let width = count;
  let position = selected;
  while (width > 1) {
    const even = position % 2 === 0;
    const neighbour = even ? position + 1 : position - 1;
    siblings = [...siblings, offset + (neighbour < width ? neighbour : position)];
    offset += width;
    width = Math.ceil(width / 2);
    position = Math.floor(position / 2);
  }
  return siblings;
}
/** UTF-8 octets are observations; ordering decisions are shared.
 * @param {number[]} left @param {number[]} right @returns {number} */
export function snapshotByteOrder(left, right) {
  let index = 0;
  while (index < left.length && index < right.length) {
    if (left[index] < right[index]) return -1;
    if (left[index] > right[index]) return 1;
    index += 1;
  }
  return left.length < right.length ? -1 : left.length > right.length ? 1 : 0;
}

/** @param {string} file @param {string} name @param {string[]} suffixes @param {string[]} languages @returns {string} */
export function snapshotSourceLanguage(file, name, suffixes, languages) {
  if (name === 'Cargo.toml' || name === 'Cargo.lock') return 'rust';
  const manifests = ['package.json', 'package-lock.json', 'npm-shrinkwrap.json', 'yarn.lock', 'pnpm-lock.yaml', 'bun.lock', 'bun.lockb'];
  const scripts = ['.js', '.mjs', '.cjs', '.ts', '.mts', '.cts', '.jsx', '.tsx'];
  for (let index = 0; index < manifests.length; index += 1) {
    if (manifests[index] === name) return 'javascript';
  }
  for (let index = 0; index < scripts.length; index += 1) {
    if (file.endsWith(scripts[index])) return 'javascript';
  }
  for (let index = 0; index < suffixes.length; index += 1) {
    if (file.endsWith(suffixes[index])) return languages[index];
  }
  return 'other';
}
/** @param {string} file @param {string[]} components @returns {boolean} */
export function snapshotPathEligible(file, components) {
  if (file === '' || file.includes('\0') || file.includes('\\') || file.startsWith('/')) return false;
  for (let index = 0; index < components.length; index += 1) {
    if (components[index] === '' || components[index] === '.' || components[index] === '..') return false;
  }
  return true;
}
/** @param {string} requested @returns {string} */
export function snapshotBackend(requested) {
  if (requested === 'cpu' || requested === 'auto') return 'cpu';
  return requested === 'gpu' ? 'gpu-unavailable' : 'invalid-backend';
}

/** @typedef {{ $: 'verified' } | { $: 'invalid' } | { $: 'hash', leftSibling: boolean, nextIndex: number, nextCount: number, nextCursor: number }} SnapshotVerification */
/** Host observations contain only byte equality; the shared policy owns proof acceptance.
 * @param {number} count @param {number} index @param {number} cursor @param {number} provided
 * @param {boolean} siblingMatchesCurrent @param {boolean} rootMatches @returns {SnapshotVerification} */
export function snapshotVerificationStep(count, index, cursor, provided, siblingMatchesCurrent, rootMatches) {
  if (count < 1 || index < 0 || index >= count) return {$: 'invalid'};
  if (count === 1) {
    if (cursor === provided && rootMatches) return {$: 'verified'};
    return {$: 'invalid'};
  }
  if (cursor >= provided) return {$: 'invalid'};
  const leftSibling = index % 2 !== 0;
  if (!leftSibling && index + 1 === count && !siblingMatchesCurrent) return {$: 'invalid'};
  return {$: 'hash', leftSibling, nextIndex: Math.floor(index / 2), nextCount: Math.ceil(count / 2), nextCursor: cursor + 1};
}
