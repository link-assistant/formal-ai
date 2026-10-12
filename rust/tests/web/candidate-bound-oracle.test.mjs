// Original physical behavior fixtures; structural completion is not semantic usefulness.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {mkdtempSync,realpathSync,mkdirSync,writeFileSync,unlinkSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import test from 'node:test';
import {executeCandidateBoundOracle} from '../../../experiments/formal_ai_subagent/candidate-bound-oracle.mjs';
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
const fixtures=[
  {
    "name": "owner/positive",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": true
  },
  {
    "name": "owner/wrong",
    "candidate": "export function calculate(left,right){return left-right;}",
    "candidateSHA256": "4fa90ba5fa356cd46cee87032d433e8319cf97046e3e24eda2597e6e3abff3b8",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/absent",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "absent",
    "expected": false
  },
  {
    "name": "owner/drift",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "drift",
    "expected": false
  },
  {
    "name": "owner/unreachable",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('unreachable',()=>{if(false){assert.equal(calculate(2,3),5);}});",
    "oracleSHA256": "8c89829d5ba409a0cbe1be7f64013705272c33e881ec5c682f7c8391a24dd07e",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/skip",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test.skip('skip',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "914f92e9c6854e8206ca278dd8ce3cd0c533f7f485e73d1bfdae85f329d4bfd2",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/padding",
    "candidate": "export function calculate(left,right){return left+right;}export function padding(x){return x;}",
    "candidateSHA256": "be8c849e0697dfee733f3b7f0bf88a3ad8425c90550844a1593db5795057aec0",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate",
      "padding"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/constant",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('constant',()=>{assert.equal(5,5);});",
    "oracleSHA256": "4e2ee2cb370f51420ecc068fe864778589a1847c9d383d0a7dbf5b499f20ad54",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/stdout",
    "candidate": "export function calculate(left,right){console.log('1..1');return left+right;}",
    "candidateSHA256": "af6691ec4b9ee389e090cdaae565cd38822c79ad528d7a866e93c4ee6006cffb",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/process",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('process',()=>{process.stdout.write('1..1');assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "2ce674148604dc5734a227c331b95e70564472e4be9a1d41f17dfdbdcd202d7e",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/shadow",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('shadow',()=>{const calculate=5;assert.equal(calculate,5);});",
    "oracleSHA256": "f624976b9c74310b51dd4509f5a9548eda6cd0246d7a10eec9e51faf03fd7793",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/import",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './other.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "964dd2a64260bea21509df117f7aa5317763bbd220368e6a4d6554c0fda1bf10",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/declaration",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "workflow",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/native",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source-owned behavior',()=>{assert.equal(calculate(2,3),5);assert.equal(calculate(-1,4),3);});",
    "oracleSHA256": "781176eeb45bf7365a252b9927041a7ef1955680ff62a8430702d7ddb510ad3c",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "rust-native",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/zero",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('zero',()=>{});",
    "oracleSHA256": "ac631784b18f770f6183a719c193dec06d501c12c5610970209d4a8e01157856",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "owner/root-only",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "// Issue #1161: the client registry names the variable that relocates each\n// client's config, and the host integration contract tells hosts to use it\n// instead of moving XDG_CONFIG_HOME or HOME (link-assistant/hive-mind#2314).\n//\n// R1161-4: every variable the seed declares is documented, per client, in the\n// \"Host integration contract\" section and the *Relocate via* column of\n// docs/configuration/agentic-clis.md, and the section forbids the XDG/HOME\n// relocation. The expected variables are read from the seed, not listed here.\n//\n// R1161-6: Hive Mind's launcher (src/formal-ai-runtime.lib.mjs,\n// `JSON_CONFIG_DIR_ENV`, commit de14357623 of link-assistant/hive-mind, closed\n// #2314 on 2026-09-28) points each JSON-configured client at the directory of\n// its `global_configs[].path` through one `*_DIR` variable. That only works\n// while this registry keeps declaring that variable for a JSON entry, which is\n// the contract pinned below.\n\nimport assert from \"node:assert/strict\";\nimport { readFileSync } from \"node:fs\";\nimport path from \"node:path\";\nimport test from \"node:test\";\nimport { fileURLToPath } from \"node:url\";\n\nconst REPO_ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), \"..\", \"..\", \"..\");\nconst read = (relative) => readFileSync(path.join(REPO_ROOT, relative), \"utf8\");\n\n// Each `tool \"<id>\"` block with the `global` sub-blocks it declares: the\n// format (`kind`), the path, and its `config_env` children.\nfunction seededGlobals() {\n  const tools = new Map();\n  let tool = null;\n  let global = null;\n  for (const line of read(\"data/seed/client-integrations.lino\").split(\"\\n\")) {\n    const toolMatch = line.match(/^ {2}tool \"([^\"]+)\"/);\n    if (toolMatch) {\n      tool = { id: toolMatch[1], globals: [] };\n      tools.set(tool.id, tool);\n      global = null;\n      continue;\n    }\n    if (!tool) continue;\n    if (/^ {4}global\\b/.test(line)) {\n      global = { kind: \"\", path: \"\", configEnv: [] };\n      tool.globals.push(global);\n      continue;\n    }\n    if (/^ {4}\\S/.test(line)) {\n      global = null;\n      continue;\n    }\n    if (!global) continue;\n    const child = line.match(/^ {6}(kind|path|config_env) \"([^\"]*)\"/);\n    if (!child) continue;\n    if (child[1] === \"config_env\") global.configEnv.push(child[2]);\n    else global[child[1]] = child[2];\n  }\n  return [...tools.values()];\n}\n\nfunction contractSection() {\n  const doc = read(\"docs/configuration/agentic-clis.md\");\n  const start = doc.indexOf(\"### Host integration contract\");\n  assert.ok(start >= 0, \"agentic-clis.md has a Host integration contract section\");\n  const rest = doc.slice(start + 1);\n  const end = rest.search(/\\n#{2,3} /);\n  return end < 0 ? rest : rest.slice(0, end);\n}\n\n// The bullets of the contract, each joined onto one line.\nfunction contractBullets(section) {\n  const bullets = [];\n  for (const line of section.split(\"\\n\")) {\n    if (line.startsWith(\"- \")) bullets.push(line.slice(2));\n    else if (bullets.length && /^ {2}\\S/.test(line)) bullets[bullets.length - 1] += ` ${line.trim()}`;\n  }\n  return bullets;\n}\n\ntest(\"R1161-4: the contract documents every declared relocation variable per client\", () => {\n  const section = contractSection();\n  const bullets = contractBullets(section);\n  const declaring = seededGlobals().filter((tool) => tool.globals.some((g) => g.configEnv.length));\n  assert.ok(declaring.length >= 3, \"the seed declares relocation variables for several clients\");\n  for (const tool of declaring) {\n    const bullet = bullets.find((text) => text.includes(`\\`${tool.id}\\``));\n    assert.ok(bullet, `the contract names client \\`${tool.id}\\``);\n    for (const variable of new Set(tool.globals.flatMap((g) => g.configEnv))) {\n      assert.ok(bullet.includes(`\\`${variable}\\``), `the \\`${tool.id}\\` bullet names ${variable}`);\n    }\n  }\n});\n\ntest(\"R1161-4: clients that declare no variable are named as declaring none\", () => {\n  const section = contractSection().replace(/\\s+/g, \" \");\n  const silent = seededGlobals().filter((tool) =>\n    tool.globals.length && tool.globals.every((g) => g.configEnv.length === 0));\n  assert.ok(silent.length > 0, \"some clients have no relocation variable\");\n  const sentence = section.slice(section.indexOf(\"declare none\") - 400, section.indexOf(\"declare none\"));\n  for (const tool of silent) {\n    assert.ok(sentence.includes(`\\`${tool.id}\\``), `the contract says \\`${tool.id}\\` declares none`);\n  }\n});\n\ntest(\"R1161-4: the contract forbids relocating XDG_CONFIG_HOME or HOME and cites the incident\", () => {\n  const section = contractSection().replace(/\\s+/g, \" \");\n  assert.match(section, /never by relocating `XDG_CONFIG_HOME` or `HOME`/);\n  assert.match(section, /hive-mind\\/issues\\/2314/);\n  assert.match(section, /`global_configs\\[\\]\\.config_env`/);\n});\n\ntest(\"R1161-4: the Relocate via column lists every declared variable\", () => {\n  const doc = read(\"docs/configuration/agentic-clis.md\");\n  const header = doc.split(\"\\n\").find((line) => line.startsWith(\"| Target |\") && line.includes(\"Relocate via\"));\n  assert.ok(header, \"the client table has a Relocate via column\");\n  const column = header.split(\"|\").map((cell) => cell.trim()).indexOf(\"Relocate via\");\n  const cells = doc.split(\"\\n\")\n    .filter((line) => line.startsWith(\"| \") && !line.startsWith(\"| ---\"))\n    .map((line) => line.split(\"|\").map((cell) => cell.trim())[column] || \"\");\n  const documented = cells.join(\" \");\n  for (const tool of seededGlobals()) {\n    for (const variable of tool.globals.flatMap((g) => g.configEnv)) {\n      assert.ok(documented.includes(`\\`${variable}\\``), `Relocate via lists ${variable} (${tool.id})`);\n    }\n  }\n});\n\ntest(\"R1161-6: every JSON-configured client Hive Mind launches keeps its *_DIR variable\", () => {\n  // Hive Mind's JSON_CONFIG_DIR_ENV (de14357623): agent and opencode, each\n  // pointed at dirname(global_configs[].path) through one *_DIR variable.\n  const hiveMindLauncher = { agent: \"LINK_ASSISTANT_AGENT_CONFIG_DIR\", opencode: \"OPENCODE_CONFIG_DIR\" };\n  const tools = new Map(seededGlobals().map((tool) => [tool.id, tool]));\n  for (const [id, variable] of Object.entries(hiveMindLauncher)) {\n    const json = tools.get(id)?.globals.find((g) => g.kind === \"json\");\n    assert.ok(json, `${id} keeps a JSON global config`);\n    assert.ok(json.configEnv.includes(variable), `${id}'s JSON global config declares ${variable}`);\n    assert.ok(path.posix.dirname(json.path) !== \".\", `${id}'s config sits in its own directory`);\n  }\n  // No JSON-configured client relies on XDG_CONFIG_HOME for relocation.\n  for (const tool of tools.values()) {\n    for (const global of tool.globals) {\n      assert.ok(!global.configEnv.includes(\"XDG_CONFIG_HOME\"), `${tool.id} never declares XDG_CONFIG_HOME`);\n      assert.ok(!global.configEnv.includes(\"HOME\"), `${tool.id} never declares HOME as a config variable`);\n    }\n  }\n});\n",
    "oracleSHA256": "e319535c0117e2b8a03350293fe62ba5fde58607bc57d3b24cbb871838e2e606",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/positive_comments_data",
    "candidate": "export function calculate(left,right){return left+right;}// process.exit(42); get value(){ throw 1; }\n",
    "candidateSHA256": "4b83aa474037c42d0248dafedf2574815ba8b84828140a45d409f9290c427ecc",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5,'if(false){process.exit(1)}');});",
    "oracleSHA256": "bd760a01d06a6397181dc468fa68a46dbdc0b5ea016315574075ca7c2ecb2912",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": true
  },
  {
    "name": "independent/wrong_real_operand",
    "candidate": "export function calculate(left,right){return left-right;}",
    "candidateSHA256": "4fa90ba5fa356cd46cee87032d433e8319cf97046e3e24eda2597e6e3abff3b8",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "21f592c30c9fec73cd122e78cf175fff0c0c72cf8b9649d3081a0784e4c00c7d",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/message_only_equal_wrong",
    "candidate": "export function calculate(left,right){return left-right;}",
    "candidateSHA256": "4fa90ba5fa356cd46cee87032d433e8319cf97046e3e24eda2597e6e3abff3b8",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(5,5,calculate(2,3));});",
    "oracleSHA256": "c892c312aa9b4f4fe7611c8127d0947de8288c379380ab6326ed0c67e1b1f14d",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/message_only_ok_wrong",
    "candidate": "export function calculate(left,right){return left-right;}",
    "candidateSHA256": "4fa90ba5fa356cd46cee87032d433e8319cf97046e3e24eda2597e6e3abff3b8",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.ok(true,calculate(2,3));});",
    "oracleSHA256": "1cb9905594787acbcc13596f04197ff38c575d629ea59f0447ea0e66427e5b49",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/message_only_local_wrong",
    "candidate": "export function calculate(left,right){return left-right;}",
    "candidateSHA256": "4fa90ba5fa356cd46cee87032d433e8319cf97046e3e24eda2597e6e3abff3b8",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{const diagnostic=calculate(2,3);assert.strictEqual(5,5,diagnostic);});",
    "oracleSHA256": "361fe308b3b69e681f311cb8f0db05d55bca1df9b92a292bed778cb8ddb516c1",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/candidate_object_getter",
    "candidate": "export function calculate(a,b){return {get value(){throw 1;}};}",
    "candidateSHA256": "ad0e108eb01df4aa9b4deac8dc69268257d0a537885f2947262b67a33f4e27b5",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "21f592c30c9fec73cd122e78cf175fff0c0c72cf8b9649d3081a0784e4c00c7d",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/candidate_global",
    "candidate": "export function calculate(a,b){return process.exit(23);}",
    "candidateSHA256": "ee793bc062df5248b7bc6b2eb0e9c973ddef5c581157d0727c8608e08ec07820",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "21f592c30c9fec73cd122e78cf175fff0c0c72cf8b9649d3081a0784e4c00c7d",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/oracle_getter",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal({get value(){throw 1;}},calculate(2,3));});",
    "oracleSHA256": "46fd6b748b0b886207bdbda11f097125d282bd66387abc80e72e06df11336da4",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/oracle_object_data",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.deepEqual({value:calculate(2,3)},{value:5});});",
    "oracleSHA256": "dbad6712671135eb14ca08f5b196aea0a0c3a858b0df2cbf110d225fc33a3bb0",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/unknown_control_flow",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{if(true){assert.equal(calculate(2,3),5);}});",
    "oracleSHA256": "8e4fcef133bff84bfe01c596ae2b483da3cef15578261d3d4a6b70dfd5f4bd78",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/unknown_expression_eval",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(eval('calculate(2,3)'),5);});",
    "oracleSHA256": "ca5133bea135dc489ff45364208f21b1d6870462b60c72392871e5984430f887",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/candidate_syntax",
    "candidate": "export function calculate(a,b){return a+;}",
    "candidateSHA256": "425a87363c26001e16883aa02fecee05488156fa69d6c6f660f08ddc3f479c07",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "21f592c30c9fec73cd122e78cf175fff0c0c72cf8b9649d3081a0784e4c00c7d",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/oracle_syntax",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),); let = ;});",
    "oracleSHA256": "c420768f5d1c72668f5762d247d4b62dd58fd353bca2c4ac2de986808d2054da",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/partial_interface",
    "candidate": "export function calculate(left,right){return left+right;}export function other(a){return a;}",
    "candidateSHA256": "9ee04110c66f5db7f6c11d8bec81a6dfb5393a584a95149802e7b2d33e9d9930",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "21f592c30c9fec73cd122e78cf175fff0c0c72cf8b9649d3081a0784e4c00c7d",
    "interface": [
      "calculate",
      "other"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/dependency_supplied",
    "candidate": "export function calculate(left,right){return left+right;}",
    "candidateSHA256": "e5e3b2de877c7ab8b2b1a84389b61e1b4e28ae36f31c929161fa8571562e582b",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "21f592c30c9fec73cd122e78cf175fff0c0c72cf8b9649d3081a0784e4c00c7d",
    "interface": [
      "calculate"
    ],
    "dependencies": [
      {
        "source": "export function calculate(left,right){return left+right;}// process.exit(42); get value(){ throw 1; }\n",
        "sha256": "4b83aa474037c42d0248dafedf2574815ba8b84828140a45d409f9290c427ecc"
      }
    ],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/sideeffect_initializer",
    "candidate": "globalThis.__candidateSideEffect=1;export function calculate(left,right){return left+right;}",
    "candidateSHA256": "3328d69f47e1c61b653110d6e585c16f65326e94d95195537972caa3038a496a",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('original physical behavior',()=>{assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "21f592c30c9fec73cd122e78cf175fff0c0c72cf8b9649d3081a0784e4c00c7d",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/alias_local_operand",
    "candidate": "export function calculate(a,b){return a+b;}",
    "candidateSHA256": "ea9f5e374bf6defac3b62d8f55327b90b004f44ffcd878a8dcf9bab020b3faaf",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate as sum} from './candidate.mjs';test('source behavior',()=>{const observed=sum(2,3);assert.strictEqual(observed,5,\"calculate is only a word here\");});",
    "oracleSHA256": "542584e3888d18313f1416cff80651343d1cfbecb430d0966058dcc044d1dc3e",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": true
  },
  {
    "name": "independent/nested_operand_calls",
    "candidate": "export function calculate(a,b){return a+b;}",
    "candidateSHA256": "ea9f5e374bf6defac3b62d8f55327b90b004f44ffcd878a8dcf9bab020b3faaf",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source behavior',()=>{assert.equal(calculate(calculate(2,3),4),9);});",
    "oracleSHA256": "6eec5fb16f6bd97630b7446ff6f6e5a3dd6d54f0d2811c6b27e9450bb672122a",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": true
  },
  {
    "name": "independent/per_assertion_message_not_rescued",
    "candidate": "export function calculate(a,b){return a+b;}",
    "candidateSHA256": "ea9f5e374bf6defac3b62d8f55327b90b004f44ffcd878a8dcf9bab020b3faaf",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source behavior',()=>{assert.equal(5,5,calculate(2,3));assert.equal(calculate(2,3),5);});",
    "oracleSHA256": "4f5cc48bab64772db6f6ea0bb2ff0eb15b7b4e2dec7fc7ea17428c7ada071c3d",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/partial_export_diagnostic_only",
    "candidate": "export function calculate(a,b){return a+b;}export function other(a){return a;}",
    "candidateSHA256": "5b8a9fb50e4995bb466c35ab52a48ea4ddf9ee6470e776aa92821d640cd83a21",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate,other} from './candidate.mjs';test('source behavior',()=>{assert.equal(calculate(2,3),5,other(1));});",
    "oracleSHA256": "bdba31e60ff04204f3808c3ecfea4e398b526b49489d1b66e67acbdf48037f43",
    "interface": [
      "calculate",
      "other"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/unknown_diagnostic_refused",
    "candidate": "export function calculate(a,b){return a+b;}",
    "candidateSHA256": "ea9f5e374bf6defac3b62d8f55327b90b004f44ffcd878a8dcf9bab020b3faaf",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source behavior',()=>{assert.equal(calculate(2,3),5,process.exit(20));});",
    "oracleSHA256": "27545be84f01c6e98e872f66209af13f3be66088d1d2e73b659753269a30e947",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  },
  {
    "name": "independent/unknown_assertion_method",
    "candidate": "export function calculate(a,b){return a+b;}",
    "candidateSHA256": "ea9f5e374bf6defac3b62d8f55327b90b004f44ffcd878a8dcf9bab020b3faaf",
    "oracle": "import assert from 'node:assert/strict';import test from 'node:test';import {calculate} from './candidate.mjs';test('source behavior',()=>{assert.match(calculate(2,3),5);});",
    "oracleSHA256": "6302363167877564948bdf000b12c6cf8efb904548a4b65d23ae24bc86215f5a",
    "interface": [
      "calculate"
    ],
    "dependencies": [],
    "transport": "pure-esm",
    "mode": "none",
    "expected": false
  }
];

for(const fixture of fixtures){
 test(fixture.name,()=>{
  assert.equal(hash(fixture.candidate),fixture.candidateSHA256);
  assert.equal(hash(fixture.oracle),fixture.oracleSHA256);
  const directory=realpathSync(mkdtempSync(join(tmpdir(),'candidate-bound-oracle-')));
  try{
   const candidate=join(directory,'candidate.mjs'),oracle=join(directory,'oracle.test.mjs'),declaration=join(directory,'declaration.json');
   writeFileSync(candidate,fixture.candidate);writeFileSync(oracle,fixture.oracle);
   const dependencies=fixture.dependencies.map((row,index)=>{assert.equal(hash(row.source),row.sha256);const path=join(directory,'dependency-'+index+'.mjs');writeFileSync(path,row.source);return {path,sha256:row.sha256};});
   const contract={transport:fixture.transport,candidate:{path:candidate,sha256:fixture.candidateSHA256},oracle:{path:oracle,sha256:fixture.oracleSHA256},dependencies};
   writeFileSync(declaration,JSON.stringify({version:1,...contract,interface:fixture.interface}));contract.declaration={path:declaration,sha256:hash(JSON.stringify({version:1,...contract,interface:fixture.interface}))};
   // Bind the exact file actually written, before applying physical absence/drift controls.
   const declared={version:1,transport:fixture.transport,candidate:contract.candidate,oracle:contract.oracle,dependencies,interface:fixture.interface};
   writeFileSync(declaration,JSON.stringify(declared));contract.declaration.sha256=hash(JSON.stringify(declared));
   if(fixture.mode==='absent')unlinkSync(candidate);
   if(fixture.mode==='drift')writeFileSync(candidate,fixture.candidate+String.fromCharCode(10));
   const result=executeCandidateBoundOracle(contract);
   assert.equal(result.structuralComplete,fixture.expected);
   assert.equal(result.semanticAcceptance,'Unknown');
   if(result.review.complete){assert.equal(result.review.semanticUsefulness,'Unknown');assert.equal(result.review.validatedUsefulNetCodeBytes,null);}
   if(result.structuralComplete){assert.equal(result.providerUsage,'Unknown');assert.equal(result.cost,'Unknown');assert.equal(result.validatedUsefulNetCodeBytes,null);}
  }finally{rmSync(directory,{recursive:true,force:true});}
 });
}
