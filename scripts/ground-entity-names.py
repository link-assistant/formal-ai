#!/usr/bin/env python3
"""Ground `data/seed/entity-names.lino` in the Wikidata cache (issue #699).

Batch 2 of the handler migration replaced the `who_is` handler's hardcoded table
of eight people (and their hand-written typos) with nearest-surface search over
remembered names. The names themselves live in `data/seed/entity-names.lino`,
and this migration keeps that file *derived* rather than hand-typed: every
surface is the checked-in Wikidata label for the entity's `grounded-in` id, in
each supported response language — or, when an entity has no label in a
language, that language's Wikipedia sitelink title, which is the name the
language's own edition uses.

For every `entity <slug> / grounded-in <Q-id>` record it:

  1. Fetches `Special:EntityData/<Q-id>.json` when the cache lacks it or the
     cache predates the current grounding shape, trims the document to the
     labels/descriptions/aliases of the supported languages, their sitelink
     titles, and the country (P17) and ISO 3166 (P297) claims — the same shape
     the checked-in entity records already use — and writes
     `data/cache/wikidata/entity/<Q-id>.json`.
  2. Generates the lossless `.lino` snapshot with the `wikidata_json_to_lino`
     cargo example, the same codec the rest of the cache is built with.
  3. Rewrites the record's `lexeme`/`surface`/`text` blocks from the cached
     labels, so the seed cannot drift from its grounding and no spelling in it
     is a human guess. An entity that resolves to a single IANA zone through
     its country and the tz database's `zone1970.tab` also projects a
     `timezone <zone>` field, which is how a scheduled time anchors to a place
     (plan 10 leaf 16, issue #869) without any hand-typed zone table.

Run `python3 scripts/ground-entity-names.py` (add `--check` to fail instead of
writing, which is what CI wants). Network is only needed the first time an id
is fetched, or when the grounding shape changes.
"""
from __future__ import annotations

import json
import subprocess
import sys
from pathlib import Path

USER_AGENT = "formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)"
LANGUAGES = ["en", "ru", "hi", "zh", "es"]
COUNTRY_CLAIM = "P17"
ISO_CODE_CLAIM = "P297"
ZONE_TAB = Path("data/cache/iana/zone1970.tab")
# The tz database's own country-to-zone table (public domain), from the
# repository IANA releases tzdb from. A place's IANA zone is read from here —
# Wikidata items for countries and cities carry no zone string of their own.
ZONE_TAB_URL = "https://raw.githubusercontent.com/eggert/tz/main/zone1970.tab"
# Recorded in the cache wrapper so a document trimmed by an older shape of this
# script is refetched instead of silently missing languages or claims.
GROUNDING = {
    "languages": LANGUAGES,
    "claims": [COUNTRY_CLAIM, ISO_CODE_CLAIM],
    "claim_ranks": True,
    "zone_tab": ZONE_TAB.name,
}
SEED = Path("data/seed/entity-names.lino")
CACHE_DIR = Path("data/cache/wikidata/entity")


def trim(entity: dict, qid: str) -> dict:
    """Reduce a fetched entity to the sections the seed derives from."""
    def per_language(section: str) -> dict:
        return {
            lang: entity[section][lang]
            for lang in LANGUAGES
            if lang in entity.get(section, {})
        }

    def mainsnaks(claim_id: str) -> list:
        # Rank travels with the snak: a place's country list mixes current and
        # historical states, and only Wikidata's preferred rank says which one
        # is the country today.
        return [
            {"mainsnak": claim["mainsnak"], "rank": claim.get("rank", "normal")}
            for claim in entity.get("claims", {}).get(claim_id, [])
            if claim.get("mainsnak", {}).get("snaktype") == "value"
            and claim.get("rank") != "deprecated"
        ]

    trimmed = {
        "type": entity.get("type", "item"),
        "id": qid,
        "labels": per_language("labels"),
        "descriptions": per_language("descriptions"),
        "aliases": per_language("aliases"),
        "sitelinks": {
            f"{lang}wiki": {"site": f"{lang}wiki", "title": link["title"]}
            for lang in LANGUAGES
            for link in [entity.get("sitelinks", {}).get(f"{lang}wiki")]
            if link
        },
    }
    claims = {
        claim_id: mainsnaks(claim_id)
        for claim_id in (COUNTRY_CLAIM, ISO_CODE_CLAIM)
        if mainsnaks(claim_id)
    }
    if claims:
        trimmed["claims"] = claims
    return trimmed


def fetch_entity(qid: str) -> dict:
    """Return the cached entity document, fetching and trimming it when absent
    or trimmed by an older grounding shape."""
    json_path = CACHE_DIR / f"{qid}.json"
    doc = None
    if json_path.exists():
        doc = json.loads(json_path.read_text(encoding="utf-8"))
    if doc is None or doc.get("grounding") != GROUNDING:
        url = f"https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"
        raw = subprocess.run(
            ["curl", "-sfL", "-A", USER_AGENT, url], capture_output=True, check=True
        ).stdout
        entity = json.loads(raw)["entities"][qid]
        doc = {
            "entities": {qid: trim(entity, qid)},
            "success": 1,
            "grounding": GROUNDING,
        }
        CACHE_DIR.mkdir(parents=True, exist_ok=True)
        with json_path.open("w", encoding="utf-8") as handle:
            json.dump(doc, handle, ensure_ascii=False, indent=2)
            handle.write("\n")
        subprocess.run(
            ["cargo", "run", "--quiet", "--example", "wikidata_json_to_lino",
             "--", qid, str(json_path), str(CACHE_DIR / f"{qid}.lino")],
            check=True,
        )
    lino_path = CACHE_DIR / f"{qid}.lino"
    if not lino_path.exists():
        subprocess.run(
            ["cargo", "run", "--quiet", "--example", "wikidata_json_to_lino",
             "--", qid, str(json_path), str(lino_path)],
            check=True,
        )
    return doc["entities"][qid]


