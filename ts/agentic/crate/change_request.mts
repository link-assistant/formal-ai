// The reviewable change request a user's request to change Formal AI itself
// becomes (rust/src/change_request.rs): a derived requirement, a proposed
// test and a patch plan against a grounded module of the owned manifest.
//
// The owned manifest's `content_id` of a module is
// `stable_id("source_module", <file text>)`; the port reads that text from the
// repository (`rust/<path>`) through the host, the twin of build.rs's
// `include_str!` of every owned file.

import { cached, readText } from '../host.mjs';
import { agenticMessage } from '../messages.mjs';
import { stableId } from './engine_stable_id.mjs';
import { splitWhitespace, trim, trimEnd, trimEndMatches, trimMatches, utf8Len } from './rust_str.mjs';
import { byteSlice } from './formalization_segment.mjs';

/** The canonical change's grounded target (Rust `canonical_change_request`). */
export const CANONICAL_TARGET_MODULE = 'src/agentic_coding/planner.rs';

/**
 * The owned manifest's `content_id` of `path` (`SourceModuleDigest::content_id`),
 * or null when the host cannot read the embedded source.
 * @param {string} path
 */
export function ownedContentId(path) {
  return cached(`owned-content-id:${path}`, () => {
    try {
      return stableId('source_module', readText(`rust/${path}`));
    } catch {
      return null;
    }
  });
}

/** Mirrors `ChangeRequest::for_module`. */
export function changeRequestForModule(request, targetModule) {
  const trimmed = trim(request);
  if (!trimmed) throw new Error('change_request_empty');
  const contentId = ownedContentId(targetModule) ?? '';
  const derivedRequirement = deriveRequirement(trimmed);
  const proposedTest = deriveTestName(trimmed);
  return {
    id: stableId('change_request', `${trimmed}:${targetModule}:${contentId}`),
    request: trimmed,
    target_module: targetModule,
    target_content_id: contentId,
    derived_requirement: derivedRequirement,
    proposed_test: proposedTest,
    patch_plan: patchPlan(targetModule, proposedTest, derivedRequirement),
  };
}

/** Mirrors `ChangeRequest::links_notation`. */
export function changeRequestLinksNotation(change) {
  let out = 'change_request\n';
  out += field('id', change.id);
  out += field('request', change.request);
  out += field('human_gated', 'true');
  out += field('target_module', change.target_module);
  out += field('target_content_id', change.target_content_id);
  out += field('derived_requirement', change.derived_requirement);
  out += field('proposed_test', change.proposed_test);
  out += '  reviewable_pull_request\n';
  for (const step of change.patch_plan) out += `    step "${quote(step)}"\n`;
  return trimEnd(out);
}

/**
 * Mirrors `ChangeRequest::review` for a green gate and a granted approval:
 * the `AcceptedChange` it returns.
 */
export function acceptedChange(change, reviewer, suite, passed) {
  return {
    change_id: change.id,
    request: change.request,
    target_module: change.target_module,
    requirement: change.derived_requirement,
    test: change.proposed_test,
    benchmark_suite: suite,
    benchmark_passed: passed,
    reviewer,
  };
}

/** Mirrors `fn canonical_change_request`. */
export function canonicalChangeRequest() {
  return changeRequestForModule(agenticMessage('change_request_canonical_request'), CANONICAL_TARGET_MODULE);
}

/** Mirrors `fn derive_requirement`. */
function deriveRequirement(request) {
  let core = splitWhitespace(request).join(' ');
  for (const prefix of agenticMessage('change_request_requirement_prefixes').split('|')) {
    const lowered = core.toLowerCase();
    if (lowered.startsWith(prefix)) {
      const stripped = lowered.slice(prefix.length);
      const total = utf8Len(core);
      core = byteSlice(core, total - utf8Len(stripped), total);
      break;
    }
  }
  const cleaned = trim(trimEndMatches(trim(core), (character) => '.!?'.includes(character)));
  return agenticMessage('change_request_requirement', { core: decapitalize(cleaned) });
}

/** Mirrors `fn derive_test_name`. */
function deriveTestName(request) {
  let slug = '';
  let lastWasSep = true;
  for (const character of request) {
    if (/^[0-9A-Za-z]$/.test(character)) {
      slug += character.toLowerCase();
      lastWasSep = false;
    } else if (!lastWasSep) {
      slug += '_';
      lastWasSep = true;
    }
  }
  const words = trimMatches(slug, (character) => character === '_').split('_').slice(0, 8);
  return `user_requested_change_${words.join('_')}`;
}

/** Mirrors `fn patch_plan`. */
function patchPlan(targetModule, test, requirement) {
  return [
    agenticMessage('change_request_step_record', { requirement }),
    agenticMessage('change_request_step_test', { test }),
    agenticMessage('change_request_step_edit', { target_module: targetModule }),
    agenticMessage('change_request_step_gate'),
    agenticMessage('change_request_step_review'),
  ];
}

/** Mirrors `fn decapitalize`. */
function decapitalize(value) {
  const characters = Array.from(value);
  if (!characters.length) return '';
  return characters[0].toLowerCase() + characters.slice(1).join('');
}

/** Mirrors `fn quote` in rust/src/change_request.rs (and rebuild_plan.rs). */
export function quote(value) {
  return value.replaceAll('\\', '\\\\').replaceAll('"', "'").replaceAll('\n', '\\n').replaceAll('\r', '\\r').replaceAll('\t', '\\t');
}

const field = (key, value) => `  ${key} "${quote(value)}"\n`;
