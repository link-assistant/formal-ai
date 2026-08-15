## Reproduction

Gemini CLI v0.55.1 can emit a spurious filesystem warning when its isolated
home is also the directory being scanned as the project:

```bash
workdir="$(mktemp -d)"
mkdir -p "$workdir/.gemini"
HOME="$workdir" GEMINI_CLI_HOME="$workdir" gemini -p "write hello.txt" --yolo
```

Observed in link-assistant/formal-ai run 31884932415:

```text
Warning: Could not read directory .../.gemini/projects.json.lock: ENOENT: no such file or directory, scandir '.../.gemini/projects.json.lock'
```

`ProjectRegistry` uses `proper-lockfile` around `projects.json`, while
`getFolderStructure` recursively enumerates the project. The lock directory can
disappear between enumeration and descent, so a normal lock release is reported
as a project-read warning. This is distinct from the proper-lockfile background
`stat` crash discussed in #1631 and closed PR #25885: this warning comes from
the workspace tree walker.

## Workaround

Keep mutable Gemini state and the scanned project as siblings:

```bash
mkdir -p "$workdir/home/.gemini" "$workdir/project"
cd "$workdir/project"
HOME="$workdir/home" GEMINI_CLI_HOME="$workdir/home" gemini -p "write hello.txt" --yolo
```

## Suggested code fix

Ignore entries that disappear with `ENOENT` between `readdir` and recursive
descent, or exclude Gemini's own transient `projects.json.lock` directory from
the folder-structure scan. Add a race regression where `readdir` returns the
lock directory and the directory is removed before the recursive read.

