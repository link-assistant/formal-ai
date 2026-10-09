// Issue #1181: standalone CLI archives on every release.
//
// The release itself is only observable after the next tag, so this test runs
// the mechanism now, from the files that will run then:
//   R1181-1  the `cli` job's matrix is the five requested targets, it uploads
//            to the resolved release, and the release resolver treats a release
//            without those archives as partial (so it builds them).
//   R1181-2  the job's own "Package the CLI archive" and "Smoke test CLI
//            archive" step scripts, executed with a stand-in binary, produce
//            `formal-ai-cli-<triple>.tar.gz` holding `formal-ai`, LICENSE and
//            README.md under `formal-ai-cli-<triple>/`, bound to shared producer receipts from the pinned
//            `--locked` command; the fixture makes no native compile claim.
//   R1181-3  `finalize` waits for `cli`, and its manifest step, executed on the
//            fragments, puts the CLI archive in SHA256SUMS.txt and reports the
//            legs that produced nothing as INCOMPLETE.
//   R1181-4  `[package.metadata.binstall]` resolves, for every matrix target,
//            the URL and in-archive path the workflow produces.
//   R1181-5  scripts/install.sh, run against a stand-in GitHub (recording curl,
//            fixed uname), installs the archive the package step built after
//            checking it against SHA256SUMS.txt, aborts on a checksum mismatch
//            without falling back, and falls back to `cargo install` only when
//            no archive exists for the host. install.ps1 is parsed by
//            PowerShell where `pwsh` is installed (the Linux CI runners).

import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { cliArchiveFixture } from "../fixtures/native-release-evidence/cli-archive.mjs";
import { createEvidenceFixture, materializePublishedFixture } from "../fixtures/native-release-evidence/observations.mjs";
import { collectReleaseEvidence, prepareReleaseEvidence } from "../../../scripts/native-release-evidence.mjs";
import { verifyArchivedExecutable } from "../../../scripts/native-release-artifact.mjs";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");
const WORKFLOW = read(".github/workflows/desktop-release.yml");
const REQUESTED_TARGETS = [
  "x86_64-unknown-linux-musl",
  "aarch64-unknown-linux-musl",
  "x86_64-apple-darwin",
  "aarch64-apple-darwin",
  "x86_64-pc-windows-msvc",
];

