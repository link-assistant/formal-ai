#!/usr/bin/env python3
"""Emit the `data/seed/hello-world-programs.lino` blocks for one catalog task.

The portable seed bundle is a hand-mirrored copy of the Rust catalog, and
`tests/source/source_tests/coding/catalog/mod/lino_parity.rs` fails the build
when the two drift.  `experiments/issue-330-coding-tasks/generate_lino.py`
predates the split of the template tables and the move of task aliases into the
meaning lexicon, so it can no longer read the sources it names; this script
reads the tables that exist today:

  * the templates from ``src/coding/catalog/templates_*.rs``
  * the task record (``slug`` / ``label`` / ``output`` / ``input``) from
    ``src/coding/catalog/tasks.rs``
  * the alias surfaces from the ``program_task_<slug>`` meaning in
    ``data/seed/meanings-coding-catalog.lino``, in every language it carries

Usage: generate_lino_blocks.py <task-slug>   (writes the blocks to stdout)
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "src" / "coding" / "catalog"


def parse_raw_string(text, start):
    i = start
    assert text[i] == "r", text[i : i + 20]
    i += 1
    hashes = 0
    while text[i] == "#":
        hashes += 1
        i += 1
    assert text[i] == '"'
    i += 1
    closing = '"' + "#" * hashes
    end = text.index(closing, i)
    return text[i:end], end + len(closing)


def templates(task_slug):
    for source in sorted(CATALOG.glob("templates*.rs")):
        text = source.read_text(encoding="utf-8")
        for match in re.finditer(
            r'ProgramTemplate\s*\{\s*task_slug:\s*"([^"]+)",\s*'
            r'language_slug:\s*"([^"]+)",\s*code:\s*',
            text,
        ):
            if match.group(1) != task_slug:
                continue
            code, _ = parse_raw_string(text, match.end())
            yield match.group(2), code


def task_record(task_slug):
    text = (CATALOG / "tasks.rs").read_text(encoding="utf-8")
    for match in re.finditer(r"ProgramTask\s*\{(.*?)\n    \},", text, flags=re.DOTALL):
        body = match.group(1)
        if re.search(r'slug:\s*"([^"]+)"', body).group(1) != task_slug:
            continue
        return {
            field: re.search(rf'{field}:\s*"((?:[^"\\]|\\.)*)"', body).group(1)
            for field in ("output", "input")
        }
    raise SystemExit(f"no ProgramTask named {task_slug}")


def aliases(task_slug):
    text = (ROOT / "data/seed/meanings-coding-catalog.lino").read_text(encoding="utf-8")
    block = re.search(
        rf"^  program_task_{task_slug}$(.*?)(?=^  \S|\Z)", text, flags=re.DOTALL | re.M
    )
    if block is None:
        raise SystemExit(f"no program_task_{task_slug} meaning")
    # A surface with no spaces is written unquoted in the lexicon -- which is
    # every Chinese surface, since Chinese is written without them.
    return [
        quoted or bare
        for quoted, bare in re.findall(
            r'^\s+text (?:"([^"]*)"|(\S+))$', block.group(1), flags=re.M
        )
    ]


def escape_code(code):
    return code.replace("\\", "\\\\").replace("'", "\\x27").replace("\n", "\\n")


def main():
    task_slug = sys.argv[1]
    record = task_record(task_slug)
    lines = [
        f"task_{task_slug}",
        f"  task {task_slug}",
        f'  aliases "{", ".join(aliases(task_slug))}"',
        f'  output "{record["output"]}"',
        f'  input "{record["input"]}"',
    ]
    for language, code in templates(task_slug):
        lines += [
            f"template_{task_slug}_{language}",
            "  intent write_program",
            f"  language {language}",
            f"  task {task_slug}",
            f"  code_fence {language}",
            f"  code '{escape_code(code)}'",
        ]
    print("\n".join(lines))


if __name__ == "__main__":
    main()
