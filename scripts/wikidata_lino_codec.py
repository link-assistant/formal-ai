"""Python port of `json_cache_file` in rust/src/json_lino.rs (issue #1172 R9).

The Wikidata cache stores every snapshot twice: the trimmed `.json` and its
canonical Links Notation encoding, which `wikidata_lino_cache_rebuilds_full_json_losslessly`
(rust/tests/unit/data_files.rs) checks byte for byte. The Rust codec runs as
the `wikidata_json_to_lino` cargo example; this port lets a grounding script
write the `.lino` without a compile. `python3 scripts/wikidata_lino_codec.py
--verify` re-encodes every committed cache `.json` and compares it with the
committed `.lino`, so the port is checked against the real codec's output.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ARRAY_ENTRY = "entry"
ENTITIES_KEY = "entities"
_NUMBER = re.compile(r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?(?:[eE][+-]?[0-9]+)?")


def strip_empty(value):
    """`strip_empty`: drop null, empty arrays and empty objects recursively."""
    if value is None:
        return None
    if isinstance(value, list):
        kept = [item for item in (strip_empty(item) for item in value) if item is not None]
        return kept or None
    if isinstance(value, dict):
        kept = {}
        for key, item in value.items():
            stripped = strip_empty(item)
            if stripped is not None:
                kept[key] = stripped
        return kept or None
    return value


def _is_scalar(value) -> bool:
    return value is None or isinstance(value, (bool, int, float, str))


def _is_bare_reference(value: str) -> bool:
    return bool(value) and all(
        (character.isascii() and character.isalnum()) or character in "-_.$" for character in value
    )


def _is_scalar_literal(value: str) -> bool:
    return value in ("null", "true", "false") or _NUMBER.fullmatch(value.strip(" \t\n\r")) is not None


def _quoted_string(value: str) -> str:
    if '"' in value:
        escaped = (
            value.replace("\\", "\\\\")
            .replace("\n", "\\n")
            .replace("\r", "\\r")
            .replace("\t", "\\t")
            .replace("'", "''")
        )
        return f"'{escaped}'"
    return json.dumps(value, ensure_ascii=False)


def string_token(value: str) -> str:
    if _is_bare_reference(value) and not _is_scalar_literal(value):
        return value
    return _quoted_string(value)


def _number_token(value) -> str:
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    return json.dumps(value)


def scalar_token(value) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return _number_token(value)
    if isinstance(value, str):
        return string_token(value)
    return ""


def _scalar_array_token(values) -> str:
    return "(" + " ".join(scalar_token(value) for value in values) + ")"


def _write_line(out: list, indent: int, name: str, value=None) -> None:
    out.append(" " * indent + name + ("" if value is None else " " + value) + "\n")


def _sorted_items(mapping: dict):
    # serde_json's default `Map` is a BTreeMap: keys in byte order.
    return sorted(mapping.items(), key=lambda pair: pair[0].encode("utf-8"))


def _write_array_element(out, indent, value):
    if isinstance(value, list) and all(_is_scalar(item) for item in value):
        _write_line(out, indent, ARRAY_ENTRY, _scalar_array_token(value))
    elif isinstance(value, list):
        _write_line(out, indent, ARRAY_ENTRY)
        for item in value:
            _write_array_element(out, indent + 2, item)
    elif isinstance(value, dict):
        _write_line(out, indent, ARRAY_ENTRY)
        _write_object_entries(out, indent + 2, value)
    else:
        _write_line(out, indent, ARRAY_ENTRY, scalar_token(value))


def _write_named_value(out, indent, name, value):
    if isinstance(value, list) and all(_is_scalar(item) for item in value):
        _write_line(out, indent, name, _scalar_array_token(value))
    elif isinstance(value, list):
        _write_line(out, indent, name)
        for item in value:
            _write_array_element(out, indent + 2, item)
    elif isinstance(value, dict):
        _write_line(out, indent, name)
        _write_object_entries(out, indent + 2, value)
    else:
        _write_line(out, indent, name, scalar_token(value))


def _write_object_entries(out, indent, mapping):
    for key, value in _sorted_items(mapping):
        _write_named_value(out, indent, key, value)


def _language_text(entry):
    if isinstance(entry, dict) and isinstance(entry.get("value"), str):
        return entry["value"]
    return None


def _write_language_strings(out, indent, name, value):
    if not isinstance(value, dict):
        _write_named_value(out, indent, name, value)
        return
    _write_line(out, indent, name)
    for language, entry in _sorted_items(value):
        text = _language_text(entry)
        if text is not None:
            _write_line(out, indent + 2, language, string_token(text))


def _write_language_aliases(out, indent, name, value):
    if not isinstance(value, dict):
        _write_named_value(out, indent, name, value)
        return
    _write_line(out, indent, name)
    for language, entries in _sorted_items(value):
        if not isinstance(entries, list):
            continue
        tokens = [string_token(text) for text in (_language_text(entry) for entry in entries) if text is not None]
        if tokens:
            _write_line(out, indent + 2, language, "(" + " ".join(tokens) + ")")


def _write_entity_fields(out, indent, entity):
    for key, value in _sorted_items(entity):
        if key in ("labels", "descriptions", "lemmas"):
            _write_language_strings(out, indent, key, value)
        elif key == "aliases":
            _write_language_aliases(out, indent, key, value)
        else:
            _write_named_value(out, indent, key, value)


def _write_document(out, value):
    if isinstance(value, dict):
        _write_object_entries(out, 0, value)
    elif isinstance(value, list):
        for item in value:
            _write_array_element(out, 0, item)
    else:
        _write_line(out, 0, "value", scalar_token(value))


def json_cache_file(root_id: str, value) -> str:
    """`json_cache_file`: the canonical Links Notation of a cached document."""
    value = strip_empty(value)
    if value is None:
        value = {}
    out: list = []
    entities = value.get(ENTITIES_KEY) if isinstance(value, dict) else None
    entity = entities.get(root_id) if isinstance(entities, dict) else None
    if isinstance(entity, dict):
        _write_line(out, 0, root_id)
        _write_entity_fields(out, 2, entity)
        for key, field in _sorted_items(value):
            if key != ENTITIES_KEY:
                _write_named_value(out, 0, key, field)
    else:
        _write_document(out, value)
    return "".join(out)


def _verify(cache_dir: Path) -> int:
    checked = mismatched = 0
    for lino_path in sorted(cache_dir.rglob("*.lino")):
        json_path = lino_path.with_suffix(".json")
        if not json_path.is_file():
            continue
        document = json.loads(json_path.read_text(encoding="utf-8"))
        encoded = json_cache_file(lino_path.stem, document)
        checked += 1
        if encoded != lino_path.read_text(encoding="utf-8"):
            mismatched += 1
            print(f"mismatch: {lino_path}")
    print(f"codec port: {checked} cache files re-encoded, {mismatched} differ")
    return 1 if mismatched else 0


if __name__ == "__main__":
    if sys.argv[1:] == ["--verify"]:
        sys.exit(_verify(Path("data/cache/wikidata")))
    print("usage: python3 scripts/wikidata_lino_codec.py --verify", file=sys.stderr)
    sys.exit(2)
