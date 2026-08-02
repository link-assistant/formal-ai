"use strict";

// Desktop data persistence & migration (issue #541, R3): make sure a user's
// conversations survive across app versions. Two real-world failure modes
// silently deleted "previous desktop conversations":
//
//   1. The userData directory name was never pinned. Electron derives it from
//      the app name (`app.getName()`), which is itself derived from
//      package.json `name`/`productName`. ANY change to that string — a rebrand,
//      a spacing/casing fix, a switch between "formal-ai Desktop" and
//      "formal-ai-desktop" — moves the whole userData directory and orphans the
//      renderer's IndexedDB ("formal-ai-demo", the conversation store) and
//      Local Storage ("formal-ai.preferences.v1"). The data was never deleted,
//      but the app now looks somewhere else and sees an empty profile.
//
//   2. No migration ever ran. Nothing copied data forward, so even a deliberate
//      directory rename was unrecoverable for the user.
//
// This module fixes both. It PINS the userData directory to a fixed,
// productName-independent name ("formal-ai") so the path never moves again, and
// on startup — before the window/session touches storage — it NON-DESTRUCTIVELY
// copies the Chromium storage subtrees (IndexedDB, Local Storage, Session
// Storage) from any legacy profile directory into the pinned one, but only when
// the pinned directory does not already have them. The legacy copy is never
// deleted, so the migration is safe to re-run and impossible to lose data to. A
// versioned `formal-ai-data-version.json` stamp records that the migration ran
// so future schema changes can transform data deterministically.
//
// Every side-effecting dependency (fs, app, clock) is injected so the whole
// contract is unit-testable without a real Electron profile on disk, matching
// the rest of desktop/lib (see docker-detect.cjs).

const nodeFs = require("node:fs");
const nodePath = require("node:path");

// Bump when the on-disk data layout changes in a way that needs a transform.
// A stamp whose version is >= this is considered already-migrated.
//
// Version 2 (issue #672, F2) widens the migrated set beyond the three canonical
// subtrees to the directories that carry *authentication* state. Version 1 only
// moved conversations and preferences, so a user upgrading from a legacy
// profile kept their history but was silently logged out of every OAuth-style
// session — the exact "partial profile transfer" F2 reported.
const DATA_VERSION = 2;

// The stable, productName-independent directory name. Once pinned, the userData
// path is `<appData>/formal-ai` on every OS and never moves with rebrands.
const PINNED_APP_NAME = "formal-ai";

// The stamp file written into the pinned directory after a migration check.
const VERSION_STAMP_FILE = "formal-ai-data-version.json";

// Chromium storage entries that hold the user's actual data, grouped by the
// DATA_VERSION that introduced them. Grouping (rather than one flat list) is
// what makes a version bump *useful*: a profile already stamped at v1 has its
// conversations, so re-running the whole set would copy nothing; the upgrade
// path below copies only the entries a newer version added.
//
//   v1 — conversations and preferences:
//     - IndexedDB        → the "formal-ai-demo" conversation/event store
//     - Local Storage    → "formal-ai.preferences.v1" (theme, demo mode, etc.)
//     - Session Storage  → transient per-tab state (harmless, copied for parity)
//
//   v2 — authentication and offline state (issue #672, F2):
//     - Cookies          → the SQLite file behind every logged-in session. This
//                          is a FILE, not a directory; `fs.cpSync` and
//                          `existsSync` handle both, so it needs no special case.
//     - Service Worker   → registrations + their scripts; without them an
//                          installed PWA-style surface re-registers from scratch.
//     - WebStorage       → Chromium's newer bucketed storage root.
//     - WebSocketStorage → the name F2 asked for. Chromium has used both this
//                          and "WebStorage" across versions; listing both is
//                          free (a missing source entry is skipped) and means
//                          the migration works on either layout.
const STORAGE_SUBTREES_BY_VERSION = Object.freeze({
  1: ["IndexedDB", "Local Storage", "Session Storage"],
  2: ["Cookies", "Service Worker", "WebStorage", "WebSocketStorage"],
});

