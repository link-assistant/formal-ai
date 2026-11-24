#!/usr/bin/env python3
"""Regenerate the historical film-timeline test fixture (issue #892).

The raw SPARQL response and its lossless LiNo twin are kept under
`rust/tests/fixtures/release-timeline/historical`. The renderer regression
replays the film dates and localized labels from captures stored together. Runtime timelines acquire publisher data
through the verified source client.

Run `python3 scripts/ground-release-timelines.py --check` for the offline check.
`--refresh` fetches a new test capture and updates the reviewed comparison data.
"""
from __future__ import annotations

import hashlib
import json
import sys
import time
import urllib.parse
import urllib.request
from pathlib import Path

from wikidata_lino_codec import json_cache_file

USER_AGENT = "formal-ai-grounding/1.0 (https://github.com/link-assistant/formal-ai)"
FIXTURE = Path("rust/tests/fixtures/release-timeline/spider-man-replay.lino")
LANGUAGE_REGISTRY = Path("data/seed/languages.lino")
FETCH_ATTEMPTS = 40
FETCH_BACKOFF_SECONDS = 15


def supported_languages() -> list[str]:
    """Return the registered language codes, so the registry stays the interface."""
    codes = []
    for line in LANGUAGE_REGISTRY.read_text(encoding="utf-8").split("\n"):
        stripped = line.strip()
        if stripped.startswith("language "):
            codes.append(stripped.split(" ", 1)[1].strip())
    return codes


LANGUAGES = supported_languages()


def get(url: str, accept: str = "application/json") -> bytes:
    """Fetch `url`, retrying while the shared runner IP is rate limited."""
    request = urllib.request.Request(
        url, headers={"User-Agent": USER_AGENT, "Accept": accept}
    )
    last_error: Exception | None = None
    for attempt in range(FETCH_ATTEMPTS):
        try:
            with urllib.request.urlopen(request) as response:
                return response.read()
        except Exception as error:  # noqa: BLE001 - retry every transport error
            last_error = error
            if attempt + 1 < FETCH_ATTEMPTS:
                time.sleep(FETCH_BACKOFF_SECONDS)
    raise RuntimeError(f"failed to fetch {url}: {last_error}")


def encode_lino(root_id: str, json_path: Path, lino_path: Path) -> None:
    """Write the canonical LiNo twin with the codec the whole cache is built with."""
    document = json.loads(json_path.read_text(encoding="utf-8"))
    lino_path.write_text(json_cache_file(root_id, document), encoding="utf-8")


