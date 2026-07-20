import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);
const {
  createDataMigration,
  DATA_VERSION,
  PINNED_APP_NAME,
  VERSION_STAMP_FILE,
  STORAGE_SUBTREES,
  STORAGE_SUBTREES_BY_VERSION,
  EXCLUDED_SUBTREES,
  subtreesAddedAfter,
} = require("../lib/data-migration.cjs");

// A minimal in-memory filesystem: `dirs` is the set of directory paths and
// `files` maps file paths to contents. It implements exactly the fs surface the
// migration uses (existsSync, mkdirSync, readFileSync, writeFileSync, cpSync) so
// the whole contract is exercised without touching a real Electron profile.
function makeFakeFs() {
  const dirs = new Set();
  const files = new Map();

  function addDir(dir) {
    let current = dir;
    while (current && !dirs.has(current)) {
      dirs.add(current);
      const parent = path.dirname(current);
      if (parent === current) break;
      current = parent;
    }
  }
  function addFile(file, content = "x") {
    addDir(path.dirname(file));
    files.set(file, content);
  }

  const fs = {
    existsSync: (p) => dirs.has(p) || files.has(p),
    mkdirSync: (p) => addDir(p),
    readFileSync: (p) => {
      if (!files.has(p)) {
        const error = new Error(`ENOENT: ${p}`);
        error.code = "ENOENT";
        throw error;
      }
      return files.get(p);
    },
    writeFileSync: (p, content) => addFile(p, content),
    cpSync: (src, dest, opts) => {
      assert.ok(opts && opts.recursive, "cpSync must be recursive");
      const prefix = `${src}${path.sep}`;
      if (dirs.has(src)) addDir(dest);
      for (const dir of [...dirs]) {
        if (dir.startsWith(prefix)) addDir(`${dest}${dir.slice(src.length)}`);
      }
      for (const [file, content] of [...files]) {
        if (file === src) addFile(dest, content);
        else if (file.startsWith(prefix))
          addFile(`${dest}${file.slice(src.length)}`, content);
      }
    },
  };

  // Seed a Chromium profile directory with the given storage subtrees, each
  // carrying a uniquely-tagged marker file so copies can be verified.
  function seedProfile(base, subtrees) {
    addDir(base);
    for (const subtree of subtrees) {
      addFile(path.join(base, subtree, "marker"), `${path.basename(base)}:${subtree}`);
    }
  }

  return { fs, dirs, files, addDir, addFile, seedProfile };
}

// A fake Electron app whose userData reflects the CURRENT name, exactly like the
// real `app.getPath('userData') === <appData>/<name>` relationship that the
// whole migration hinges on.
function makeFakeApp({ name, appData }) {
  const calls = { setName: [] };
  return {
    getName: () => name,
    setName: (next) => {
      name = next;
      calls.setName.push(next);
    },
    getPath: (key) => {
      if (key === "appData") return appData;
      if (key === "userData") return path.join(appData, name);
      throw new Error(`unexpected getPath(${key})`);
    },
    calls,
  };
}

const APP_DATA = "/home/user/.config";

test("pinAppName pins the app name to the stable productName-independent value", () => {
  const { fs } = makeFakeFs();
  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  assert.deepEqual(app.calls.setName, [PINNED_APP_NAME]);
  // After pinning, userData resolves to the stable directory regardless of the
  // old productName.
  assert.equal(app.getPath("userData"), path.join(APP_DATA, PINNED_APP_NAME));
});

test("legacy candidates include the pre-pin default name plus known historical names", () => {
  const { fs } = makeFakeFs();
  const app = makeFakeApp({ name: "Custom Old Name", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  const names = migration.legacyCandidateNames();
  // The pre-pin name is captured FIRST so we migrate from wherever the data was.
  assert.equal(names[0], "Custom Old Name");
  assert.ok(names.includes("formal-ai Desktop"));
  assert.ok(names.includes("formal-ai-desktop"));
});

test("fresh install with no legacy profile copies nothing and stamps the version", () => {
  const { fs, files } = makeFakeFs();
  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 1234 });
  migration.pinAppName();
  const result = migration.migrate();

  assert.equal(result.migrated, false);
  assert.equal(result.reason, "no-legacy-data");
  const stampPath = path.join(APP_DATA, PINNED_APP_NAME, VERSION_STAMP_FILE);
  assert.ok(files.has(stampPath), "version stamp must be written");
  const stamp = JSON.parse(files.get(stampPath));
  assert.equal(stamp.version, DATA_VERSION);
  assert.equal(stamp.migratedFrom, null);
});

