// Does the browser mirror answer the request issue #723 reported the way the
// Rust engine now does?
//
// The worker ships as a list of shards loaded with `importScripts`, and it reads
// its vocabulary from the seed at init() time. This harness reproduces both:
// every shard is evaluated in one VM context in the declared order, then the
// `meanings*.lino` seed files are handed to `hydrateLinoSeedText` exactly as
// `loadSeed` does in the browser. Expressions are evaluated *inside* the context
// because the catalog tables are `const` — global lexical bindings, which are
// reachable from script code but are not properties of the sandbox object.
//
//   node experiments/issue-1021-laravel/worker_check.mjs
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const repo = path.resolve(import.meta.dirname, "../..");
const webRoot = path.join(repo, "src/web");
const moduleList = fs.readFileSync(path.join(webRoot, "worker-modules.js"), "utf8");
const shards = [...moduleList.matchAll(/"(worker\/[^"]*\.js)"/g)].map((match) => match[1]);
if (shards.length === 0) throw new Error("no worker shards found in worker-modules.js");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => {};
sandbox.postMessage = () => {};
sandbox.addEventListener = () => {};
sandbox.setTimeout = setTimeout;
sandbox.clearTimeout = clearTimeout;
sandbox.fetch = async () => {
  throw new Error("the harness serves the seed from disk, not over the network");
};
sandbox.location = { search: "", origin: "http://localhost", href: "http://localhost/" };
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
sandbox.performance = performance;
vm.createContext(sandbox);
for (const shard of shards) {
  vm.runInContext(fs.readFileSync(path.join(webRoot, shard), "utf8"), sandbox, { filename: shard });
}

const seedDir = path.join(repo, "data/seed");
const raw = {};
for (const file of fs.readdirSync(seedDir)) {
  if (file.endsWith(".lino")) raw[`seed/${file}`] = fs.readFileSync(path.join(seedDir, file), "utf8");
}
sandbox.__seedRaw = raw;
vm.runInContext("hydrateLinoSeedText(__seedRaw)", sandbox);

const failures = [];
let checked = 0;
function check(name, expression, expected) {
  checked += 1;
  const actual = vm.runInContext(expression, sandbox);
  const ok = actual === expected;
  console.log(`${ok ? "PASS" : "FAIL"}: ${name} -> ${JSON.stringify(actual)}`);
  if (!ok) failures.push(`${name}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
}

const resolve = (prompt) =>
  `programLanguageFromPrompt(normalizeProgramPrompt(${JSON.stringify(prompt)}))`;

// The request issue #723 reported, and the three languages it is reported in.
check("reported ru prompt", resolve("напиши мне код на PHP Laravel"), "laravel");
check("reported en prompt", resolve("write me PHP Laravel code"), "laravel");
check("reported hi prompt", resolve("PHP Laravel में कोड लिखें"), "laravel");
check("reported zh prompt", resolve("用 PHP Laravel 写代码"), "laravel");
// A framework named alone still resolves; naming the language alone is unchanged.
check("bare framework", resolve("write me Laravel code"), "laravel");
check("bare language", resolve("write me some PHP code"), "php");
// An uncatalogued framework falls back to the language it is written in.
check("uncatalogued framework", resolve("write me PHP Symfony code"), "php");
check("unrelated language", resolve("write me Rust code"), "rust");

// The fields the framework owns are the ones the request actually asked for.
check("laravel is a framework of php", "WRITE_PROGRAM_LANGUAGES.laravel.frameworkOf", "php");
check(
  "laravel saves where Artisan looks",
  "WRITE_PROGRAM_LANGUAGES.laravel.saveAs",
  "app/Console/Commands/HelloWorld.php",
);
check("laravel runs its own command", "WRITE_PROGRAM_LANGUAGES.laravel.runCommand", "php artisan hello:world");
check("laravel borrows the php fence", "WRITE_PROGRAM_LANGUAGES.laravel.fence", "php");
check(
  "laravel hello_world template is mirrored",
  "typeof WRITE_PROGRAM_TEMPLATES.hello_world.laravel",
  "string",
);
check(
  "the mirrored template is the Artisan command",
  "WRITE_PROGRAM_TEMPLATES.hello_world.laravel.includes('php artisan')",
  false,
);
check(
  "the mirrored template prints the greeting",
  "WRITE_PROGRAM_TEMPLATES.hello_world.laravel.includes('Hello, world!')",
  true,
);

// What the mirror does *not* carry, recorded rather than assumed: the browser
// answers eleven tasks against the engine's twelve, and the copy-stdin task is
// the one it is missing (case study finding 21).
check(
  "the mirror does not carry the stdin task",
  "Object.prototype.hasOwnProperty.call(WRITE_PROGRAM_TASKS, 'copy_stdin_to_stdout')",
  false,
);
check("the mirror carries eleven tasks", "Object.keys(WRITE_PROGRAM_TASKS).length", 11);

if (failures.length > 0) {
  console.error(`\n${failures.length} failure(s):\n  ${failures.join("\n  ")}`);
  process.exit(1);
}
console.log(`\nAll ${checked} worker checks passed across ${shards.length} shards.`);
