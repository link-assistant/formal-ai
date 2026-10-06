// The whole-repository source <-> links projection (rust/src/self_source_links.rs).
//
// The owned manifest is exact. Rust's build.rs (`emit_owned_source_manifest`)
// walks `rust/src` recursively with `fs::read_dir`, keeps every `.rs` file,
// sorts the paths as strings and embeds each one with `include_str!` as
// `OWNED_SOURCE_FILES` of `(path, text)` pairs. The port walks the same
// directory through the host (`listDirectory`) and reads each file with
// `readText`, so `owned_file_count`, `owned_total_bytes`, every
// `content_id` and `owned_manifest_content_id` are the Rust values for the
// checked-out tree.
//
// Substitution: `SourceModuleProjection::project` calls
// `agentic_coding::self_ast::ast_census`, a tree-sitter parse through the
// meta-language links network, which has no JavaScript twin. The port follows
// the Rust crate built without the `meta-language` feature, whose
// `#[cfg(not(feature = "meta-language"))] ast_census` returns zero links, zero
// named nodes and `text_preserved: false`; that census is `NO_ENGINE_CENSUS`
// below. The parse-derived counts therefore differ from the default (engine)
// Rust build, and no committed artifact pins them (the Rust tests assert the
// document live because it changes with every source edit).

import { cached, listDirectory, readText } from '../host.mjs';
import { utf8Len } from './rust_str.mjs';

/** The directory build.rs embeds, relative to the crate manifest (`rust/`). */
const CRATE_DIR = 'rust';
const SOURCE_DIR = 'src';

const FNV_PRIME_LOW = 0x1b3;
const encoder = new TextEncoder();

/**
 * `crate::engine::stable_id` (js/agentic/crate/engine_stable_id.mjs) over 16-bit
 * limbs instead of BigInt: the manifest hashes the whole ~10 MB source tree,
 * where the BigInt loop is seconds slower. FNV-1a 64: `prime = 2^40 + 0x1b3`,
 * so `h * prime = h * 0x1b3 + (h << 40)` modulo 2^64.
 * @param {string} prefix
 * @param {string} text
 */
export function fastStableId(prefix, text) {
  let h0 = 0x2325;
  let h1 = 0x8422;
  let h2 = 0x9ce4;
  let h3 = 0xcbf2;
  for (const byte of encoder.encode(String(text))) {
    h0 ^= byte;
    const t0 = h0 * FNV_PRIME_LOW;
    const t1 = h1 * FNV_PRIME_LOW + (t0 >>> 16);
    const t2 = h2 * FNV_PRIME_LOW + (t1 >>> 16) + ((h0 & 0xff) << 8);
    const t3 = h3 * FNV_PRIME_LOW + (t2 >>> 16) + ((h0 >>> 8) | ((h1 & 0xff) << 8));
    h0 = t0 & 0xffff;
    h1 = t1 & 0xffff;
    h2 = t2 & 0xffff;
    h3 = t3 & 0xffff;
  }
  const hex = (limb) => limb.toString(16).padStart(4, '0');
  return `${prefix}_${hex(h3)}${hex(h2)}${hex(h1)}${hex(h0)}`;
}

/**
 * Mirrors `fn enumerate_rust_files` in rust/build.rs: every `.rs` file under
 * `dir`, recursively, as crate-relative paths.
 */
function enumerateRustFiles(dir, out) {
  for (const entry of listDirectory(`${CRATE_DIR}/${dir}`)) {
    const path = `${dir}/${entry.name}`;
    if (entry.isDirectory) enumerateRustFiles(path, out);
    else if (entry.name.endsWith('.rs') && entry.name.length > 3) out.push(path);
  }
}

/** Rust `str::cmp` (byte order of UTF-8), used by build.rs's path sort. */
function byteOrder(left, right) {
  const a = encoder.encode(left);
  const b = encoder.encode(right);
  const length = Math.min(a.length, b.length);
  for (let index = 0; index < length; index += 1) {
    if (a[index] !== b[index]) return a[index] - b[index];
  }
  return a.length - b.length;
}

/**
 * Mirrors `fn owned_source_files` (build.rs `OWNED_SOURCE_FILES`): the
 * `[path, text]` pairs of every owned Rust source file, sorted by path.
 * @returns {Array<[string, string]>}
 */
export function ownedSourceFiles() {
  return cached('owned-source-files', () => {
    const paths = [];
    enumerateRustFiles(SOURCE_DIR, paths);
    paths.sort(byteOrder);
    return paths.map((path) => [path, readText(`${CRATE_DIR}/${path}`)]);
  });
}

/** Mirrors `SourceModuleDigest::of`. */
export function sourceModuleDigest(path, source) {
  return { path, byte_len: utf8Len(source), content_id: fastStableId('source_module', source) };
}

