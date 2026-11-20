#!/usr/bin/env python3
"""Extract verified raw Actions archives without the deprecated Node ZIP parser.

Uses the raw-download adapter pattern from link-foundation's Rust template at
309387f761f9f0f1b2ea15bfe181cb51f5a9a1fb (Unlicense). Existing regular files
can be refreshed, as with download-artifact; links and conflicting members fail.
"""
import argparse
from contextlib import ExitStack
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import stat
import tempfile
import uuid
import zipfile


def member_path(name):
    """Require a portable, unambiguous path below the selected destination."""
    if not name or "\\" in name or "\0" in name or name.startswith("/") or re.match(r"^[A-Za-z]:", name):
        raise ValueError(f"Unsafe artifact member: {name!r}")
    parts = name.removesuffix("/").split("/")
    if any(part in ("", ".", "..") for part in parts):
        raise ValueError(f"Ambiguous artifact member: {name!r}")
    return PurePosixPath(*parts)


def validate_destination(root, relative, directory):
    current = root
    parts = relative.parts
    for index, part in enumerate(parts):
        current = current / part
        if current.is_symlink():
            raise ValueError(f"Artifact destination is a symlink: {current}")
        must_be_directory = index < len(parts) - 1 or directory
        if current.exists() and ((must_be_directory and not current.is_dir()) or (not must_be_directory and not current.is_file())):
            raise ValueError(f"Conflicting artifact destination: {current}")
    return root.joinpath(*parts)


def raw_archives(raw):
    """SDK raw mode writes one ZIP per download, flat for a single artifact."""
    if not raw.is_dir() or raw.is_symlink():
        raise ValueError("Raw artifact directory is absent or is a symlink")
    sources = []
    for entry in sorted(raw.iterdir()):
        if entry.is_symlink():
            raise ValueError("Raw artifact path is a symlink")
        if entry.is_file():
            sources.append((PurePosixPath(), entry))
        elif entry.is_dir():
            files = list(entry.iterdir())
            if len(files) != 1 or files[0].is_symlink() or not files[0].is_file():
                raise ValueError("Expected one raw ZIP per artifact")
            sources.append((member_path(entry.name), files[0]))
        else:
            raise ValueError("Raw artifact path is not a regular file or directory")
    if not sources or (len(sources) > 1 and any(not prefix.parts for prefix, _ in sources)):
        raise ValueError("Raw artifact layout is empty or ambiguous")
    return sources


def extract_artifacts(raw, destination, merge_multiple=False):
    root = Path(destination).expanduser().absolute()
    if root.is_symlink() or (root.exists() and not root.is_dir()):
        raise ValueError("Artifact destination is not a real directory")
    root = root.resolve()
    planned, files, directories = [], set(), set()
    with ExitStack() as stack:
        for prefix, source in raw_archives(Path(raw)):
            archive = stack.enter_context(zipfile.ZipFile(source))
            for member in archive.infolist():
                member_path(member.orig_filename)
                relative = member_path(member.filename)
                if not merge_multiple:
                    relative = prefix / relative
                mode = stat.S_IFMT(member.external_attr >> 16)
                directory = member.is_dir()
                if mode not in (0, stat.S_IFDIR if directory else stat.S_IFREG):
                    raise ValueError(f"Non-regular artifact member: {member.filename!r}")
                key = str(relative).casefold()
                parents = {str(parent).casefold() for parent in relative.parents if parent.parts}
                if key in files or parents & files or (not directory and key in directories):
                    raise ValueError(f"Duplicate or conflicting artifact member: {relative}")
                directories.update(parents)
                (directories if directory else files).add(key)
                target = validate_destination(root, relative, directory)
                planned.append((archive, member, relative, target))
        # Validate every path and member type before any destination is changed.
        if not planned:
            raise ValueError("Artifact archives contain no members")
        root.mkdir(parents=True, exist_ok=True)
        for archive, member, relative, target in planned:
            validate_destination(root, relative, member.is_dir())
            if member.is_dir():
                target.mkdir(parents=True, exist_ok=True)
                continue
            target.parent.mkdir(parents=True, exist_ok=True)
            temporary = None
            try:
                # CRC errors never replace an existing cached executable.
                with tempfile.NamedTemporaryFile(dir=target.parent, prefix=".artifact-", delete=False) as output:
                    temporary = Path(output.name)
                    with archive.open(member) as content:
                        shutil.copyfileobj(content, output, length=1024 * 1024)
                temporary.chmod(0o644)
                validate_destination(root, relative, False)
                os.replace(temporary, target)
            finally:
                if temporary is not None:
                    temporary.unlink(missing_ok=True)
    return root


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("raw", type=Path)
    parser.add_argument("destination", type=Path)
    parser.add_argument("--merge-multiple", choices=("true", "false"), default="false")
    options = parser.parse_args()
    destination = extract_artifacts(options.raw, options.destination, options.merge_multiple == "true")
    if os.environ.get("GITHUB_OUTPUT"):
        delimiter = "artifact_path_" + uuid.uuid4().hex
        with open(os.environ["GITHUB_OUTPUT"], "a", encoding="utf-8") as output:
            output.write(f"download-path<<{delimiter}\n{destination}\n{delimiter}\n")
    print(f"Extracted verified artifact archives to {destination}")


if __name__ == "__main__":
    main()