// Deliberately NOT migrated, and this is a reconciliation of F2's original
// sketch rather than an omission (see docs/case-studies/issue-672/README.md).
// F2 listed `Cache` and `Code Cache` alongside the auth subtrees, but those two
// are pure derived caches: Chromium regenerates them on demand, they are the
// largest directories in a profile, and carrying a cache built by a *different*
// Chromium build into a fresh profile is the documented way to produce
// hard-to-diagnose corruption. Copying them cannot fix the reported symptom
// (being logged out), so the risk buys nothing. GPUCache and the lock/singleton
// files are excluded for the same reason.
const EXCLUDED_SUBTREES = Object.freeze([
  "Cache",
  "Code Cache",
  "GPUCache",
  "SingletonLock",
]);

// The full set, in version order, for a first-ever migration.
const STORAGE_SUBTREES = Object.freeze(
  Object.keys(STORAGE_SUBTREES_BY_VERSION)
    .map((version) => Number(version))
    .sort((a, b) => a - b)
    .flatMap((version) => STORAGE_SUBTREES_BY_VERSION[version]),
);

// The entries a profile stamped at `fromVersion` is still missing. Used by the
// v1 → v2 top-up so an existing installation gains its auth state without
// re-copying (or risking) anything it already has.
function subtreesAddedAfter(fromVersion) {
  const base = Number.isFinite(fromVersion) ? fromVersion : 0;
  return Object.keys(STORAGE_SUBTREES_BY_VERSION)
    .map((version) => Number(version))
    .filter((version) => version > base)
    .sort((a, b) => a - b)
    .flatMap((version) => STORAGE_SUBTREES_BY_VERSION[version]);
}

// Known historical profile directory names to migrate FROM, in priority order.
// We also prepend whatever name Electron would have used before we pinned it, so
// the migration works regardless of whether the old name came from package
// `name` or `productName`.
const KNOWN_LEGACY_NAMES = [
  "formal-ai Desktop",
  "formal-ai-desktop",
  "Formal AI",
  "formal_ai",
];

function dedupe(values) {
  const seen = new Set();
  const out = [];
  for (const value of values) {
    const trimmed = String(value || "").trim();
    if (trimmed && !seen.has(trimmed)) {
      seen.add(trimmed);
      out.push(trimmed);
    }
  }
  return out;
}