/**
 * The census of the Rust crate built without the `meta-language` feature
 * (`#[cfg(not(feature = "meta-language"))] fn ast_census`).
 */
const NO_ENGINE_CENSUS = Object.freeze({ total_link_count: 0, named_node_count: 0, text_preserved: false });

/** Mirrors `SourceModuleProjection::project` (see the header on the census). */
export function projectSourceModule(path, source) {
  const census = NO_ENGINE_CENSUS;
  return {
    path,
    byte_len: utf8Len(source),
    content_id: fastStableId('source_module', source),
    total_link_count: census.total_link_count,
    named_node_count: census.named_node_count,
    faithful: census.text_preserved,
  };
}

/** Mirrors `struct SourceLinks` in rust/src/self_source_links.rs. */
export class SourceLinks {
  /** @param {Array<object>} modules */
  constructor(modules) {
    this.modules = modules;
  }

  /** Mirrors `SourceLinks::compile`. @param {Array<[string, string]>} files */
  static compile(files) {
    return new SourceLinks(files.map(([path, source]) => projectSourceModule(path, source)));
  }

  /** Mirrors `SourceLinks::module_count`. */
  moduleCount() {
    return this.modules.length;
  }

  /** Mirrors `SourceLinks::faithful_count`. */
  faithfulCount() {
    return this.modules.filter((module) => module.faithful).length;
  }

  /** Mirrors `SourceLinks::is_fully_faithful`. */
  isFullyFaithful() {
    return this.modules.length > 0 && this.faithfulCount() === this.modules.length;
  }

  /** Mirrors `SourceLinks::coverage_permille`. */
  coveragePermille() {
    if (this.modules.length === 0) return 0;
    return Math.floor((this.faithfulCount() * 1000) / this.modules.length);
  }

  /** Mirrors `SourceLinks::total_link_count`. */
  totalLinkCount() {
    return this.modules.reduce((sum, module) => sum + module.total_link_count, 0);
  }

  /** Mirrors `SourceLinks::total_named_node_count`. */
  totalNamedNodeCount() {
    return this.modules.reduce((sum, module) => sum + module.named_node_count, 0);
  }

  /** Mirrors `SourceLinks::links_notation`. */
  linksNotation() {
    const out = [
      'source_links',
      '  engine meta_language',
      '  language rust',
      `  module_count ${this.moduleCount()}`,
      `  faithful_count ${this.faithfulCount()}`,
      `  coverage_permille ${this.coveragePermille()}`,
      `  fully_faithful ${this.isFullyFaithful()}`,
      `  total_link_count ${this.totalLinkCount()}`,
      `  total_named_node_count ${this.totalNamedNodeCount()}`,
      '  modules',
    ];
    for (const module of this.modules) {
      out.push(
        '    module',
        `      path "${quote(module.path)}"`,
        `      byte_len ${module.byte_len}`,
        `      content_id "${quote(module.content_id)}"`,
        `      total_link_count ${module.total_link_count}`,
        `      named_node_count ${module.named_node_count}`,
        `      faithful ${module.faithful}`,
      );
    }
    return out.join('\n');
  }
}

/** Mirrors `fn owned_manifest`. */
export function ownedManifest() {
  return cached('owned-manifest', () => ownedSourceFiles().map(([path, source]) => sourceModuleDigest(path, source)));
}

/** Mirrors `fn owned_file_count`. */
export function ownedFileCount() {
  return ownedSourceFiles().length;
}

/** Mirrors `fn owned_total_bytes`. */
export function ownedTotalBytes() {
  return ownedManifest().reduce((sum, digest) => sum + digest.byte_len, 0);
}

/** Mirrors `fn owned_manifest_notation`. */
export function ownedManifestNotation() {
  return cached('owned-manifest-notation', () => {
    const manifest = ownedManifest();
    const out = [
      'source_manifest',
      '  engine meta_language',
      '  language rust',
      `  file_count ${manifest.length}`,
      `  total_bytes ${ownedTotalBytes()}`,
      '  files',
    ];
    for (const digest of manifest) {
      out.push('    file', `      path "${quote(digest.path)}"`, `      byte_len ${digest.byte_len}`,
        `      content_id "${quote(digest.content_id)}"`);
    }
    return out.join('\n');
  });
}

/** Mirrors `fn owned_manifest_content_id`. */
export function ownedManifestContentId() {
  return cached('owned-manifest-content-id', () => fastStableId('source_tree', ownedManifestNotation()));
}

/** Mirrors `fn quote` in rust/src/self_source_links.rs (and self_explanation.rs). */
export function quote(value) {
  return value.replaceAll('\\', '\\\\').replaceAll('"', "'").replaceAll('\n', '\\n')
    .replaceAll('\r', '\\r').replaceAll('\t', '\\t');
}
