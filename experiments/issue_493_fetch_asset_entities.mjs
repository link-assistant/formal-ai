import { writeFileSync } from "node:fs";
const UA = "formal-ai-research/1.0 (link.assistant.team@proton.me)";
const ids = ["Q16783523", "Q131723"]; // Ethereum, Bitcoin
const url = new URL("https://www.wikidata.org/w/api.php");
url.searchParams.set("action", "wbgetentities");
url.searchParams.set("ids", ids.join("|"));
url.searchParams.set("format", "json");
url.searchParams.set("languages", "en|ru|hi|zh");
url.searchParams.set("props", "labels|descriptions|aliases|datatype");
const res = await fetch(url, { headers: { "User-Agent": UA } });
if (!res.ok) throw new Error(`HTTP ${res.status} ${res.statusText}`);
const json = await res.json();
for (const id of ids) {
  const entity = json.entities?.[id];
  if (!entity) throw new Error(`${id}: missing`);
  const single = { entities: { [id]: entity }, success: json.success ?? 1 };
  const dest = `data/cache/wikidata/entity/${id}.json`;
  writeFileSync(dest, JSON.stringify(single, null, 2) + "\n");
  console.log(`wrote ${dest}: ${entity.labels?.en?.value}`);
}
