import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import {
  copyFileSync,
  mkdtempSync,
  mkdirSync,
  readFileSync,
  rmSync,
  unlinkSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import test from "node:test";

import {
  ARCHIVE_DIR,
  ARCHIVE_LINE_LIMIT,
  addPendingRelease,
  rebaseLinks,
  relativeTo,
  renderReconstruction,
} from "./issue_711_rebuild_changelog.mjs";

test("fragment map is derivable without release commit hashes", () => {
  const first = renderReconstruction(new Map(), [
    { path: "changelog.d/fix.md", version: "1.2.3", commit: "aaaa" },
  ]).map;
  const amended = renderReconstruction(new Map(), [
    { path: "changelog.d/fix.md", version: "1.2.3", commit: "bbbb" },
  ]).map;

  assert.equal(first, amended);
  assert.equal(first, "fragment\tfirst_release\nchangelog.d/fix.md\t1.2.3\n");
});

test("pending release fragments are rendered before their commit exists", () => {
  const reconstruction = {
    assignments: [],
    groups: new Map([
      ["0.1.0", {
        version: "0.1.0",
        date: "2026-01-01",
        body: "### Added\n- Initial release.",
        fragments: null,
      }],
    ]),
  };

  addPendingRelease(reconstruction, {
    version: "0.2.0",
    date: "2026-07-17",
    fragments: [{
      path: "changelog.d/20260717_fix.md",
      body: "### Fixed\n- Record the release map in the release commit.",
    }],
  });
  const result = renderReconstruction(
    reconstruction.groups,
    reconstruction.assignments,
  );

  assert.match(result.changelog, /## \[0\.2\.0] - 2026-07-17/);
  assert.equal(
    result.map,
    "fragment\tfirst_release\nchangelog.d/20260717_fix.md\t0.2.0\n",
  );
  assert.equal(result.assignments[0].commit, undefined);
});

test("release regeneration records a consumed fragment in the release commit", () => {
  const root = mkdtempSync(join(tmpdir(), "issue-738-release-"));
  const repo = join(root, "repo");
  const source = process.cwd();
  try {
    execFileSync("git", ["clone", "--shared", "--quiet", source, repo]);
    execFileSync("git", ["-C", repo, "config", "user.email", "test@example.com"]);
    execFileSync("git", ["-C", repo, "config", "user.name", "Test"]);
    copyFileSync(
      join(source, "experiments/issue_711_rebuild_changelog.mjs"),
      join(repo, "experiments/issue_711_rebuild_changelog.mjs"),
    );

    const fragment = "changelog.d/20990101_issue_738_fixture.md";
    mkdirSync(join(repo, "changelog.d"), { recursive: true });
    writeFileSync(
      join(repo, fragment),
      "### Fixed\n- Fixture consumed by the pending release.\n",
    );
    execFileSync("git", ["-C", repo, "add", fragment]);
    execFileSync("git", ["-C", repo, "commit", "--quiet", "-m", "add fixture fragment"]);
    unlinkSync(join(repo, fragment));

    execFileSync("node", [
      "experiments/issue_711_rebuild_changelog.mjs",
      "--write",
      "--ref",
      "HEAD",
      "--pending-release",
      "999.0.0",
      "--pending-date",
      "2099-01-01",
    ], { cwd: repo });

    const map = readFileSync(
      join(repo, "docs/case-studies/issue-711/fragment-release-map.tsv"),
      "utf8",
    );
    const changelog = readFileSync(join(repo, "CHANGELOG.md"), "utf8");
    assert.match(map, /^fragment\tfirst_release$/m);
    assert.match(map, /^changelog\.d\/20990101_issue_738_fixture\.md\t999\.0\.0$/m);
    assert.doesNotMatch(map, /^fragment\tfirst_release\tfirst_release_commit$/m);
    assert.match(changelog, /## \[999\.0\.0] - 2099-01-01/);
    assert.match(changelog, /Fixture consumed by the pending release/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

function releaseGroups(count, linesPerRelease) {
  const groups = new Map();
  for (let minor = 1; minor <= count; minor += 1) {
    const body = Array.from({ length: linesPerRelease }, (_, line) => `- Change ${minor}.${line}`)
      .join("\n");
    groups.set(`0.${minor}.0`, {
      version: `0.${minor}.0`,
      date: "2026-01-01",
      body: `### Added\n${body}`,
      fragments: null,
    });
  }
  return groups;
}

test("a short history keeps every release in CHANGELOG.md and writes no archive", () => {
  const result = renderReconstruction(releaseGroups(3, 5), []);
  assert.equal(result.archives.length, 0);
  assert.doesNotMatch(result.changelog, /docs\/changelog/);
  assert.match(result.changelog, /<!-- changelog-insert-here -->\n\n## \[0\.3\.0]/);
});

test("older releases roll into size-capped archives linked from CHANGELOG.md", () => {
  const result = renderReconstruction(releaseGroups(60, 100), []);
  const lines = (text) => text.split("\n").length - 1;
  assert.ok(lines(result.changelog) <= 1400, `${lines(result.changelog)} lines`);
  assert.ok(result.archives.length >= 2);
  for (const { path, content } of result.archives) {
    assert.match(path, /^docs\/changelog\/releases-from-[\d.]+\.md$/);
    // An archive is named for the oldest release it holds, its last section.
    const oldest = [...content.matchAll(/^## \[([^\]]+)\]/gmu)].at(-1)[1];
    assert.equal(path, `docs/changelog/releases-from-${oldest}.md`);
    assert.ok(lines(content) <= ARCHIVE_LINE_LIMIT, `${path}: ${lines(content)} lines`);
    assert.match(content, /\[CHANGELOG\.md]\(\.\.\/\.\.\/CHANGELOG\.md\)/);
    assert.match(result.changelog, new RegExp(`\\]\\(${path.replaceAll(".", "\\.")}\\)`));
  }
  // Every release appears exactly once across CHANGELOG.md and the archives.
  const everything = [result.changelog, ...result.archives.map(({ content }) => content)].join("\n");
  for (let minor = 1; minor <= 60; minor += 1) {
    assert.equal(everything.split(`## [0.${minor}.0] - `).length, 2, `0.${minor}.0`);
  }
  // The newest release stays in CHANGELOG.md; the oldest opens the first archive.
  assert.match(result.changelog, /<!-- changelog-insert-here -->\n\n## \[0\.60\.0]/);
  assert.match(result.archives[0].content, /^# Changelog archive: 0\.1\.0 to /);
});

test("a full archive file is not rewritten when a new release is added", () => {
  const before = renderReconstruction(releaseGroups(60, 100), []);
  const after = renderReconstruction(releaseGroups(61, 100), []);
  assert.equal(after.archives[0].content, before.archives[0].content);
});

test("archived link targets are rebased to the archive directory", () => {
  assert.equal(relativeTo(ARCHIVE_DIR, "docs/upload-memory.md"), "../upload-memory.md");
  assert.equal(relativeTo(ARCHIVE_DIR, "scripts/install.sh"), "../../scripts/install.sh");
  assert.equal(
    rebaseLinks("[a](docs/a.md#x) [b](https://example.org) [c](#top)", ARCHIVE_DIR),
    "[a](../a.md#x) [b](https://example.org) [c](#top)",
  );
});