function createDataMigration(options = {}) {
  const app = options.app;
  const fs = options.fs || nodeFs;
  const path = options.path || nodePath;
  const log = typeof options.log === "function" ? options.log : () => {};
  const now = typeof options.now === "function" ? options.now : () => Date.now();
  const pinnedName =
    typeof options.pinnedName === "string" && options.pinnedName.trim()
      ? options.pinnedName.trim()
      : PINNED_APP_NAME;

  if (!app || typeof app.getPath !== "function") {
    throw new Error(
      "createDataMigration requires an Electron-like app with getPath()",
    );
  }

  // Capture the name Electron would use BEFORE we pin it, so the pre-pin profile
  // directory becomes a migration source no matter how it was derived. This must
  // be read before pinAppName() runs.
  let defaultName = "";
  if (typeof app.getName === "function") {
    try {
      defaultName = String(app.getName() || "").trim();
    } catch (_error) {
      defaultName = "";
    }
  }

  // Pin the userData directory to a stable name. Must be called before the
  // Electron `ready` event (i.e. before any window/session is created) for the
  // override to take effect. Idempotent.
  function pinAppName() {
    if (typeof app.setName !== "function") {
      log("data-migration: app.setName unavailable; cannot pin app name");
      return;
    }
    try {
      app.setName(pinnedName);
      log(`data-migration: pinned app name to "${pinnedName}"`);
    } catch (error) {
      log(
        "data-migration: failed to pin app name:",
        error && error.message ? error.message : String(error),
      );
    }
  }

  function exists(target) {
    try {
      return Boolean(fs.existsSync(target));
    } catch (_error) {
      return false;
    }
  }

  // A directory "has storage" if it carries either of the two subtrees that hold
  // real user data. Session Storage alone does not count (it is transient).
  function hasStorage(dir) {
    return (
      exists(path.join(dir, "IndexedDB")) ||
      exists(path.join(dir, "Local Storage"))
    );
  }

  // Candidate legacy profile directory names, most-likely first.
  function legacyCandidateNames() {
    return dedupe([defaultName, ...KNOWN_LEGACY_NAMES]);
  }

  function readStamp(stampPath) {
    if (!exists(stampPath)) {
      return null;
    }
    try {
      const parsed = JSON.parse(fs.readFileSync(stampPath, "utf8"));
      return parsed && typeof parsed === "object" ? parsed : null;
    } catch (_error) {
      return null;
    }
  }

  function writeStamp(stampPath, extra) {
    const payload = {
      name: pinnedName,
      version: DATA_VERSION,
      migratedAt: now(),
      ...extra,
    };
    try {
      fs.writeFileSync(stampPath, `${JSON.stringify(payload, null, 2)}\n`);
    } catch (error) {
      log(
        "data-migration: failed to write version stamp:",
        error && error.message ? error.message : String(error),
      );
    }
    return payload;
  }

  // Locate the first legacy profile directory that exists and contains real
  // storage, skipping the pinned directory itself (never migrate onto self).
  function findLegacySource(appDataDir, pinnedDir) {
    const pinnedResolved = path.resolve(pinnedDir);
    for (const name of legacyCandidateNames()) {
      const candidate = path.join(appDataDir, name);
      if (path.resolve(candidate) === pinnedResolved) {
        continue;
      }
      if (hasStorage(candidate)) {
        return candidate;
      }
    }
    return null;
  }

  function ensureDir(dir) {
    try {
      fs.mkdirSync(dir, { recursive: true });
    } catch (error) {
      log(
        "data-migration: failed to create pinned dir:",
        error && error.message ? error.message : String(error),
      );
    }
  }

  // Copy one storage subtree, but only when the destination does not already
  // have it — the migration must never clobber data already in the pinned
  // profile. Returns true if a copy happened.
  function copySubtree(sourceDir, destDir, subtree) {
    const src = path.join(sourceDir, subtree);
    const dest = path.join(destDir, subtree);
    if (!exists(src) || exists(dest)) {
      return false;
    }
    try {
      fs.cpSync(src, dest, { recursive: true });
      log(`data-migration: copied "${subtree}" from legacy profile`);
      return true;
    } catch (error) {
      log(
        `data-migration: failed to copy "${subtree}":`,
        error && error.message ? error.message : String(error),
      );
      return false;
    }
  }

  // Copy `subtrees` from the legacy profile, honouring the per-entry
  // "destination wins" guard. Shared by the first-run migration, the version
  // top-up and the user-triggered replay so all three can never drift apart.
  function copyFrom(source, pinnedDir, subtrees) {
    const copied = [];
    for (const subtree of subtrees) {
      if (copySubtree(source, pinnedDir, subtree)) {
        copied.push(subtree);
      }
    }
    return copied;
  }

  // The main entry point. Safe to call once per startup, after pinAppName() and
  // after the app is ready (so getPath resolves), but before any window/session
  // is created. Returns a summary describing what (if anything) happened.
  //
  // `options.force` (issue #672, F2) ignores the version stamp and retries every
  // entry. It backs the user-visible "replay migration" affordance: a user who
  // sees the notice and believes data is still missing can ask for another pass
  // without editing files by hand. It stays safe because the per-entry guard is
  // unchanged — a replay can only ever fill gaps, never overwrite.
  function migrate(options = {}) {
    const force = Boolean(options && options.force);
    const pinnedDir = app.getPath("userData");
    ensureDir(pinnedDir);
    const stampPath = path.join(pinnedDir, VERSION_STAMP_FILE);

    const existingStamp = readStamp(stampPath);
    const stampedVersion =
      existingStamp && Number.isFinite(existingStamp.version)
        ? existingStamp.version
        : null;

    if (!force && stampedVersion !== null && stampedVersion >= DATA_VERSION) {
      log("data-migration: already current, nothing to do");
      return {
        migrated: false,
        reason: "already-current",
        version: stampedVersion,
        dataVersion: DATA_VERSION,
        copied: [],
        migratedFrom: existingStamp.migratedFrom || null,
        pinnedDir,
      };
    }

    const appDataDir =
      typeof app.getPath === "function" ? app.getPath("appData") : "";
    const source = appDataDir
      ? findLegacySource(appDataDir, pinnedDir)
      : null;

    // A profile stamped at an OLDER version is an upgrade, not a fresh install:
    // it legitimately already holds storage, so the "already populated" guard
    // below must not short-circuit it. Copy exactly the entries the newer
    // version added (v1 → v2: the auth state) and re-stamp.
    if (!force && stampedVersion !== null && stampedVersion < DATA_VERSION) {
      const pending = subtreesAddedAfter(stampedVersion);
      const copied = source ? copyFrom(source, pinnedDir, pending) : [];
      const migratedFrom = source
        ? path.basename(source)
        : existingStamp.migratedFrom || null;
      writeStamp(stampPath, { migratedFrom, copied });
      log(
        `data-migration: upgraded v${stampedVersion} → v${DATA_VERSION}, copied ${copied.length} entry(ies)`,
      );
      return {
        migrated: copied.length > 0,
        reason: "upgraded",
        fromVersion: stampedVersion,
        version: DATA_VERSION,
        dataVersion: DATA_VERSION,
        migratedFrom: source || null,
        copied,
        pinnedDir,
      };
    }

    // Never overwrite a pinned profile that already holds data (e.g. a prior
    // pinned build, or a fresh install that already wrote conversations). Just
    // stamp it so we do not probe legacy directories on every launch.
    if (!force && hasStorage(pinnedDir)) {
      writeStamp(stampPath, { migratedFrom: null, copied: [] });
      log("data-migration: pinned profile already populated; stamped only");
      return {
        migrated: false,
        reason: "pinned-already-populated",
        version: DATA_VERSION,
        dataVersion: DATA_VERSION,
        copied: [],
        migratedFrom: null,
        pinnedDir,
      };
    }

    if (!source) {
      writeStamp(stampPath, { migratedFrom: null, copied: [] });
      log("data-migration: no legacy profile found; stamped empty profile");
      return {
        migrated: false,
        reason: "no-legacy-data",
        version: DATA_VERSION,
        dataVersion: DATA_VERSION,
        copied: [],
        migratedFrom: null,
        pinnedDir,
      };
    }

    const copied = copyFrom(source, pinnedDir, STORAGE_SUBTREES);
    const migratedFrom = path.basename(source);
    writeStamp(stampPath, { migratedFrom, copied });
    log(
      `data-migration: migrated ${copied.length} entry(ies) from "${migratedFrom}"`,
    );
    return {
      migrated: copied.length > 0,
      reason: force
        ? "replayed"
        : copied.length > 0
          ? "copied-legacy"
          : "legacy-empty",
      version: DATA_VERSION,
      dataVersion: DATA_VERSION,
      migratedFrom: source,
      copied,
      pinnedDir,
    };
  }

  return {
    pinAppName,
    migrate,
    legacyCandidateNames,
    pinnedName,
  };
}

module.exports = {
  createDataMigration,
  DATA_VERSION,
  PINNED_APP_NAME,
  VERSION_STAMP_FILE,
  STORAGE_SUBTREES,
  STORAGE_SUBTREES_BY_VERSION,
  EXCLUDED_SUBTREES,
  subtreesAddedAfter,
  KNOWN_LEGACY_NAMES,
};
