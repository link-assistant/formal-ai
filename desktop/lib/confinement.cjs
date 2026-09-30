"use strict";

// Spec-driven path confinement for the desktop tool router (issue #953).
//
// The hand-rolled `path.relative(...) + startsWith("..")` checks that used
// to live inline in tool-router.cjs are replaced by this module, whose
// every rule is read from desktop/lib/tool-router-confinement.lino — the
// same declarative spec the Rust core's /v1/tools/authorize endpoint is
// pinned to. The module stays pure and injectable (fs access comes in as
// an option) so the adversarial suite in
// rust/tests/web/issue-953-tool-router-confinement.test.mjs can drive it
// without a live desktop.
//
// Hardening beyond the naive relative check, each answering a deny line
// from the spec:
//   * literal ".." segments are rejected before any resolution
//   * Windows UNC prefixes (\\) are refused outright
//   * Windows reserved segments (trailing dots or spaces) are refused
//   * symlink escape: when a realpath resolver is injected, every existing
//     ancestor of the requested path is resolved first and the resolved
//     path must still confine — a symlink pointing outside the root is a
//     sandbox escape, not a technicality.

const path = require("node:path");
const fs = require("node:fs");

const SPEC_PATH = path.join(__dirname, "tool-router-confinement.lino");

/** Parse the spec into { rule: { field: [...], deny: { token: reason } } }. */
function parseSpec(text) {
  const rules = {};
  let currentRule = null;
  let currentDenyToken = null;
  for (const rawLine of String(text || "").split(/\r?\n/)) {
    const line = rawLine.replace(/\t/g, "  ");
    if (!line.trim() || line.trim().startsWith("#")) continue;
    const depth = line.length - line.trimStart().length;
    const trimmed = line.trim();
    if (depth === 2 && trimmed.startsWith("rule ")) {
      currentRule = trimmed.slice(5).trim();
      rules[currentRule] = { field: {}, deny: {} };
      currentDenyToken = null;
      continue;
    }
    if (!currentRule) continue;
    const quoted = trimmed.match(/^(field|deny)\s+"([^"]*)"\s+"(.+)"$/);
    if (quoted) {
      const [, kind, token, reason] = quoted;
      if (kind === "field") rules[currentRule].field[token] = reason;
      else rules[currentRule].deny[token] = reason;
      currentDenyToken = null;
      continue;
    }
    const denyPair = trimmed.match(/^deny\s+(\S+)\s+"(.+)"$/);
    if (denyPair) {
      const [, token, reason] = denyPair;
      rules[currentRule].deny[token] = reason;
      currentDenyToken = token;
      continue;
    }
    const reasonOnly = trimmed.match(/^"(.+)"$/);
    if (reasonOnly && currentDenyToken) {
      rules[currentRule].deny[currentDenyToken] = reasonOnly[1];
      continue;
    }
    const fieldOnly = trimmed.match(/^field\s+(\S+)\s+"(.+)"$/);
    if (fieldOnly) {
      const [, token, reason] = fieldOnly;
      rules[currentRule].field[token] = reason;
    }
  }
  return rules;
}

function loadSpec(specText) {
  if (typeof specText === "string") return parseSpec(specText);
  try {
    return parseSpec(fs.readFileSync(SPEC_PATH, "utf8"));
  } catch (_error) {
    // A missing spec must fail closed: with no rules to apply, every
    // confined request is refused rather than guessed at.
    return {};
  }
}

function denial(rules, rule, token) {
  const reason = rules[rule] && rules[rule].deny[token];
  return { error: reason || `confinement rule ${rule}:${token} denied` };
}

/** Shared lexical hardening: "..", UNC, and reserved Windows segments. */
function lexicalDenial(rules, value) {
  const segments = String(value || "").split(/[\\/]+/).filter(Boolean);
  if (segments.includes("..")) return denial(rules, "dot_segments", "parent_segment");
  if (/^[\\/]{2}/.test(String(value || ""))) {
    return denial(rules, "windows_hardening", "unc_prefix");
  }
  for (const segment of segments) {
    if (/[. ]$/.test(segment) && segment !== "." && segment !== "..") {
      return denial(rules, "windows_hardening", "reserved_segment");
    }
  }
  return null;
}

/**
 * Resolve symlinks for every existing ancestor of `requested` and return
 * the fully resolved path. When `realpath` is not injected (unit tests,
 * or hosts without the target existing yet) the lexical path is used
 * as-is and the lexical rules above remain the guard.
 */
async function resolveSymlinks(realpath, requested) {
  if (typeof realpath !== "function") return String(requested);
  let resolved = String(requested);
  try {
    resolved = await realpath(String(requested));
  } catch (_error) {
    // The leaf may not exist yet (a write target). Resolve the deepest
    // existing ancestor and re-append the missing tail.
    const segments = String(requested).split(path.sep);
    const tail = [];
    while (segments.length > 1) {
      tail.unshift(segments.pop());
      try {
        resolved = await realpath(segments.join(path.sep));
        resolved = path.join(resolved, ...tail);
        break;
      } catch (_ancestorError) {
        // keep walking up
      }
    }
  }
  return resolved;
}