def write_json(path: Path, document: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(document, handle, ensure_ascii=False, indent=2)
        handle.write("\n")


def fetch_query(source_url: str, query_path: Path, cache_path: Path, slug: str) -> None:
    """Refresh the query snapshot and its LiNo twin from the live endpoint."""
    query = query_path.read_text(encoding="utf-8")
    url = f"{source_url}?{urllib.parse.urlencode({'query': query})}"
    raw = get(url, accept="application/sparql-results+json")
    write_json(cache_path, json.loads(raw))
    encode_lino(slug, cache_path, cache_path.with_suffix(".lino"))


def fetch_entity(qid: str, entity_dir: Path, *, offline: bool) -> dict:
    """Return the cached entity document, fetching and trimming it when absent."""
    json_path = entity_dir / f"{qid}.json"
    if not offline and not json_path.exists():
        url = f"https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"
        entity = json.loads(get(url))["entities"][qid]
        trimmed = {
            "type": entity.get("type", "item"),
            "id": qid,
            "labels": {
                language: entity["labels"][language]
                for language in LANGUAGES
                if language in entity.get("labels", {})
            },
            "descriptions": {
                language: entity["descriptions"][language]
                for language in LANGUAGES
                if language in entity.get("descriptions", {})
            },
            "aliases": {
                language: entity["aliases"][language]
                for language in LANGUAGES
                if language in entity.get("aliases", {})
            },
        }
        write_json(json_path, {"entities": {qid: trimmed}, "success": 1})
    lino_path = json_path.with_suffix(".lino")
    if not offline and not lino_path.exists():
        encode_lino(qid, json_path, lino_path)
    return json.loads(json_path.read_text(encoding="utf-8"))["entities"][qid]


def query_rows(cache_path: Path) -> list[tuple[str, str]]:
    """Return `(Q-id, release date)` pairs from a cached SPARQL answer."""
    document = json.loads(cache_path.read_text(encoding="utf-8"))
    rows = []
    for binding in document["results"]["bindings"]:
        qid = binding["film"]["value"].rsplit("/", 1)[1]
        date = binding.get("firstRelease", {}).get("value", "")
        rows.append((qid, date.split("T", 1)[0]))
    rows.sort(key=lambda row: (row[1] == "", row[1], row[0]))
    return rows


def today_utc() -> str:
    return time.strftime("%Y-%m-%d", time.gmtime())


def field(lines: list[str], key: str) -> str:
    for line in lines:
        stripped = line.strip()
        if stripped.startswith(f"{key} "):
            return stripped[len(key) + 1 :].strip().strip('"')
    raise SystemExit(f"{FIXTURE}: timeline is missing `{key}`")


def quote(text: str) -> str:
    """Quote a value the Links Notation way: a delimiter inside is doubled."""
    return '"' + text.replace('"', '""') + '"'


def render_entries(
    rows: list[tuple[str, str]], indent: str, entity_dir: Path, *, offline: bool
) -> list[str]:
    lines = []
    for qid, date in rows:
        entity = fetch_entity(qid, entity_dir, offline=offline)
        lines.append(f"{indent}entry {qid}")
        if date:
            lines.append(f"{indent}  release-date {date}")
        for language in LANGUAGES:
            title = entity.get("labels", {}).get(language, {}).get("value")
            if title:
                lines.append(f"{indent}  localized {language}")
                lines.append(f"{indent}    title {quote(title)}")
    return lines


def render(text: str, refresh: bool, *, offline: bool) -> str:
    """Rewrite every timeline's provenance stamp and entry blocks from the cache."""
    lines = text.split("\n")
    output: list[str] = []
    index = 0
    while index < len(lines):
        line = lines[index]
        if not line.startswith("  timeline "):
            output.append(line)
            index += 1
            continue

        start = index
        index += 1
        while index < len(lines) and lines[index].startswith("    "):
            index += 1
        block = lines[start:index]
        output.extend(render_timeline(block, refresh, offline=offline))

    return "\n".join(output)


def render_timeline(block: list[str], refresh: bool, *, offline: bool) -> list[str]:
    slug = block[0].strip().split(" ", 1)[1]
    cache_path = Path(field(block, "cache-file"))
    query_path = Path(field(block, "query-file"))
    if not offline and (refresh or not cache_path.exists()):
        fetch_query(field(block, "source-url"), query_path, cache_path, slug)
    digest = hashlib.sha256(cache_path.read_bytes()).hexdigest()
    retrieved = today_utc() if refresh else field(block, "retrieved-at")

    header = []
    for line in block:
        stripped = line.strip()
        # Entries are always regenerated, and always trail the header fields.
        if line.startswith("    entry "):
            break
        if stripped.startswith("sha256 "):
            header.append(f'    sha256 "{digest}"')
            continue
        if stripped.startswith("retrieved-at "):
            header.append(f"    retrieved-at {retrieved}")
            continue
        header.append(line)
    while header and not header[-1].strip():
        header.pop()
    entity_dir = cache_path.with_name("wikidata-entities")
    return header + render_entries(query_rows(cache_path), "    ", entity_dir, offline=offline)


def main(argv: list[str]) -> int:
    check = "--check" in argv[1:]
    refresh = "--refresh" in argv[1:]
    if check and refresh:
        print("--check and --refresh are mutually exclusive", file=sys.stderr)
        return 2
    text = FIXTURE.read_text(encoding="utf-8")
    try:
        rendered = render(text, refresh, offline=check)
    except FileNotFoundError as error:
        print(f"Missing release timeline capture: {error.filename}", file=sys.stderr)
        return 1
    if rendered == text:
        print(f"{FIXTURE}: already grounded in the checked-in Wikidata cache")
        return 0
    if check:
        print(
            f"{FIXTURE} does not match its Wikidata cache; run "
            "python3 scripts/ground-release-timelines.py",
            file=sys.stderr,
        )
        return 1
    FIXTURE.write_text(rendered, encoding="utf-8")
    print(f"{FIXTURE}: rewrote the timelines from the Wikidata cache")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