test("upgrading user: legacy conversations are copied into the pinned profile and the legacy copy is preserved", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  // The old profile holds the user's conversations (IndexedDB) and prefs.
  seedProfile(legacyDir, ["IndexedDB", "Local Storage", "Session Storage"]);

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  assert.equal(result.migrated, true);
  assert.equal(result.reason, "copied-legacy");
  assert.deepEqual(result.copied, ["IndexedDB", "Local Storage", "Session Storage"]);

  // The conversations now live in the pinned profile...
  assert.equal(
    files.get(path.join(pinnedDir, "IndexedDB", "marker")),
    "formal-ai Desktop:IndexedDB",
  );
  assert.equal(
    files.get(path.join(pinnedDir, "Local Storage", "marker")),
    "formal-ai Desktop:Local Storage",
  );
  // ...and the legacy copy is NEVER deleted (non-destructive).
  assert.ok(files.has(path.join(legacyDir, "IndexedDB", "marker")));

  const stamp = JSON.parse(
    files.get(path.join(pinnedDir, VERSION_STAMP_FILE)),
  );
  assert.equal(stamp.migratedFrom, "formal-ai Desktop");
});

test("never overwrites a pinned profile that already holds data", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  seedProfile(legacyDir, ["IndexedDB", "Local Storage"]);
  // The pinned profile already has the user's CURRENT conversations.
  fs.writeFileSync(path.join(pinnedDir, "IndexedDB", "marker"), "current-data");

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  assert.equal(result.migrated, false);
  assert.equal(result.reason, "pinned-already-populated");
  // The current data must be untouched — never clobbered by the legacy copy.
  assert.equal(
    files.get(path.join(pinnedDir, "IndexedDB", "marker")),
    "current-data",
  );
});

test("per-subtree guard: only the missing subtrees are copied, present ones are left intact", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  seedProfile(legacyDir, ["IndexedDB", "Local Storage", "Session Storage"]);
  // The pinned profile has a Session Storage (transient, does NOT count as
  // "storage") but no IndexedDB/Local Storage, so migration still proceeds and
  // must copy only the two missing subtrees, leaving the existing one alone.
  fs.writeFileSync(
    path.join(pinnedDir, "Session Storage", "marker"),
    "pinned-session",
  );

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  assert.equal(result.migrated, true);
  assert.deepEqual(result.copied, ["IndexedDB", "Local Storage"]);
  // The pre-existing Session Storage was not overwritten.
  assert.equal(
    files.get(path.join(pinnedDir, "Session Storage", "marker")),
    "pinned-session",
  );
});

test("idempotent: a profile already stamped at the current version is left untouched", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  seedProfile(legacyDir, ["IndexedDB"]);
  fs.writeFileSync(
    path.join(pinnedDir, VERSION_STAMP_FILE),
    JSON.stringify({ name: PINNED_APP_NAME, version: DATA_VERSION }),
  );

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  assert.equal(result.migrated, false);
  assert.equal(result.reason, "already-current");
  // No copy happened — the legacy IndexedDB was not pulled into the pinned dir.
  assert.ok(!files.has(path.join(pinnedDir, "IndexedDB", "marker")));
});

// ---------------------------------------------------------------------------
// Issue #672 (F1..F5 follow-ups), item F2: "Migration replay UI for partial
// profile transfers". v1 carried only the three canonical storage subtrees, so
// a user whose session was OAuth-shaped (auth state in Chromium's Cookies /
// Service Worker stores) landed in the pinned profile logged out, with no way
// to ask for another pass. v2 widens the copied set and adds a replay entry
// point; the tests below pin both halves of that contract.
// ---------------------------------------------------------------------------

test("v2 copies the authentication state Chromium keeps outside the canonical subtrees", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  seedProfile(legacyDir, [
    "IndexedDB",
    "Local Storage",
    "Session Storage",
    "Cookies",
    "Service Worker",
    "WebStorage",
    "WebSocketStorage",
  ]);

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  assert.equal(result.migrated, true);
  // Exactly the v1 + v2 sets, in version order — no more, no less.
  assert.deepEqual(result.copied, [...STORAGE_SUBTREES]);
  // The user stays logged in because the cookie jar came along.
  assert.equal(
    files.get(path.join(pinnedDir, "Cookies", "marker")),
    "formal-ai Desktop:Cookies",
  );
  assert.equal(
    files.get(path.join(pinnedDir, "Service Worker", "marker")),
    "formal-ai Desktop:Service Worker",
  );
});

test("derived caches are deliberately excluded from the copied set", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  seedProfile(legacyDir, ["IndexedDB", ...EXCLUDED_SUBTREES]);

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  for (const excluded of EXCLUDED_SUBTREES) {
    assert.ok(
      !STORAGE_SUBTREES.includes(excluded),
      `${excluded} must not be in the copied set`,
    );
    assert.ok(
      !files.has(path.join(pinnedDir, excluded, "marker")),
      `${excluded} must not be copied — it is derived, and a stale copy is worse than a rebuild`,
    );
  }
  assert.deepEqual(result.copied, ["IndexedDB"]);
});

