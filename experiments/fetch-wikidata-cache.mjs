import fs from "node:fs/promises";
import path from "node:path";

const root = process.cwd();
const cacheDir = path.join(root, "data/cache/wikidata");
const requestedIds = process.argv.slice(2);
const ids = requestedIds.length > 0 ? requestedIds : await discoverCacheIds();

function kindFor(id) {
  if (id.startsWith("L")) return "lexeme";
  if (id.startsWith("P")) return "property";
  if (id.startsWith("Q")) return "entity";
  throw new Error(`unknown Wikidata id ${id}`);
}

function apiUrl(id) {
  const url = new URL("https://www.wikidata.org/w/api.php");
  url.searchParams.set("action", "wbgetentities");
  url.searchParams.set("ids", id);
  url.searchParams.set("format", "json");
  url.searchParams.set("languages", "en|ru|hi|zh");
  if (!id.startsWith("L")) {
    url.searchParams.set("props", "labels|descriptions|aliases|datatype");
  }
  return url;
}

const pending = [];
for (const id of ids) {
  const destinationDir = path.join(cacheDir, kindFor(id));
  await fs.mkdir(destinationDir, { recursive: true });
  const destination = path.join(destinationDir, `${id}.json`);
  if (await exists(destination)) {
    console.log(`${id} -> ${path.relative(root, destination)} (cached)`);
    continue;
  }
  pending.push(id);
}

for (const chunk of chunks(
  pending.filter((id) => !id.startsWith("L")),
  40,
)) {
  const response = await fetchWithRetry(chunk.join("|"));
  const json = await response.json();
  for (const id of chunk) {
    const entity = json.entities?.[id];
    if (!entity) throw new Error(`${id}: missing entity in batch response`);
    const destination = path.join(cacheDir, kindFor(id), `${id}.json`);
    const single = { entities: { [id]: entity }, success: json.success ?? 1 };
    await fs.writeFile(destination, `${JSON.stringify(single, null, 2)}\n`);
    console.log(`${id} -> ${path.relative(root, destination)}`);
  }
  await sleep(1000);
}

for (const id of pending.filter((id) => id.startsWith("L"))) {
  const destination = path.join(cacheDir, kindFor(id), `${id}.json`);
  const response = await fetchWithRetry(id);
  const json = await response.json();
  await fs.writeFile(destination, `${JSON.stringify(json, null, 2)}\n`);
  console.log(`${id} -> ${path.relative(root, destination)}`);
  await sleep(250);
}

async function discoverCacheIds() {
  const ids = new Set();
  for (const name of await fs.readdir(cacheDir)) {
    if (/^[LPQ]\d+\.lino$/.test(name)) {
      ids.add(name.replace(/\.lino$/, ""));
    }
  }
  for (const directory of ["entity", "property", "lexeme"]) {
    const typedDir = path.join(cacheDir, directory);
    if (!(await exists(typedDir))) continue;
    for (const name of await fs.readdir(typedDir)) {
      if (/^[LPQ]\d+\.lino$/.test(name)) {
        ids.add(name.replace(/\.lino$/, ""));
      }
    }
  }
  return [...ids].sort((a, b) => a.localeCompare(b));
}

async function fetchWithRetry(id) {
  let delay = 1000;
  for (let attempt = 1; attempt <= 6; attempt += 1) {
    const response = await fetch(apiUrl(id), {
      headers: { "User-Agent": "formal-ai-issue-398-cache-refresh/1.0" },
    });
    if (response.ok) return response;
    if (response.status !== 429 || attempt === 6) {
      throw new Error(`${id}: ${response.status} ${response.statusText}`);
    }
    await sleep(delay);
    delay *= 2;
  }
  throw new Error(`${id}: exhausted retries`);
}

async function exists(file) {
  try {
    await fs.access(file);
    return true;
  } catch {
    return false;
  }
}

function sleep(ms) {
  return new Promise((resolve) => {
    setTimeout(resolve, ms);
  });
}

function chunks(values, size) {
  const out = [];
  for (let index = 0; index < values.length; index += size) {
    out.push(values.slice(index, index + size));
  }
  return out;
}
