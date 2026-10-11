// Actual Git and source ports for repository observations (PR #1188).
// Native counterpart: rust/src/repository_workspace/operation.rs::test_targets.
import { execFileSync } from "node:child_process";
import { readFileSync, realpathSync } from "node:fs";
import { resolve, relative } from "node:path";
import { createHash } from "node:crypto";
import { cargoTestTargets } from "../../scripts/lib/native-test-module-graph.mjs";
import { selectCargoManifest } from "./observation.mjs";
const sha = (bytes) => createHash("sha256").update(bytes).digest("hex");

/** Every returned path/hash is from the selected Git checkout and exact actual bytes. */
export function observeCargoTestTargets(root, expectedCommit) {
  const actualRoot = realpathSync(root);
  const git = (args) =>
    execFileSync("git", args, { cwd: actualRoot, maxBuffer: 16 * 1024 * 1024 });
  const head = git(["rev-parse", "HEAD"]).toString("utf8").trim();
  if (!/^[0-9a-f]{40}$/u.test(expectedCommit) || head !== expectedCommit) {
    throw new Error("workspace base commit mismatch");
  }
  const tracked = git(["ls-files", "-z"])
    .toString("utf8")
    .split("\0")
    .filter(Boolean);
  const manifest = selectCargoManifest(tracked);
  const targets = cargoTestTargets(actualRoot, manifest);
  const paths = [
    manifest,
    ...targets.map((target) =>
      relative(actualRoot, target.path).replaceAll("\\", "/"),
    ),
  ];
  const sources = paths.map((path) => {
    const absolute = realpathSync(resolve(actualRoot, path));
    const owned = relative(actualRoot, absolute);
    if (owned === ".." || owned.startsWith("../") || owned.startsWith("..\\")) {
      throw new Error("test target leaves owned checkout");
    }
    const bytes = readFileSync(absolute);
    return { path, sha256: sha(bytes), byte_length: bytes.length };
  });
  if (git(["rev-parse", "HEAD"]).toString("utf8").trim() !== expectedCommit) {
    throw new Error("source context changed during query");
  }
  return {
    schema: "repository-query/v1",
    query: "cargo-test-targets",
    root: actualRoot,
    base_commit: head,
    sources,
  };
}
