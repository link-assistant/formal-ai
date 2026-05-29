#!/usr/bin/env python3
"""Generate Links Notation (.lino) blocks for the program catalog.

Reads the Rust source of truth (src/coding/catalog/{tasks,templates_core,
templates_extended}.rs), extracts every ProgramTask and ProgramTemplate, and
emits the matching `task_*` / `template_*_*` blocks for
data/seed/hello-world-programs.lino so the portable knowledge bundle stays a
faithful, complete mirror of the catalog (issue #330).

The lino single-quoted `code '...'` payload escapes the raw template code as:
  backslash -> \\\\, newline -> \\n, single-quote -> \\x27
(matching the existing hand-authored entries, e.g. C++ `'\\n'` -> `\\x27\\\\n\\x27`).
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "src" / "coding" / "catalog"


def read(name: str) -> str:
    return (CATALOG / name).read_text(encoding="utf-8")


def parse_raw_string(text: str, start: int):
    """Parse a Rust raw string literal beginning at `text[start] == 'r'`.

    Returns (content, end_index_after_closing_delimiter).
    """
    i = start
    assert text[i] == "r"
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


def parse_templates(text: str):
    """Yield (task_slug, language_slug, code) for each ProgramTemplate."""
    for m in re.finditer(
        r'ProgramTemplate\s*\{\s*task_slug:\s*"([^"]+)",\s*'
        r'language_slug:\s*"([^"]+)",\s*code:\s*',
        text,
    ):
        task, lang = m.group(1), m.group(2)
        code, _ = parse_raw_string(text, m.end())
        yield task, lang, code


def parse_tasks(text: str):
    """Yield (slug, [aliases], output) for each ProgramTask, in source order."""
    # Split into per-task chunks on the `slug:` field.
    for m in re.finditer(
        r'ProgramTask\s*\{(.*?)\}\s*,', text, flags=re.DOTALL
    ):
        body = m.group(1)
        slug = re.search(r'slug:\s*"([^"]+)"', body).group(1)
        output = re.search(r'output:\s*"((?:[^"\\]|\\.)*)"', body).group(1)
        alias_block = re.search(
            r'aliases:\s*&\[(.*?)\]', body, flags=re.DOTALL
        ).group(1)
        aliases = []
        for line in alias_block.splitlines():
            line = line.strip()
            if line.startswith("//") or not line:
                continue
            for a in re.findall(r'"((?:[^"\\]|\\.)*)"', line):
                aliases.append(a)
        yield slug, aliases, output


def escape_code(code: str) -> str:
    code = code.replace("\\", "\\\\")
    code = code.replace("'", "\\x27")
    code = code.replace("\n", "\\n")
    return code


def main() -> int:
    tasks_src = read("tasks.rs")
    templates = list(parse_templates(read("templates_core.rs"))) + list(
        parse_templates(read("templates_extended.rs"))
    )
    tasks = list(parse_tasks(tasks_src))

    # Which tasks/templates are NEW relative to the current lino?
    new_tasks = sys.argv[1:] if len(sys.argv) > 1 else [
        "list_files_arg",
        "fizzbuzz",
        "factorial",
        "reverse_string",
        "sum_to_ten",
    ]

    out = []
    for slug, aliases, output in tasks:
        if slug not in new_tasks:
            continue
        out.append(f"task_{slug}")
        out.append(f'  task "{slug}"')
        out.append(f'  aliases "{", ".join(aliases)}"')
        out.append(f'  output "{output}"')
        out.append("")

    for task, lang, code in templates:
        if task not in new_tasks:
            continue
        # The catalog stores the code_fence implicitly as the language slug for
        # all current languages (rust->rust, cpp->cpp, ...), matching existing
        # entries.
        out.append(f"template_{task}_{lang}")
        out.append('  intent "write_program"')
        out.append(f'  language "{lang}"')
        out.append(f'  task "{task}"')
        out.append(f'  code_fence "{lang}"')
        out.append(f"  code '{escape_code(code)}'")
        out.append("")

    sys.stdout.write("\n".join(out))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
