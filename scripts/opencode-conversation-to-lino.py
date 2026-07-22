#!/usr/bin/env python3
"""Extract one complete OpenCode SQLite session as Links Notation.

The database is opened read-only, uses only Python's standard library, and
orders messages and parts by ``(time_created, id)`` for deterministic output.
"""

from __future__ import annotations

import argparse
import base64
import json
import os
import sqlite3
import sys
from typing import Any

INDENT = "  "
BARE_SAFE = set(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_.-/"
)
SEQUENCE_ITEM_NAME = {"messages": "message", "parts": "part"}


def flatten(value: str) -> str:
    """Keep a scalar on one physical Links Notation line."""
    return value.replace("\r", "\\r").replace("\n", "\\n").replace("\t", "\\t")


def quote(value: str) -> str:
    """Select a safe delimiter, or use tagged base64 when both occur."""
    flat = flatten(value)
    if '"' not in flat:
        return f'"{flat}"'
    if "'" not in flat:
        return f"'{flat}'"
    encoded = base64.b64encode(value.encode("utf-8")).decode("ascii")
    return f'"b64:{encoded}"'


def scalar(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, (int, float)):
        return repr(value)
    text = str(value)
    if text and all(character in BARE_SAFE for character in text):
        return text
    return quote(text)


def item_name(key: str) -> str:
    if key in SEQUENCE_ITEM_NAME:
        return SEQUENCE_ITEM_NAME[key]
    if key.endswith("ies") and len(key) > 3:
        return f"{key[:-3]}y"
    if key.endswith("s") and len(key) > 1:
        return key[:-1]
    return f"{key}_item"


def emit(node: Any, key: str | None, depth: int, output: list[str]) -> None:
    """Render dictionaries, scalar lists, and native repeated-key sequences."""
    pad = INDENT * depth
    if isinstance(node, dict):
        if key is not None:
            output.append(f"{pad}{key}")
        child_depth = depth + 1 if key is not None else depth
        for child_key in sorted(node):
            emit(node[child_key], str(child_key), child_depth, output)
        return

    if isinstance(node, list):
        heading = key if key is not None else "list"
        if all(not isinstance(item, (dict, list)) for item in node):
            inline = " ".join(scalar(item) for item in node)
            output.append(f"{pad}{heading} ({inline})")
            return
        output.append(f"{pad}{heading}")
        singular = item_name(heading)
        for item in node:
            emit(item, singular, depth + 1, output)
        return

    if key is None:
        output.append(f"{pad}{scalar(node)}")
    else:
        output.append(f"{pad}{key} {scalar(node)}")


def default_db_path() -> str:
    data_home = os.environ.get("XDG_DATA_HOME")
    base = data_home if data_home else os.path.expanduser("~/.local/share")
    return os.path.join(base, "opencode", "opencode.db")


def open_ro(db_path: str) -> sqlite3.Connection:
    if not os.path.exists(db_path):
        raise FileNotFoundError(f"opencode database not found: {db_path}")
    connection = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    connection.row_factory = sqlite3.Row
    return connection


def decode_json(blob: Any) -> Any:
    if blob is None:
        return None
    try:
        return json.loads(blob)
    except (TypeError, ValueError):
        return {"_raw": str(blob)}


def fetch_session(connection: sqlite3.Connection, session_id: str) -> dict[str, Any]:
    row = connection.execute(
        "SELECT * FROM session WHERE id = ?", (session_id,)
    ).fetchone()
    if row is None:
        raise LookupError(f"session not found: {session_id}")
    session = dict(row)
    for name in ("model", "metadata"):
        value = session.get(name)
        if isinstance(value, str) and value.strip().startswith(("{", "[")):
            session[name] = decode_json(value)
    return session


def fetch_messages(
    connection: sqlite3.Connection, session_id: str
) -> list[dict[str, Any]]:
    rows = connection.execute(
        "SELECT id, time_created, time_updated, data FROM message "
        "WHERE session_id = ? ORDER BY time_created, id",
        (session_id,),
    ).fetchall()
    return [
        {
            "id": row["id"],
            "time_created": row["time_created"],
            "time_updated": row["time_updated"],
            "data": decode_json(row["data"]),
        }
        for row in rows
    ]


def fetch_parts(
    connection: sqlite3.Connection, message_id: str
) -> list[dict[str, Any]]:
    rows = connection.execute(
        "SELECT id, time_created, data FROM part "
        "WHERE message_id = ? ORDER BY time_created, id",
        (message_id,),
    ).fetchall()
    return [
        {
            "id": row["id"],
            "time_created": row["time_created"],
            "data": decode_json(row["data"]),
        }
        for row in rows
    ]


def build_tree(connection: sqlite3.Connection, session_id: str) -> dict[str, Any]:
    session = fetch_session(connection, session_id)
    messages = fetch_messages(connection, session_id)
    message_nodes: list[dict[str, Any]] = []
    for message in messages:
        data = message["data"] or {}
        parts = fetch_parts(connection, message["id"])
        message_nodes.append(
            {
                "id": message["id"],
                "role": data.get("role"),
                "time_created": message["time_created"],
                "time_updated": message["time_updated"],
                "data": data,
                "parts": parts,
                "part_count": len(parts),
            }
        )
    return {
        "source": {
            "tool": "opencode",
            "storage": "sqlite",
            "notation": "links-notation",
            "extractor": "opencode-conversation-to-lino.py",
        },
        "session": session,
        "message_count": len(messages),
        "messages": message_nodes,
    }


def render(tree: dict[str, Any], session_id: str) -> str:
    output = [f"conversation {session_id}"]
    for key in ("source", "session", "message_count", "messages"):
        emit(tree[key], key, 1, output)
    return "\n".join(output) + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("session_id", help="OpenCode session id, e.g. ses_XXXX")
    parser.add_argument("--db", default=default_db_path(), help="path to opencode.db")
    parser.add_argument(
        "--format", choices=("lino", "json"), default="lino", help="output format"
    )
    parser.add_argument(
        "-o", "--output", default="-", help="output path (default: stdout)"
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        connection = open_ro(os.path.expanduser(args.db))
        try:
            tree = build_tree(connection, args.session_id)
        finally:
            connection.close()
    except (FileNotFoundError, LookupError, sqlite3.Error) as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    text = (
        render(tree, args.session_id)
        if args.format == "lino"
        else json.dumps(tree, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    )
    if args.output == "-":
        sys.stdout.write(text)
    else:
        with open(os.path.expanduser(args.output), "w", encoding="utf-8") as output:
            output.write(text)
        print(f"wrote {args.output}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
