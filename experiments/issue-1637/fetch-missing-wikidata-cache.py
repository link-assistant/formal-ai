#!/usr/bin/env python3
"""Fetch Wikidata cache records the semantic-grounding gate reports missing.

Usage: python3 -I experiments/issue-1637/fetch-missing-wikidata-cache.py ID:TOKEN...
Mirrors scripts/ground-meanings.rs (trim to en/ru/hi/zh/es, refuse an id whose
labels lack TOKEN) and encodes the .lino twin with scripts/wikidata_lino_codec.py.
"""
import json, subprocess, sys, urllib.request
from collections import OrderedDict
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts"))
from wikidata_lino_codec import json_cache_file  # noqa: E402

LANGS = ["en", "ru", "hi", "zh", "es"]
for spec in sys.argv[1:]:
    qid, token = spec.split(":", 1)
    kind = {"P": "property", "L": "lexeme"}.get(qid[0], "entity")
    out = Path("data/cache/wikidata") / kind
    url = f"https://www.wikidata.org/wiki/Special:EntityData/{qid}.json"
    req = urllib.request.Request(url, headers={"User-Agent": "formal-ai-grounding/1.0"})
    entity = json.load(urllib.request.urlopen(req, timeout=60))["entities"][qid]
    labels = " | ".join(v["value"] for v in entity.get("labels", {}).values()).lower()
    if token.lower() not in labels:
        sys.exit(f"{qid}: token {token!r} not in labels")
    keep = lambda s: OrderedDict((l, s[l]) for l in LANGS if l in s)
    trimmed = OrderedDict(type=entity["type"])
    if "datatype" in entity:
        trimmed["datatype"] = entity["datatype"]
    trimmed["id"] = entity["id"]
    for key in ("labels", "descriptions", "aliases"):
        if key in entity and keep(entity[key]):
            trimmed[key] = keep(entity[key])
    doc = OrderedDict(entities=OrderedDict([(qid, trimmed)]), success=1)
    (out / f"{qid}.json").write_text(json.dumps(doc, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (out / f"{qid}.lino").write_text(json_cache_file(qid, doc), encoding="utf-8")
    print(f"wrote {out}/{qid}.json/.lino ({entity['labels'].get('en', {}).get('value')})")