def label_for(entity: dict, language: str) -> str | None:
    """The entity's name in `language`: its Wikidata label, or the title its
    language's Wikipedia edition uses when no label exists."""
    label = entity.get("labels", {}).get(language, {}).get("value")
    if label:
        return label
    return entity.get("sitelinks", {}).get(f"{language}wiki", {}).get("title")


def timezone_value(entity: dict) -> str | None:
    """The entity's IANA time zone, resolved through the tz database.

    Wikidata grounds the chain place → country (P17, preferred rank first) →
    ISO 3166 code (P297); the tz database's country-to-zone table names the
    zone. A country with several zones yields the row whose comment names the
    place itself (an exclave like Büsingen), or the one row covering "most of"
    the country — and when neither applies, the place has no single grounded
    zone and the seed says so by projecting no `timezone` field at all.
    """
    codes: list[str] = []
    own_code = iso_code_of(entity)
    if own_code:
        codes.append(own_code)
    else:
        for country in item_ids(entity, COUNTRY_CLAIM):
            country_code = iso_code_of(fetch_entity(country))
            if country_code:
                codes.append(country_code)
    if not codes:
        return None
    label = (entity.get("labels", {}).get("en", {}) or {}).get("value", "")
    normalized_label = label.casefold().replace(" ", "")
    for code in codes:
        rows = load_zone_tab().get(code, [])
        if len(rows) == 1:
            return rows[0][0]
        for zone, comment in rows:
            for segment in comment.replace(";", ",").split(","):
                if segment.strip() and normalized_label == segment.strip().casefold().replace(" ", ""):
                    return zone
        # A country split into several zones still names the one that covers
        # "most of" it; a place that is not itself a row's named city lies in
        # that zone unless the table says otherwise.
        covering = [zone for zone, comment in rows if comment.strip().casefold().startswith("most of")]
        if len(covering) == 1:
            return covering[0]
    return None


def iso_code_of(entity: dict) -> str | None:
    for value in string_values(entity, ISO_CODE_CLAIM):
        return value
    return None


def ranked_claims(entity: dict, claim_id: str) -> list:
    """A claim's mainsnaks with preferred-rank statements first."""
    claims = entity.get("claims", {}).get(claim_id, [])
    return sorted(claims, key=lambda claim: claim.get("rank") != "preferred")


def item_ids(entity: dict, claim_id: str) -> list[str]:
    ids = []
    for claim in ranked_claims(entity, claim_id):
        datavalue = claim.get("mainsnak", {}).get("datavalue", {})
        value = datavalue.get("value")
        if isinstance(value, dict) and value.get("id"):
            ids.append(value["id"])
    return ids


def string_values(entity: dict, claim_id: str) -> list[str]:
    values = []
    for claim in ranked_claims(entity, claim_id):
        datavalue = claim.get("mainsnak", {}).get("datavalue", {})
        value = datavalue.get("value")
        if isinstance(value, str) and value:
            values.append(value)
    return values


def load_zone_tab() -> dict[str, list[tuple[str, str]]]:
    """The tz database country-to-zone table, fetched once into the cache."""
    if not ZONE_TAB.exists():
        raw = subprocess.run(
            ["curl", "-sfL", "-A", USER_AGENT, ZONE_TAB_URL],
            capture_output=True, check=True,
        ).stdout
        ZONE_TAB.parent.mkdir(parents=True, exist_ok=True)
        ZONE_TAB.write_bytes(raw)
    zones: dict[str, list[tuple[str, str]]] = {}
    for line in ZONE_TAB.read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        if len(fields) < 3:
            continue
        for code in fields[0].split(","):
            zones.setdefault(code, []).append((fields[2], fields[3] if len(fields) > 3 else ""))
    return zones


def render(records: list[tuple[str, str, dict]]) -> str:
    """Render the whole registry from the cached labels."""
    lines = ["entity_names"]
    for slug, qid, entity in records:
        lines.append(f"  entity {slug}")
        lines.append(f"    grounded-in {qid}")
        zone = timezone_value(entity)
        if zone:
            lines.append(f'    timezone "{zone}"')
        for language in LANGUAGES:
            label = label_for(entity, language)
            if not label:
                continue
            lines.append(f"    lexeme {language}")
            lines.append("      surface")
            lines.append(f'        text "{label}"')
    return "\n".join(lines) + "\n"


def parse_records(text: str) -> list[tuple[str, str]]:
    records = []
    slug = None
    for line in text.split("\n"):
        stripped = line.strip()
        if stripped.startswith("entity ") and line.startswith("  entity "):
            slug = stripped.split(" ", 1)[1]
        elif stripped.startswith("grounded-in ") and slug:
            records.append((slug, stripped.split(" ", 1)[1]))
            slug = None
    return records


def main(argv: list[str]) -> int:
    check = "--check" in argv[1:]
    text = SEED.read_text(encoding="utf-8")
    records = [(slug, qid, fetch_entity(qid)) for slug, qid in parse_records(text)]
    rendered = render(records)
    if rendered == text:
        print(f"{SEED}: {len(records)} entities, already grounded")
        return 0
    if check:
        print(f"{SEED} does not match its Wikidata grounding; run "
              "python3 scripts/ground-entity-names.py", file=sys.stderr)
        return 1
    SEED.write_text(rendered, encoding="utf-8")
    print(f"{SEED}: rewrote {len(records)} entities from the Wikidata cache")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
