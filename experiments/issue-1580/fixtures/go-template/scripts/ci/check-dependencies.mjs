#!/usr/bin/env bun

// Check active CI pins against upstream releases, including immutable ref integrity.
// Registry errors fail the check so unavailable metadata cannot conceal stale pins.
//
// A newer upstream release is a fact about the world, not about the commit, so
// it only fails the check where a contributor can act on it: pull requests and
// local runs. On push, schedule and workflow_dispatch it is reported as a
// warning and in the step summary, and Dependabot proposes the bump. Pin
// integrity, malformed pins and lookup failures fail on every event.
import { execFileSync } from "node:child_process";
import { appendFileSync, readFileSync, readdirSync } from "node:fs";
import { printUntrusted } from "../github-actions-log.mjs";

const VERSION = /^\d+\.\d+\.\d+$/;

export function findDependencies(workflows) {
  const dependencies = [];
  for (const { path, source } of workflows) {
    for (const [index, line] of source.split("\n").entries()) {
      const location = `${path}:${index + 1}`;
      const action = line.match(/^\s*-?\s*uses:\s+([^\s#]+)/);
      if (action) {
        if (action[1].startsWith("./")) continue;
        const match = line.match(
          /uses:\s+(docker:\/\/)?([\w.-]+\/[\w.-]+)@(\S+)\s+#\s+v(\d+\.\d+\.\d+)\s*$/
        );
        if (
          !match ||
          !(match[1] ? /^sha256:[0-9a-f]{64}$/ : /^[0-9a-f]{40}$/).test(
            match[3]
          )
        ) {
          throw new Error(
            `${location}: pin actions to a commit SHA and containers to a digest, with an exact version comment`
          );
        }
        dependencies.push({
          location,
          name: match[2],
          ref: match[3],
          version: match[4],
          kind: match[1] ? "container" : "action",
        });
        continue;
      }
      const tool = line.match(
        /^\s*(bun-version|go-version):\s*['"]?([^\s'"#]+)/
      );
      const staticcheck = line.match(
        /go install honnef\.co\/go\/tools\/cmd\/staticcheck@([^\s]+)/
      );
      if (tool || staticcheck) {
        const version = tool ? tool[2] : staticcheck[1];
        if (!VERSION.test(version))
          throw new Error(
            `${location}: pin tool versions to an exact stable release`
          );
        dependencies.push({
          location,
          name: staticcheck
            ? "dominikh/go-tools"
            : tool[1] === "bun-version"
              ? "oven-sh/bun"
              : "go",
          version,
          kind: "tool",
        });
      }
    }
  }
  return dependencies;
}

function github(endpoint) {
  return JSON.parse(
    execFileSync("gh", ["api", endpoint], { encoding: "utf8" })
  );
}

async function getJson(url, options) {
  const response = await fetch(url, options);
  if (!response.ok) throw new Error(`${url} returned HTTP ${response.status}`);
  return response.json();
}

async function latestVersion(dependency) {
  if (dependency.name === "go") {
    const releases = await getJson("https://go.dev/dl/?mode=json");
    const version = releases.find((release) => release.stable)?.version;
    if (!version) throw new Error("Go endpoint returned no stable releases");
    const current = version.replace(/^go/, "");
    if (!VERSION.test(current))
      throw new Error(`Unsupported upstream release ${version}`);
    return current;
  }
  const release = github(`repos/${dependency.name}/releases/latest`);
  if (release.draft || release.prerelease)
    throw new Error("Upstream latest release is not stable");
  const version = release.tag_name.replace(/^(?:bun-)?v/, "");
  if (!VERSION.test(version))
    throw new Error(`Unsupported upstream release ${release.tag_name}`);
  return version;
}

async function verifyPin(dependency) {
  if (dependency.kind === "action") {
    let object = github(
      `repos/${dependency.name}/git/ref/tags/v${dependency.version}`
    ).object;
    // Actions such as Codecov use annotated tags; resolve to the actual commit.
    for (let depth = 0; object.type === "tag" && depth < 5; depth += 1) {
      object = github(`repos/${dependency.name}/git/tags/${object.sha}`).object;
    }
    if (object.type !== "commit" || object.sha !== dependency.ref)
      throw new Error("Pinned SHA does not match the release tag");
  } else if (dependency.kind === "container") {
    const { token } = await getJson(
      `https://auth.docker.io/token?service=registry.docker.io&scope=repository:${dependency.name}:pull`
    );
    const response = await fetch(
      `https://registry-1.docker.io/v2/${dependency.name}/manifests/${dependency.version}`,
      {
        headers: {
          Authorization: `Bearer ${token}`,
          Accept:
            "application/vnd.oci.image.index.v1+json, application/vnd.docker.distribution.manifest.list.v2+json",
        },
      }
    );
    if (!response.ok)
      throw new Error(`Container registry returned HTTP ${response.status}`);
    const digest = response.headers.get("docker-content-digest");
    await response.arrayBuffer();
    if (digest !== dependency.ref)
      throw new Error("Pinned digest does not match the release image");
  }
}

export async function checkDependencies(
  dependencies,
  { latest = latestVersion, verifyPin: verify = verifyPin } = {}
) {
  const errors = [];
  const versions = new Map();
  const pins = new Set();
  for (const dependency of dependencies) {
    try {
      if (!versions.has(dependency.name))
        versions.set(dependency.name, await latest(dependency));
      const current = versions.get(dependency.name);
      if (dependency.version !== current) {
        errors.push({
          stale: true,
          message: `${dependency.location}: ${dependency.name} pins ${dependency.version}; latest stable is ${current}`,
        });
        continue;
      }
      const key = `${dependency.name}@${dependency.ref}`;
      if (dependency.ref && !pins.has(key)) {
        await verify(dependency);
        pins.add(key);
      }
    } catch (error) {
      errors.push({
        stale: false,
        message: `${dependency.location}: ${dependency.name}: ${error.message}`,
      });
    }
  }
  return errors;
}

// Events whose commit did not cause a newer upstream release. Every other
// event, including pull_request and a local run, gates on stale pins.
export const WARN_ONLY_EVENTS = new Set(["push", "schedule", "workflow_dispatch"]);

/**
 * Print the outcome and return the process exit code.
 *
 * @param {{stale: boolean, message: string}[]} errors
 * @param {{event?: string, total?: number, summaryFile?: string,
 *   stdout?: {write: (text: string) => unknown},
 *   stderr?: {write: (text: string) => unknown}, githubActions?: boolean}} options
 */
export function report(errors, options = {}) {
  const {
    event = "",
    total = 0,
    summaryFile,
    stdout = process.stdout,
    stderr = process.stderr,
    githubActions,
  } = options;
  const gate = !WARN_ONLY_EVENTS.has(event);
  const failing = errors.filter((error) => gate || !error.stale);
  const warnings = errors.filter((error) => !failing.includes(error));
  const print = (error, stream) =>
    printUntrusted(error.message, { stream, githubActions });

  if (warnings.length) {
    stdout.write(
      `::warning title=Newer CI dependency releases::${warnings.length} pinned CI dependencies have newer upstream releases. ` +
        `This does not fail ${event} runs; update the pins (Dependabot proposes action bumps).\n`
    );
    for (const error of warnings) print(error, stdout);
  }
  if (failing.length) {
    stderr.write("::error::CI dependencies need attention\n");
    for (const error of failing) print(error, stderr);
  }
  if (!errors.length) {
    stdout.write(
      `All ${total} CI dependency pins match current stable releases\n`
    );
  }

  if (summaryFile && errors.length) {
    // Backticks are removed so upstream text cannot break out of code spans.
    const line = (error) => `- \`${error.message.replaceAll("`", "'")}\``;
    const sections = ["## CI dependency check\n"];
    if (failing.length)
      sections.push(
        `### Failing (${failing.length})\n\n${failing.map(line).join("\n")}\n`
      );
    if (warnings.length)
      sections.push(
        `### Newer upstream releases (warning on \`${event}\`, gating on pull requests) (${warnings.length})\n\n` +
          `${warnings.map(line).join("\n")}\n`
      );
    appendFileSync(summaryFile, `${sections.join("\n")}\n`);
  }
  return failing.length ? 1 : 0;
}

async function main() {
  const directory = ".github/workflows";
  const workflows = readdirSync(directory)
    .filter((file) => /\.ya?ml$/.test(file))
    .map((file) => ({
      path: `${directory}/${file}`,
      source: readFileSync(`${directory}/${file}`, "utf8"),
    }));
  const dependencies = findDependencies(workflows);
  const errors = await checkDependencies(dependencies);
  process.exitCode = report(errors, {
    event: process.env.GITHUB_EVENT_NAME,
    total: dependencies.length,
    summaryFile: process.env.GITHUB_STEP_SUMMARY,
  });
}

if (import.meta.main) {
  try {
    await main();
  } catch (error) {
    console.error("::error::CI dependency check failed");
    printUntrusted(error.message, { stream: process.stderr });
    process.exitCode = 1;
  }
}
