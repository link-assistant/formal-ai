import { readFileSync } from "node:fs";
import { evaluate } from "./browser-runtime.mjs";

export const factFixtureRaw = Object.fromEntries(
  ["fact-captures.lino", "fact-realization.lino"].map((name) => [
    name, readFileSync(new URL(`../../fixtures/fact-claims/${name}`, import.meta.url), "utf8"),
  ]),
);

export async function loadHistoricalFactFixtures(worker) {
  await evaluate(worker, "loadSeed()");
  worker.factFixtureRaw = factFixtureRaw;
  evaluate(worker, "FACTS = FACTS.concat(derivedCaptureFacts(factFixtureRaw));");
}

export function wikidataFixture(url, subject) {
  const target = new URL(url);
  if (target.hostname !== "www.wikidata.org") return null;
  if (target.searchParams.get("action") === "wbsearchentities") {
    const label = target.searchParams.get("search");
    return { search: label ? [{ id: subject, label }] : [] };
  }
  const entity = target.searchParams.get("ids") || target.pathname.split("/").at(-1)?.split(".")[0];
  if (!/^Q[0-9]+$/u.test(entity)) return null;
  return JSON.parse(readFileSync(new URL(`../../fixtures/fact-claims/entities/${entity}.json`, import.meta.url), "utf8"));
}