// One job of the workflow: its header line through the next two-space key.
function job(name) {
  const lines = WORKFLOW.split("\n");
  const start = lines.indexOf(`  ${name}:`);
  assert.ok(start >= 0, `desktop-release.yml has a ${name} job`);
  const end = lines.findIndex((line, index) => index > start && /^ {2}[^ #][^:]*:\s*$/.test(line));
  return lines.slice(start, end < 0 ? lines.length : end).join("\n");
}

// The matrix legs of the cli job as objects.
function cliLegs() {
  return job("cli").split("\n")
    .filter((line) => /^\s+- \{ os:/.test(line))
    .map((line) => Object.fromEntries(line.trim().replace(/^- \{|\}$/g, "").split(",")
      .map((pair) => pair.split(":").map((part) => part.trim().replace(/^"|"$/g, "")))));
}

// The `run: |` body of a named step, with the leg's `${{ matrix.* }}` filled in.
function stepScript(jobText, stepName, leg = {}) {
  const lines = jobText.split("\n");
  const step = lines.findIndex((line) => line.trim() === `- name: ${stepName}`);
  assert.ok(step >= 0, `step "${stepName}" exists`);
  const run = lines.findIndex((line, index) => index > step && line.trim() === "run: |");
  const indent = lines[run].indexOf("run:") + 2;
  const body = [];
  for (const line of lines.slice(run + 1)) {
    if (line.trim() && line.search(/\S/) < indent) break;
    body.push(line.slice(indent));
  }
  return body.join("\n").replace(/\$\{\{ matrix\.([a-z]+) \}\}/g, (_, key) => leg[key] ?? "");
}

const scratch = () => mkdtempSync(path.join(tmpdir(), "formal-ai-1181-"));
const bash = (script, options) => execFileSync("bash", ["-c", script], { encoding: "utf8", ...options });

test("R1181-1: the cli job builds the five requested targets and uploads to the release", () => {
  const legs = cliLegs();
  assert.deepEqual(legs.map((leg) => leg.target).sort(), [...REQUESTED_TARGETS].sort());
  for (const leg of legs) {
    assert.equal(leg.archive, leg.target.includes("windows") ? "zip" : "tar.gz", `${leg.target} archive`);
    assert.ok(leg.label.startsWith("cli-"), `${leg.target} label is cli-prefixed`);
  }
  const cli = job("cli");
  assert.match(cli, /needs: \[resolve, native-source, native\]/);
  const producer = job("native");
  for (const target of REQUESTED_TARGETS) assert.ok(producer.includes(`target: ${target},`), `${target} has a shared producer`);
  assert.match(cli, /native-release-artifact\.mjs" verify native-bin native-source/u);
  assert.match(cli, /gh release upload "\$TAG" "formal-ai-cli-\$\{\{ matrix\.target \}\}\.\$\{\{ matrix\.archive \}\}"/);
  assert.match(cli, /TAG: \$\{\{ needs\.resolve\.outputs\.tag \}\}/);
  assert.match(cli, /contents: write/);
});

test("R1181-2: the package and smoke-test steps build the archive layout from the pinned command", () => {
  const cli = job("cli");
  assert.match(job("native"), /cargo build --manifest-path rust\/Cargo\.toml --target-dir target --release --bin formal-ai --locked --target \$\{\{ matrix\.target \}\}/);
  assert.doesNotMatch(cli, /run: cargo build/u);
  assert.match(cli, /cp "\$FORMAL_AI_VERIFIED_NATIVE_BINARY"/u);
  const leg = cliLegs().find((entry) => entry.archive === "tar.gz");
  const dir = scratch();
  try {
    const fixture = cliArchiveFixture(dir, leg.target);
    bash(stepScript(cli, "Package the CLI archive", leg), { cwd: dir, env: fixture.environment });
    const archive = `formal-ai-cli-${leg.target}.tar.gz`;
    const listing = bash(`tar -tzf ${archive}`, { cwd: dir }).split("\n").filter(Boolean).map((entry) => entry.replace(/\/$/, ""));
    for (const entry of ["formal-ai", "LICENSE", "README.md"]) {
      assert.ok(listing.includes(`formal-ai-cli-${leg.target}/${entry}`), `${archive} holds ${entry}: ${listing}`);
    }
    const smoke = bash(stepScript(cli, "Smoke test CLI archive", leg), { cwd: dir, env: fixture.environment });
    assert.match(smoke, /formal-ai 9\.9\.9/);
    const extracted = path.join(dir, `smoke-${leg.target}`, `formal-ai-cli-${leg.target}`, "formal-ai");
    assert.deepEqual(readFileSync(extracted), readFileSync(fixture.binary), "archive preserves every verified executable byte");
    assert.equal(verifyArchivedExecutable(extracted, fixture.receipt), extracted);
    writeFileSync(extracted, "different executable bytes\n");
    assert.throws(() => verifyArchivedExecutable(extracted, fixture.receipt), "substituted extracted bytes must fail");
    bash(stepScript(cli, "Collect CLI checksum fragment", leg), { cwd: dir });
    assert.match(readFileSync(path.join(dir, `SHA256SUMS-${leg.label}.partial`), "utf8"), new RegExp(`^[0-9a-f]{64}  ${archive}\\n$`));
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("R1181-3: finalize consolidates the CLI fragments and reports missing legs as INCOMPLETE", () => {
  const finalize = job("finalize");
  assert.match(finalize, /needs: \[[^\]]*\bcli\b[^\]]*\]/);
  const script = stepScript(finalize, "Build consolidated SHA256SUMS.txt and BUILD-PROVENANCE.txt");
  for (const leg of cliLegs()) {
    assert.match(script, new RegExp(`\\[${leg.label}\\]="${leg.os}"`), `builder_of maps ${leg.label}`);
    assert.match(script, new RegExp(`for label in [^;]*\\b${leg.label}\\b`), `the label loop covers ${leg.label}`);
  }
  const dir = scratch();
  try {
    const leg = cliLegs()[0];
    mkdirSync(path.join(dir, "fragments"));
    const line = `${"a".repeat(64)}  formal-ai-cli-${leg.target}.tar.gz\n`;
    writeFileSync(path.join(dir, "fragments", `SHA256SUMS-${leg.label}.partial`), line);
    mkdirSync(path.join(dir, "bin"));
    writeFileSync(path.join(dir, "bin", "gh"), "#!/bin/sh\necho 0123abcd\n");
    chmodSync(path.join(dir, "bin", "gh"), 0o755);
    writeFileSync(path.join(dir, "output"), "");
    const evidenceFixture = createEvidenceFixture({version: "9.9.9"});
    const evidence = prepareReleaseEvidence(path.join(dir, "release-evidence"), collectReleaseEvidence(evidenceFixture));
    assert.equal(evidence.complete, true, "canonical typed fixture is complete");
    bash(script, {
      cwd: dir,
      env: { ...process.env, PATH: `${path.join(dir, "bin")}:${process.env.PATH}`, GH_TOKEN: "x", REPO: "o/r", TAG: "v9.9.9", RUN_URL: "u", GITHUB_OUTPUT: path.join(dir, "output"),
        NATIVE_SOURCE_COMMIT: evidenceFixture.source.source_commit, NATIVE_SOURCE_TREE: evidenceFixture.source.source_tree,
        NATIVE_SELECTION_SHA256: evidenceFixture.bindings.sourceSha256, NATIVE_COMPILER_RELEASE: evidenceFixture.source.compiler_release,
        NATIVE_COMPILER_COMMIT: evidenceFixture.source.compiler_commit, EVIDENCE_INCOMPLETE: "" },
    });
    const evidenceLines = readFileSync(path.join(dir, "release-evidence", "SHA256SUMS-evidence.partial"), "utf8").trim().split("\n");
    const expectedLines = [line.trim(), ...evidenceLines].sort((left, right) => left.split("  ")[1].localeCompare(right.split("  ")[1], "en"));
    assert.equal(readFileSync(path.join(dir, "SHA256SUMS.txt"), "utf8"), expectedLines.join("\n") + "\n");
    const provenance = readFileSync(path.join(dir, "BUILD-PROVENANCE.txt"), "utf8");
    assert.match(provenance, new RegExp(`${leg.os} \\(${leg.label}\\)`));
    assert.match(provenance, /INCOMPLETE : no artifacts from: .*cli-/);
    assert.match(readFileSync(path.join(dir, "output"), "utf8"), /^incomplete=/m);
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("R1181-1: the release resolver builds a release that lacks the CLI archives", () => {
  const dir = scratch();
  try {
    mkdirSync(path.join(dir, "bin"));
    const published = path.join(dir, "published");
    const head = "a".repeat(40), commit = "b".repeat(40), tree = "d".repeat(40);
    const mock = "#!/usr/bin/env node\n" + "const fixture=" + JSON.stringify({published,head,commit,tree}) + ";\n" + String.raw`
const fs=require('node:fs'),path=require('node:path');
const args=process.argv.slice(2), endpoint=args[1]??'';
const write=value=>process.stdout.write(String(value)+'\n');
if(args[0]==='api') {
 if(endpoint.includes('/tags?'))write('');
 else if(endpoint.includes('/commits/'))write(args.at(-1)==='.parents[0].sha'?fixture.head:args.at(-1)==='.commit.tree.sha'?fixture.tree:fixture.commit);
 else if(endpoint.includes('/releases/tags/'))write(fs.readFileSync(path.join(fixture.published,'release-assets.json'),'utf8'));
 else process.exit(2);
} else if(args[0]==='release'&&args[1]==='view') {
 if(args.includes('assets'))write(process.env.MOCK_ASSET_NAMES??'');
 else write(args[2]?.startsWith('--')?'v0.9.0':'{}');
} else if(args[0]==='release'&&args[1]==='download') {
 const destination=args[args.indexOf('--dir')+1];fs.mkdirSync(destination,{recursive:true});
 for(const name of fs.readdirSync(fixture.published))fs.copyFileSync(path.join(fixture.published,name),path.join(destination,name));
} else if(args[0]==='attestation'&&args[1]==='verify') {
 // Explicit fixture attestation response, not an actual GitHub signing claim.
} else process.exit(2);
`;
    writeFileSync(path.join(dir, "bin", "gh"), mock);chmodSync(path.join(dir, "bin", "gh"), 0o755);
    const names = bash(`source <(sed -n '/^expected_.*() {/,/^}/p' scripts/desktop-release-resolve.sh); DESKTOP_RELEASE_WORKFLOW=.github/workflows/desktop-release.yml; expected_desktop_assets 0.9.0`, {cwd: REPO_ROOT}).trim().split("\n");
    const cli = new Set(cliLegs().map((leg) => `formal-ai-cli-${leg.target}.${leg.archive}`));
    const resolve = (assets) => {
      rmSync(published, {recursive: true, force: true});
      materializePublishedFixture(published, {version: "0.9.0", sourceCommit: commit, sourceTree: tree, expectedAssets: assets});
      const output = path.join(dir, "output");writeFileSync(output, "");
      const result = spawnSync("bash", [path.join(REPO_ROOT, "scripts", "desktop-release-resolve.sh")], {
        env: {...process.env, PATH: `${path.join(dir, "bin")}:${process.env.PATH}`, EVENT: "workflow_run", WORKFLOW_RUN_HEAD_SHA: head,
          REPO: "o/r", GH_TOKEN: "x", GITHUB_OUTPUT: output, MOCK_ASSET_NAMES: assets.join("\n")}, encoding: "utf8",
      });
      assert.equal(result.status, 0, result.stderr);
      return readFileSync(output, "utf8").match(/^should_build=(.*)$/m)?.[1];
    };
    assert.equal(resolve(names), "false", "a complete release with verified durable observations is not rebuilt");
    assert.equal(resolve(names.filter((name) => !cli.has(name))), "true", "a release without the CLI archives is rebuilt");
  } finally {
    rmSync(dir, {recursive: true, force: true});
  }
});
test("R1181-4: cargo binstall resolves the archive and path the workflow produces", () => {
  const cargo = read("rust/Cargo.toml");
  const section = (header) => {
    const start = cargo.indexOf(`[${header}]\n`);
    assert.ok(start >= 0, `Cargo.toml has [${header}]`);
    const body = cargo.slice(start + header.length + 3);
    const end = body.search(/^\[/m);
    return Object.fromEntries((end < 0 ? body : body.slice(0, end)).split("\n")
      .map((line) => line.match(/^([a-z-]+) = "(.*)"$/)).filter(Boolean).map((match) => [match[1], match[2]]));
  };
  const base = section("package.metadata.binstall");
  const fill = (template, target, ext) => template
    .replaceAll("{ repo }", "https://github.com/link-assistant/formal-ai")
    .replaceAll("{ version }", "9.9.9")
    .replaceAll("{ target }", target)
    .replaceAll("{ bin }", "formal-ai")
    .replaceAll("{ binary-ext }", ext);
  for (const leg of cliLegs()) {
    const override = cargo.includes(`[package.metadata.binstall.overrides.${leg.target}]`)
      ? section(`package.metadata.binstall.overrides.${leg.target}`) : {};
    const meta = { ...base, ...override };
    const archive = `formal-ai-cli-${leg.target}.${leg.archive}`;
    assert.equal(fill(meta["pkg-url"], leg.target, leg.binext),
      `https://github.com/link-assistant/formal-ai/releases/download/v9.9.9/${archive}`, `${leg.target} pkg-url`);
    assert.equal(meta["pkg-fmt"], leg.archive === "zip" ? "zip" : "tgz", `${leg.target} pkg-fmt`);
    assert.equal(fill(meta["bin-dir"], leg.target, leg.binext), `formal-ai-cli-${leg.target}/formal-ai${leg.binext}`);
  }
});

// A stand-in GitHub for scripts/install.sh: `curl` serves files from `served`
// by URL basename and logs each URL, `uname` reports the given host, and
// `cargo` records that the fallback ran.
function installHarness(dir, { os, arch }) {
  const bin = path.join(dir, "bin");
  mkdirSync(bin);
  const served = path.join(dir, "served");
  mkdirSync(served);
  writeFileSync(path.join(bin, "curl"), `#!/bin/sh
dest=""; url=""
while [ $# -gt 0 ]; do
  case "$1" in -o) dest="$2"; shift ;; https://*) url="$1" ;; esac
  shift
done
echo "$url" >> "${dir}/curl.log"
file="${served}/$(basename "$url")"
[ -f "$file" ] || exit 22
if [ -n "$dest" ]; then cp "$file" "$dest"; else cat "$file"; fi
`);
  writeFileSync(path.join(bin, "uname"), `#!/bin/sh\ncase "$1" in -s) echo ${os} ;; -m) echo ${arch} ;; esac\n`);
  writeFileSync(path.join(bin, "cargo"), `#!/bin/sh\necho "$*" >> "${dir}/cargo.log"\n`);
  writeFileSync(path.join(bin, "wget"), "#!/bin/sh\nexit 1\n");
  for (const name of ["curl", "uname", "cargo", "wget"]) chmodSync(path.join(bin, name), 0o755);
  return { bin, served };
}

function runInstaller(dir, bin) {
  return spawnSync("sh", [path.join(REPO_ROOT, "scripts", "install.sh"), "cli"], {
    env: {
      PATH: `${bin}:${process.env.PATH}`,
      HOME: path.join(dir, "home"),
      FORMAL_AI_INSTALL_BIN: path.join(dir, "installed"),
    },
    encoding: "utf8",
  });
}

function releaseJson(names) {
  return JSON.stringify({
    tag_name: "v9.9.9",
    assets: names.map((name) => ({ name, browser_download_url: `https://github.com/link-assistant/formal-ai/releases/download/v9.9.9/${name}` })),
  });
}

// Builds the linux-x64 archive with the workflow's own package step.
function packagedArchive(dir) {
  const leg = cliLegs().find((entry) => entry.target === "x86_64-unknown-linux-musl");
  const work = path.join(dir, "work");
  const fixture = cliArchiveFixture(work, leg.target);
  bash(stepScript(job("cli"), "Package the CLI archive", leg), { cwd: work, env: fixture.environment });
  bash(stepScript(job("cli"), "Collect CLI checksum fragment", leg), { cwd: work });
  return {
    archive: path.join(work, `formal-ai-cli-${leg.target}.tar.gz`),
    sums: readFileSync(path.join(work, `SHA256SUMS-${leg.label}.partial`), "utf8"),
  };
}

test("R1181-5: install.sh installs the verified prebuilt archive without cargo", () => {
  const dir = scratch();
  try {
    const { bin, served } = installHarness(dir, { os: "Linux", arch: "x86_64" });
    const { archive, sums } = packagedArchive(dir);
    copyFileSync(archive, path.join(served, path.basename(archive)));
    writeFileSync(path.join(served, "SHA256SUMS.txt"), sums);
    writeFileSync(path.join(served, "latest"), releaseJson([path.basename(archive), "SHA256SUMS.txt"]));
    const run = runInstaller(dir, bin);
    assert.equal(run.status, 0, run.stderr);
    assert.match(run.stderr, /checksum OK for formal-ai-cli-x86_64-unknown-linux-musl\.tar\.gz/);
    assert.match(run.stderr, /CLI installed at/);
    assert.equal(execFileSync(path.join(dir, "installed", "formal-ai"), ["--version"], { encoding: "utf8" }).trim(), "formal-ai 9.9.9");
    assert.ok(!existsSync(path.join(dir, "cargo.log")), "cargo install never ran");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("R1181-5: a checksum mismatch aborts and never falls back to cargo", () => {
  const dir = scratch();
  try {
    const { bin, served } = installHarness(dir, { os: "Linux", arch: "x86_64" });
    const { archive, sums } = packagedArchive(dir);
    copyFileSync(archive, path.join(served, path.basename(archive)));
    writeFileSync(path.join(served, "SHA256SUMS.txt"), sums.replace(/^[0-9a-f]{64}/, "0".repeat(64)));
    writeFileSync(path.join(served, "latest"), releaseJson([path.basename(archive), "SHA256SUMS.txt"]));
    const run = runInstaller(dir, bin);
    assert.notEqual(run.status, 0);
    assert.match(run.stderr, /checksum MISMATCH/);
    assert.ok(!existsSync(path.join(dir, "cargo.log")), "a mismatch is not silently replaced by cargo install");
    assert.ok(!existsSync(path.join(dir, "installed", "formal-ai")), "nothing was installed");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("R1181-5: a host with no archive falls back to cargo install", () => {
  const dir = scratch();
  try {
    // Linux on riscv64: no matrix leg builds it.
    const { bin, served } = installHarness(dir, { os: "Linux", arch: "riscv64" });
    writeFileSync(path.join(served, "latest"), releaseJson(cliLegs().map((leg) => `formal-ai-cli-${leg.target}.${leg.archive}`)));
    const run = runInstaller(dir, bin);
    assert.equal(run.status, 0, run.stderr);
    assert.match(run.stderr, /no prebuilt CLI archive for this OS\/arch/);
    assert.equal(readFileSync(path.join(dir, "cargo.log"), "utf8").trim(), "install formal-ai");
    assert.ok(!readFileSync(path.join(dir, "curl.log"), "utf8").includes("formal-ai-cli-"), "no archive was downloaded");
  } finally {
    rmSync(dir, { recursive: true, force: true });
  }
});

test("R1181-5: install.sh and install.ps1 map hosts to the matrix targets", () => {
  const targets = new Set(cliLegs().map((leg) => leg.target));
  const shellTargets = [...read("scripts/install.sh").matchAll(/echo "([a-z0-9_]+-[a-z0-9_-]+)" ;;/g)].map((match) => match[1]);
  assert.deepEqual(new Set(shellTargets), targets, "install.sh maps every host to a built target and nothing else");
  const ps1 = read("scripts/install.ps1");
  const psTargets = [...ps1.matchAll(/'((?:x86_64|aarch64)-[a-z0-9_-]+)'/g)].map((match) => match[1]);
  assert.ok(psTargets.length > 0 && psTargets.every((target) => targets.has(target)), `install.ps1 targets ${psTargets}`);
  assert.match(ps1, /function Install-CliPrebuilt/);
  assert.match(ps1, /Test-Checksum/);
});

test("R1181-5: install.ps1 parses under PowerShell", { skip: spawnSync("pwsh", ["-v"]).error ? "pwsh is not installed on this host" : false }, () => {
  const script = path.join(REPO_ROOT, "scripts", "install.ps1").replaceAll("'", "''");
  const run = spawnSync("pwsh", ["-NoProfile", "-NonInteractive", "-Command",
    `$errors = $null; [System.Management.Automation.Language.Parser]::ParseFile('${script}', [ref]$null, [ref]$errors) | Out-Null; if ($errors.Count) { $errors | ForEach-Object { $_.ToString() }; exit 1 }`], { encoding: "utf8" });
  assert.equal(run.status, 0, `${run.stdout}${run.stderr}`);
});