/** Confine `requested` under `root`. Returns { requested } or { error }. */
function confineToRoot(rules, root, requested, realpath) {
  const lexical = lexicalDenial(rules, requested);
  if (lexical) return lexical;
  if (!String(requested || "")) return denial(rules, "read_root", "empty_path");
  if (root) {
    const relative = path.relative(String(root), String(requested));
    if (relative.startsWith("..") || path.isAbsolute(relative)) {
      return denial(rules, "read_root", "escapes_root");
    }
  }
  return { requested: String(requested), resolvedBy: realpath || null };
}

/**
 * Async variant used at effect time: resolves symlinked ancestors first,
 * then confines the resolved path. A symlink that points outside the root
 * is denied with the spec's symlink_escape reason.
 */
async function confineToRootResolved(rules, root, requested, realpath) {
  const base = confineToRoot(rules, root, requested);
  if (base.error) return base;
  if (typeof realpath !== "function") return base;
  const resolved = await resolveSymlinks(realpath, String(requested));
  if (root) {
    const relative = path.relative(String(root), resolved);
    if (relative.startsWith("..") || path.isAbsolute(relative)) {
      return denial(rules, "read_root", "symlink_escape");
    }
  }
  return { requested: String(requested), resolved };
}

/** plan_id grammar from the spec: [A-Za-z0-9][A-Za-z0-9._-]{0,127}. */
function safePlanId(rules, planId) {
  const value = String(planId || "");
  if (
    value.length > 128
    || !/^[A-Za-z0-9][A-Za-z0-9._-]*$/.test(value)
    || value === "."
    || value === ".."
  ) {
    return denial(
      rules,
      "computer_use_workspace",
      value.length > 128 ? "plan_id_too_long" : "plan_id_unsafe",
    );
  }
  return { planId: value };
}

/** Confine `relative` inside the plan workspace (sync lexical checks). */
function confineToWorkspace(rules, computerUseRoot, input, value) {
  if (!computerUseRoot) {
    return { error: "computer-use isolation root is not configured" };
  }
  const plan = safePlanId(rules, input && input.plan_id);
  if (plan.error) return plan;
  const relative = String(value || "");
  const lexical = lexicalDenial(rules, relative);
  if (lexical) return lexical;
  if (!relative || path.isAbsolute(relative)) {
    return denial(rules, "computer_use_workspace", "path_not_relative");
  }
  const planRoot = path.resolve(String(computerUseRoot), plan.planId);
  const requested = path.resolve(planRoot, relative);
  const confined = path.relative(planRoot, requested);
  if (confined.startsWith("..") || path.isAbsolute(confined)) {
    return denial(rules, "computer_use_workspace", "escapes_workspace");
  }
  return { requested };
}

/** Async symlink-resolved workspace confinement (used at effect time). */
async function confineToWorkspaceResolved(rules, computerUseRoot, input, value, realpath) {
  const base = confineToWorkspace(rules, computerUseRoot, input, value);
  if (base.error || typeof realpath !== "function") return base;
  const resolved = await resolveSymlinks(realpath, base.requested);
  const plan = safePlanId(rules, input && input.plan_id);
  const planRoot = path.resolve(String(computerUseRoot), plan.planId);
  const confined = path.relative(planRoot, resolved);
  if (confined.startsWith("..") || path.isAbsolute(confined)) {
    return denial(rules, "computer_use_workspace", "symlink_escape");
  }
  return { ...base, resolved };
}

/** Archive entries must land inside the requested extraction directory. */
function confineArchiveEntry(rules, computerUseRoot, input, destination, entryPath) {
  const destinationPath = confineToWorkspace(rules, computerUseRoot, input, destination);
  const entry = String(entryPath || "");
  const entryLexical = lexicalDenial(rules, entry);
  if (entryLexical) return entryLexical;
  if (!destinationPath.requested || !entry || path.isAbsolute(entry)) {
    return {
      error: destinationPath.error
        || denial(rules, "archive_entry", "entry_not_relative").error,
    };
  }
  const relative = path.join(String(destination), entry);
  const targetPath = confineToWorkspace(rules, computerUseRoot, input, relative);
  if (!targetPath.requested) return targetPath;
  const confined = path.relative(destinationPath.requested, targetPath.requested);
  if (
    confined === ""
    || confined === ".."
    || confined.startsWith(`..${path.sep}`)
    || path.isAbsolute(confined)
  ) {
    return denial(rules, "archive_entry", "entry_escapes");
  }
  return { requested: targetPath.requested, relative };
}

function createConfinement(options = {}) {
  const rules = loadSpec(options.specText);
  const realpath = options.realpath || null;
  return {
    rules,
    confineToRoot: (root, requested) => confineToRoot(rules, root, requested),
    confineToRootResolved: (root, requested) =>
      confineToRootResolved(rules, root, requested, realpath),
    confineToWorkspace: (input, value) => confineToWorkspace(rules, options.computerUseRoot, input, value),
    confineToWorkspaceResolved: (input, value) =>
      confineToWorkspaceResolved(rules, options.computerUseRoot, input, value, realpath),
    confineArchiveEntry: (input, destination, entryPath) =>
      confineArchiveEntry(rules, options.computerUseRoot, input, destination, entryPath),
    safePlanId: (planId) => safePlanId(rules, planId),
  };
}

module.exports = {
  parseSpec,
  loadSpec,
  createConfinement,
  confineToRoot,
  confineToRootResolved,
  confineToWorkspace,
  confineToWorkspaceResolved,
  confineArchiveEntry,
  safePlanId,
  SPEC_PATH,
};