test("v1 → v2 top-up: an already-migrated profile still receives the newly covered subtrees", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  seedProfile(legacyDir, [
    "IndexedDB",
    "Local Storage",
    "Session Storage",
    "Cookies",
    "Service Worker",
  ]);
  // The state a v1 build left behind: the canonical subtrees copied and stamped.
  seedProfile(pinnedDir, ["IndexedDB", "Local Storage", "Session Storage"]);
  fs.writeFileSync(
    path.join(pinnedDir, VERSION_STAMP_FILE),
    JSON.stringify({
      name: PINNED_APP_NAME,
      version: 1,
      migratedFrom: "formal-ai Desktop",
    }),
  );

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  // Without the version-aware branch this profile would hit the
  // "pinned-already-populated" guard (it legitimately HAS IndexedDB) and the
  // auth state would never arrive.
  assert.equal(result.reason, "upgraded");
  assert.equal(result.fromVersion, 1);
  assert.equal(result.migrated, true);
  // Only the v2 additions are touched, and only those the legacy profile
  // actually has (this fixture has no Web[Socket]Storage — Chromium only
  // creates those under some layouts).
  assert.deepEqual(result.copied, ["Cookies", "Service Worker"]);
  for (const subtree of result.copied) {
    assert.ok(
      subtreesAddedAfter(1).includes(subtree),
      `${subtree} must be a v2 addition, not a v1 re-copy`,
    );
  }
  assert.equal(
    files.get(path.join(pinnedDir, "Cookies", "marker")),
    "formal-ai Desktop:Cookies",
  );
  // The v1 data the user has been using since is untouched.
  assert.equal(
    files.get(path.join(pinnedDir, "IndexedDB", "marker")),
    `${PINNED_APP_NAME}:IndexedDB`,
  );
  const stamp = JSON.parse(files.get(path.join(pinnedDir, VERSION_STAMP_FILE)));
  assert.equal(stamp.version, DATA_VERSION);
});

test("subtreesAddedAfter reports only the entries a later version introduced", () => {
  assert.deepEqual(subtreesAddedAfter(0), [...STORAGE_SUBTREES]);
  assert.deepEqual(subtreesAddedAfter(1), [...STORAGE_SUBTREES_BY_VERSION[2]]);
  assert.deepEqual(subtreesAddedAfter(DATA_VERSION), []);
});

test("replay: force re-runs the copy for a stamped profile and fills only the gaps", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  seedProfile(legacyDir, ["IndexedDB", "Local Storage", "Cookies"]);
  seedProfile(pinnedDir, ["IndexedDB"]);
  // Current-version stamp: the normal path would return "already-current".
  fs.writeFileSync(
    path.join(pinnedDir, VERSION_STAMP_FILE),
    JSON.stringify({ name: PINNED_APP_NAME, version: DATA_VERSION }),
  );

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();

  assert.equal(migration.migrate().reason, "already-current");

  const replay = migration.migrate({ force: true });
  assert.equal(replay.reason, "replayed");
  assert.deepEqual(replay.copied, ["Local Storage", "Cookies"]);
  // The gap is filled...
  assert.equal(
    files.get(path.join(pinnedDir, "Cookies", "marker")),
    "formal-ai Desktop:Cookies",
  );
  // ...and the data the user already has is never clobbered by the replay.
  assert.equal(
    files.get(path.join(pinnedDir, "IndexedDB", "marker")),
    `${PINNED_APP_NAME}:IndexedDB`,
  );
});

test("replay is safe when there is nothing left to copy", () => {
  const { fs, seedProfile } = makeFakeFs();
  const legacyDir = path.join(APP_DATA, "formal-ai Desktop");
  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  seedProfile(legacyDir, ["IndexedDB"]);
  seedProfile(pinnedDir, ["IndexedDB"]);

  const app = makeFakeApp({ name: "formal-ai Desktop", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();

  const replay = migration.migrate({ force: true });
  assert.equal(replay.reason, "replayed");
  assert.deepEqual(replay.copied, []);
  assert.equal(replay.migrated, false);
  assert.equal(replay.dataVersion, DATA_VERSION);
});

test("migrates from a non-standard pre-pin name captured via app.getName()", () => {
  const { fs, files, seedProfile } = makeFakeFs();
  // A build whose name was neither productName nor the package name.
  const legacyDir = path.join(APP_DATA, "Custom Old Name");
  seedProfile(legacyDir, ["IndexedDB", "Local Storage"]);

  const app = makeFakeApp({ name: "Custom Old Name", appData: APP_DATA });
  const migration = createDataMigration({ app, fs, now: () => 0 });
  migration.pinAppName();
  const result = migration.migrate();

  const pinnedDir = path.join(APP_DATA, PINNED_APP_NAME);
  assert.equal(result.migrated, true);
  assert.equal(result.migratedFrom, legacyDir);
  assert.ok(files.has(path.join(pinnedDir, "IndexedDB", "marker")));
});
