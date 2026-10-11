// The grounded whole-system self-explanation (rust/src/self_explanation.rs).
//
// An exact port: every source citation resolves its `content_id` from the
// owned manifest (js/agentic/crate/self_source_links.mjs, the twin of build.rs
// `OWNED_SOURCE_FILES`), and the header carries the manifest's content id. The
// section statements and the question live in data/meta/agentic-messages.lino
// (`self_explanation_*`).

import { agenticMessage } from '../messages.mjs';
import { fastStableId, ownedFileCount, ownedManifest, ownedManifestContentId, quote } from './self_source_links.mjs';

/**
 * Mirrors `Citation::source`: a source citation grounded against the owned
 * manifest. Throws (the Rust `panic!`) when `path` is not an owned file.
 * @param {string} path
 */
export function sourceCitation(path) {
  const digest = ownedManifest().find((candidate) => candidate.path === path);
  if (!digest) throw new Error(`self_explanation_unowned_source: ${path}`);
  return { kind: 'source', path, content_id: digest.content_id };
}

/** Mirrors `Citation::data`. @param {string} path */
export function dataCitation(path) {
  return { kind: 'data', path, content_id: null };
}

/** Mirrors `Citation::test`. @param {string} path */
export function testCitation(path) {
  return { kind: 'test', path, content_id: null };
}

/** Mirrors `ExplanationSection::new`; the statement is the `self_explanation_<topic>` message. */
function section(topic, citations) {
  return { topic, statement: agenticMessage(`self_explanation_${topic}`), citations };
}

/** Mirrors `struct SystemExplanation` in rust/src/self_explanation.rs. */
export class SystemExplanation {
  /** @param {Array<object>} sections */
  constructor(sections) {
    this.sections = sections;
  }

  /** Mirrors `SystemExplanation::canonical`. */
  static canonical() {
    return new SystemExplanation([
      section('deterministic_meta_algorithm', [
        sourceCitation('src/agentic_coding/planner.rs'),
        sourceCitation('src/agentic_coding/mod.rs'),
      ]),
      section('source_to_links_round_trip', [
        sourceCitation('src/self_source_links.rs'),
        sourceCitation('src/agentic_coding/self_ast.rs'),
        testCitation('rust/tests/unit/agentic-coding/issue_558_source_links.rs'),
      ]),
      section('self_healing_loop', [
        sourceCitation('src/self_healing.rs'),
        dataCitation('data/meta/self-healing-case.lino'),
        testCitation('rust/tests/unit/agentic-coding/issue_558_self_healing.rs'),
      ]),
      section('human_gated_promotion_ledger', [
        sourceCitation('src/learning_ledger.rs'),
        dataCitation('data/meta/learning-ledger.lino'),
        testCitation('rust/tests/unit/agentic-coding/issue_558_learning_ledger.rs'),
      ]),
      section('agentic_interface', [
        sourceCitation('src/agentic_coding/driver.rs'),
        testCitation('rust/tests/integration/issue_558_learning_ledger.rs'),
      ]),
    ]);
  }

  /** Mirrors `SystemExplanation::section_count`. */
  sectionCount() {
    return this.sections.length;
  }

  /** Mirrors `SystemExplanation::citations`. */
  citations() {
    return this.sections.flatMap((entry) => entry.citations);
  }

  /** Mirrors `SystemExplanation::citation_count`. */
  citationCount() {
    return this.citations().length;
  }

  /** Mirrors `SystemExplanation::citations_of`. @param {'source'|'data'|'test'} kind */
  citationsOf(kind) {
    return this.citations().filter((citation) => citation.kind === kind);
  }

  /** Mirrors `SystemExplanation::links_notation`. */
  linksNotation() {
    const out = [
      'system_explanation',
      '  engine meta_language',
      `  question "${agenticMessage('self_explanation_question')}"`,
      `  source_file_count ${ownedFileCount()}`,
      `  source_manifest_content_id "${ownedManifestContentId()}"`,
      `  section_count ${this.sectionCount()}`,
      `  citation_count ${this.citationCount()}`,
      '  sections',
    ];
    for (const entry of this.sections) {
      out.push('    section', `      topic "${quote(entry.topic)}"`, `      statement "${quote(entry.statement)}"`,
        '      citations');
      for (const citation of entry.citations) {
        out.push('        citation', `          kind ${citation.kind}`, `          path "${quote(citation.path)}"`);
        if (citation.content_id !== null) out.push(`          content_id "${quote(citation.content_id)}"`);
      }
    }
    return out.join('\n');
  }

  /** Mirrors `SystemExplanation::content_id`. */
  contentId() {
    return fastStableId('system_explanation', this.linksNotation());
  }
}

/** Mirrors `fn canonical_explanation`. */
export function canonicalExplanation() {
  return SystemExplanation.canonical();
}
