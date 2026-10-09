// Source-owned observation goals and actual child-process status (PR #1188).
import assert from "node:assert/strict";
import { before, test } from "node:test";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { WorkerHost } from "../../../js/server/worker-host.mjs";
import { installNodeHost } from "../../../js/agentic/node-host.mjs";
import {
  classifyRepositoryOperation,
  selectsRepositoryStage,
  selectCargoManifest,
  observedCommandReport,
} from "../../../js/repository-workspace/observation.mjs";
before(async () => {
  await installNodeHost(new WorkerHost());
});
test("original question selects source reads and the original command selects verified execution", () => {
  assert.equal(
    classifyRepositoryOperation("Where do unit tests live in this repository?")
      .kind,
    "test-targets",
  );
  const command = classifyRepositoryOperation(
    "Run cargo check on this repository and tell me whether it succeeds.",
  );
  assert.equal(command.kind, "run");
  assert.equal(command.line, "cargo check");
  assert.deepEqual(
    ["clone", "locate", "read", "edit", "verify", "diff", "commit"].filter(
      (stage) => selectsRepositoryStage(command, stage),
    ),
    ["clone", "verify"],
  );
});
test("heldout inspection grammar shares the same goal and compound authoring remains a gap", () => {
  for (const prompt of [
    "Where are regressions?",
    "Locate test definitions.",
    "Inspect tests.",
  ]) {
    assert.equal(
      classifyRepositoryOperation(prompt).kind,
      "test-targets",
      prompt,
    );
  }
  for (const prompt of [
    "Run cargo check and write a new function.",
    "Inspect tests and implement a new function.",
  ]) {
    assert.equal(
      classifyRepositoryOperation(prompt).kind,
      "unsupported",
      prompt,
    );
  }
  assert.equal(
    classifyRepositoryOperation("Replace beta with gamma in source.rs.").kind,
    "mutation",
  );
  assert.equal(
    classifyRepositoryOperation("Check the current exchange rate on the web.")
      .kind,
    "mutation",
  );
});
test("manifest discovery follows actual paths and refuses missing or ambiguous siblings", () => {
  assert.equal(
    selectCargoManifest([
      "fixture/Cargo.toml",
      "engine/Cargo.toml",
      "Cargo.toml",
    ]),
    "Cargo.toml",
  );
  assert.equal(
    selectCargoManifest(["engine/Cargo.toml", "fixtures/old/Cargo.toml"]),
    "engine/Cargo.toml",
  );
  assert.throws(() => selectCargoManifest([]), /missing or ambiguous/u);
  assert.throws(
    () => selectCargoManifest(["alpha/Cargo.toml", "beta/Cargo.toml"]),
    /ambiguous/u,
  );
});
test("Windows source paths use the same normalized manifest rule", () => {
  assert.equal(
    selectCargoManifest(["engine\\Cargo.toml", "fixtures\\old\\Cargo.toml"]),
    "engine/Cargo.toml",
  );
});
function observed(exit) {
  const child = spawnSync(
    process.execPath,
    [
      "-e",
      'process.stdout.write("λ\\r\\n");process.stderr.write("observed\\n");process.exit(' +
        exit +
        ");",
    ],
    { encoding: "utf8" },
  );
  assert.equal(child.error, undefined);
  const output = child.stdout + child.stderr;
  const argv = [process.execPath, "-e", "actual-test-child"];
  const line = "observed-test-child";
  const evidence = {
    command: line,
    argv,
    exit_code: child.status,
    observed_byte_length: Buffer.byteLength(output),
    evidence_id: "actual-child",
    observed_output_sha256: createHash("sha256").update(output).digest("hex"),
  };
  return {
    line,
    argv,
    root: process.cwd(),
    baseCommit: execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: process.cwd(),
    })
      .toString("utf8")
      .trim(),
    observedCommit: execFileSync("git", ["rev-parse", "HEAD"], {
      cwd: process.cwd(),
    })
      .toString("utf8")
      .trim(),
    observation: {
      exit_code: child.status,
      timed_out: false,
      partial_output: output,
      elapsed_seconds: 0,
      deadline_seconds: 2,
    },
    evidence,
  };
}
for (const exit of [0, 7])
  test("actual process status controls completion: " + exit, () => {
    const report = observedCommandReport(observed(exit));
    assert.equal(report.complete, exit === 0);
    assert.equal(report.exit_code, exit);
    assert.equal(report.combined_output, "λ\r\nobserved\n");
  });
