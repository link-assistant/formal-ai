#!/usr/bin/env python3
"""Extract one complete OpenCode SQLite session as structured JSON.

The database is opened read-only, uses only Python's standard library, and
orders messages and parts by ``(time_created, id)`` for deterministic output.
Formal AI's context CLI owns the shared JSON-to-Links-Notation conversion.
"""

from __future__ import annotations

import argparse
import json
import os
import sqlite3
import sys
from typing import Any

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
            "notation": "json",
            "extractor": "opencode-conversation-to-lino.py",
        },
        "session": session,
        "message_count": len(messages),
        "messages": message_nodes,
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("session_id", help="OpenCode session id, e.g. ses_XXXX")
    parser.add_argument("--db", default=default_db_path(), help="path to opencode.db")
    parser.add_argument(
        "--format", choices=("json",), default="json", help="output format"
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

    text = json.dumps(tree, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
    if args.output == "-":
        sys.stdout.write(text)
    else:
        with open(os.path.expanduser(args.output), "w", encoding="utf-8") as output:
            output.write(text)
        print(f"wrote {args.output}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
