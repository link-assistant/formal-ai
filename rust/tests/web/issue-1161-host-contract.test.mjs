// Issue #1161: the client registry names the variable that relocates each
// client's config, and the host integration contract tells hosts to use it
// instead of moving XDG_CONFIG_HOME or HOME (link-assistant/hive-mind#2314).
//
// R1161-4: every variable the seed declares is documented, per client, in the
// "Host integration contract" section and the *Relocate via* column of
// docs/configuration/agentic-clis.md, and the section forbids the XDG/HOME
// relocation. The expected variables are read from the seed, not listed here.
//
// R1161-6: Hive Mind's launcher (src/formal-ai-runtime.lib.mjs,
// `JSON_CONFIG_DIR_ENV`, commit de14357623 of link-assistant/hive-mind, closed
// #2314 on 2026-09-28) points each JSON-configured client at the directory of
// its `global_configs[].path` through one `*_DIR` variable. That only works
// while this registry keeps declaring that variable for a JSON entry, which is
// the contract pinned below.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..", "..");
const read = (relative) => readFileSync(path.join(REPO_ROOT, relative), "utf8");

// Each `tool "<id>"` block with the `global` sub-blocks it declares: the
// format (`kind`), the path, and its `config_env` children.
function seededGlobals() {
  const tools = new Map();
  let tool = null;
  let global = null;
  for (const line of read("data/seed/client-integrations.lino").split("\n")) {
    const toolMatch = line.match(/^ {2}tool "([^"]+)"/);
    if (toolMatch) {
      tool = { id: toolMatch[1], globals: [] };
      tools.set(tool.id, tool);
      global = null;
      continue;
    }
    if (!tool) continue;
    if (/^ {4}global\b/.test(line)) {
      global = { kind: "", path: "", configEnv: [] };
      tool.globals.push(global);
      continue;
    }
    if (/^ {4}\S/.test(line)) {
      global = null;
      continue;
    }
    if (!global) continue;
    const child = line.match(/^ {6}(kind|path|config_env) "([^"]*)"/);
    if (!child) continue;
    if (child[1] === "config_env") global.configEnv.push(child[2]);
    else global[child[1]] = child[2];
  }
  return [...tools.values()];
}

function contractSection() {
  const doc = read("docs/configuration/agentic-clis.md");
  const start = doc.indexOf("### Host integration contract");
  assert.ok(start >= 0, "agentic-clis.md has a Host integration contract section");
  const rest = doc.slice(start + 1);
  const end = rest.search(/\n#{2,3} /);
  return end < 0 ? rest : rest.slice(0, end);
}

// The bullets of the contract, each joined onto one line.
function contractBullets(section) {
  const bullets = [];
  for (const line of section.split("\n")) {
    if (line.startsWith("- ")) bullets.push(line.slice(2));
    else if (bullets.length && /^ {2}\S/.test(line)) bullets[bullets.length - 1] += ` ${line.trim()}`;
  }
  return bullets;
}

test("R1161-4: the contract documents every declared relocation variable per client", () => {
  const section = contractSection();
  const bullets = contractBullets(section);
  const declaring = seededGlobals().filter((tool) => tool.globals.some((g) => g.configEnv.length));
  assert.ok(declaring.length >= 3, "the seed declares relocation variables for several clients");
  for (const tool of declaring) {
    const bullet = bullets.find((text) => text.includes(`\`${tool.id}\``));
    assert.ok(bullet, `the contract names client \`${tool.id}\``);
    for (const variable of new Set(tool.globals.flatMap((g) => g.configEnv))) {
      assert.ok(bullet.includes(`\`${variable}\``), `the \`${tool.id}\` bullet names ${variable}`);
    }
  }
});

test("R1161-4: clients that declare no variable are named as declaring none", () => {
  const section = contractSection().replace(/\s+/g, " ");
  const silent = seededGlobals().filter((tool) =>
    tool.globals.length && tool.globals.every((g) => g.configEnv.length === 0));
  assert.ok(silent.length > 0, "some clients have no relocation variable");
  const sentence = section.slice(section.indexOf("declare none") - 400, section.indexOf("declare none"));
  for (const tool of silent) {
    assert.ok(sentence.includes(`\`${tool.id}\``), `the contract says \`${tool.id}\` declares none`);
  }
});

test("R1161-4: the contract forbids relocating XDG_CONFIG_HOME or HOME and cites the incident", () => {
  const section = contractSection().replace(/\s+/g, " ");
  assert.match(section, /never by relocating `XDG_CONFIG_HOME` or `HOME`/);
  assert.match(section, /hive-mind\/issues\/2314/);
  assert.match(section, /`global_configs\[\]\.config_env`/);
});

test("R1161-4: the Relocate via column lists every declared variable", () => {
  const doc = read("docs/configuration/agentic-clis.md");
  const header = doc.split("\n").find((line) => line.startsWith("| Target |") && line.includes("Relocate via"));
  assert.ok(header, "the client table has a Relocate via column");
  const column = header.split("|").map((cell) => cell.trim()).indexOf("Relocate via");
  const cells = doc.split("\n")
    .filter((line) => line.startsWith("| ") && !line.startsWith("| ---"))
    .map((line) => line.split("|").map((cell) => cell.trim())[column] || "");
  const documented = cells.join(" ");
  for (const tool of seededGlobals()) {
    for (const variable of tool.globals.flatMap((g) => g.configEnv)) {
      assert.ok(documented.includes(`\`${variable}\``), `Relocate via lists ${variable} (${tool.id})`);
    }
  }
});

test("R1161-6: every JSON-configured client Hive Mind launches keeps its *_DIR variable", () => {
  // Hive Mind's JSON_CONFIG_DIR_ENV (de14357623): agent and opencode, each
  // pointed at dirname(global_configs[].path) through one *_DIR variable.
  const hiveMindLauncher = { agent: "LINK_ASSISTANT_AGENT_CONFIG_DIR", opencode: "OPENCODE_CONFIG_DIR" };
  const tools = new Map(seededGlobals().map((tool) => [tool.id, tool]));
  for (const [id, variable] of Object.entries(hiveMindLauncher)) {
    const json = tools.get(id)?.globals.find((g) => g.kind === "json");
    assert.ok(json, `${id} keeps a JSON global config`);
    assert.ok(json.configEnv.includes(variable), `${id}'s JSON global config declares ${variable}`);
    assert.ok(path.posix.dirname(json.path) !== ".", `${id}'s config sits in its own directory`);
  }
  // No JSON-configured client relies on XDG_CONFIG_HOME for relocation.
  for (const tool of tools.values()) {
    for (const global of tool.globals) {
      assert.ok(!global.configEnv.includes("XDG_CONFIG_HOME"), `${tool.id} never declares XDG_CONFIG_HOME`);
      assert.ok(!global.configEnv.includes("HOME"), `${tool.id} never declares HOME as a config variable`);
    }
  }
});