test("a real child success cannot complete an unbound or changed source head", () => {
  for (const commit of [undefined, "0".repeat(40)]) {
    const input = observed(0);
    input.observedCommit = commit;
    const report = observedCommandReport(input);
    assert.equal(report.complete, false);
    assert.equal(report.observed_commit, commit ?? null);
  }
});
test("matching length cannot substitute for an exact output hash", () => {
  const input = observed(0);
  input.observation.partial_output = input.observation.partial_output.replace(
    "λ",
    "μ",
  );
  assert.throws(() => observedCommandReport(input), /unbound/u);
});
test("missing status and timeout can never be completion", () => {
  for (const timedOut of [false, true]) {
    const input = observed(0);
    input.observation.exit_code = null;
    input.evidence.exit_code = null;
    input.observation.timed_out = timedOut;
    assert.equal(observedCommandReport(input).complete, false);
  }
});
test("wrong argv, root, source identity and status remain unbound", () => {
  const mutations = [
    (input) => {
      input.argv = ["other"];
    },
    (input) => {
      input.root = "";
    },
    (input) => {
      input.baseCommit = "HEAD";
    },
    (input) => {
      input.evidence.exit_code = 7;
    },
  ];
  for (const change of mutations) {
    const input = observed(0);
    change(input);
    assert.throws(() => observedCommandReport(input), /unbound/u);
  }
});

// Real manifest paths and bytes are exercised through the Node source port.
import {
  mkdtempSync,
  mkdirSync,
  writeFileSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, dirname } from "node:path";
import { execFileSync } from "node:child_process";
import { observeCargoTestTargets } from "../../../js/repository-workspace/source.mjs";
function sourceFixture(context, prefix = "") {
  const root = mkdtempSync(join(tmpdir(), "formal-ai-cargo-query-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const path = prefix + "checks/independent.rs";
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(
    join(root, prefix + "Cargo.toml"),
    '[package]\nname="observed"\nversion="0.0.0"\n[[test]]\nname="heldout"\npath="checks/independent.rs"\n',
  );
  writeFileSync(join(root, path), "// λ\r\n#[test]\nfn heldout() {}\n");
  const git = (args) =>
    execFileSync("git", args, { cwd: root, stdio: ["ignore", "pipe", "pipe"] });
  git(["init"]);
  git(["add", "."]);
  git([
    "-c",
    "user.name=Fixture",
    "-c",
    "user.email=fixture@example.invalid",
    "commit",
    "-m",
    "Observed",
  ]);
  return {
    root,
    path,
    head: git(["rev-parse", "HEAD"]).toString("utf8").trim(),
    git,
  };
}
for (const prefix of ["", "different/"])
  test("actual Cargo query follows heldout layout " + prefix, (context) => {
    const fixture = sourceFixture(context, prefix),
      before = fixture.git(["status", "--porcelain"]);
    const report = observeCargoTestTargets(fixture.root, fixture.head);
    assert.ok(report.sources.some((source) => source.path === fixture.path));
    for (const source of report.sources) {
      const bytes = readFileSync(join(fixture.root, source.path));
      assert.equal(
        source.sha256,
        createHash("sha256").update(bytes).digest("hex"),
      );
      assert.equal(source.byte_length, bytes.length);
    }
    assert.deepEqual(fixture.git(["status", "--porcelain"]), before);
  });
test("actual query rejects wrong source head before reporting", (context) => {
  const fixture = sourceFixture(context);
  assert.throws(
    () => observeCargoTestTargets(fixture.root, "0".repeat(40)),
    /commit mismatch/u,
  );
});
test("actual query refuses a missing declared target rather than naming an invented path", (context) => {
  const fixture = sourceFixture(context);
  rmSync(join(fixture.root, fixture.path));
  assert.throws(
    () => observeCargoTestTargets(fixture.root, fixture.head),
    /ENOENT/u,
  );
});
