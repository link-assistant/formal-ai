"""Measure the cache gate's references and identify surviving test/view consumers."""

from pathlib import Path
import json
import re

BUCKETS = (
    Path("data/cache/wikidata/entity"),
    Path("data/cache/wordnet/en"),
    Path("data/cache/wiktionary/en"),
)
REFERENCE_ROOTS = ("data/seed", "data/cache", "data/overrides", "rust/src", "js/source-cache")
REFERENCE_EXTENSIONS = {".lino", ".rs", ".json", ".rq"}
TOKENS = re.compile(rb"[A-Za-z0-9-]+")


def main():
    records = {
        bucket: {path.stem.encode() for path in bucket.iterdir()
                 if path.is_file() and not path.stem.startswith(".") and path.name != "README.md"}
        for bucket in BUCKETS
    }
    unreferenced = set().union(*records.values())
    for root in REFERENCE_ROOTS:
        for path in Path(root).rglob("*"):
            if path.suffix in REFERENCE_EXTENSIONS and path.is_file():
                own_stem = path.stem.encode()
                for token in TOKENS.finditer(path.read_bytes()):
                    if token[0] != own_stem:
                        unreferenced.discard(token[0])

    entity_references = {stem: [] for stem in records[BUCKETS[0]] & unreferenced}
    for path in Path("rust/tests/fixtures").rglob("*"):
        if path.suffix not in REFERENCE_EXTENSIONS or not path.is_file():
            continue
        entity_ids = set(re.findall(rb"\bQ[0-9]+\b", path.read_bytes()))
        for stem in entity_ids & entity_references.keys():
            entity_references[stem].append(str(path))

    report = []
    for bucket, stems in records.items():
        for stem in sorted(stems & unreferenced):
            name = stem.decode()
            source = [str(path) for path in sorted(bucket.glob(name + ".*"))]
            view = Path(f"data/view/en/{name}.lino")
            report.append({
                "record": str(bucket / name),
                "source": source,
                "committedView": str(view) if view.is_file() else None,
                "testReferences": sorted(entity_references.get(stem, [])),
            })
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
