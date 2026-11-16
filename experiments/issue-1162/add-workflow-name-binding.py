# Issue #1162: the Hello World issues say "The workflow should have a
# meaningful name like `test-hello-world.yml`"; bind that file name inside
# .github/workflows/ instead of the program-stem default (Main.yml).
# Usage: python3 experiments/issue-1162/add-workflow-name-binding.py
import pathlib
ANCHOR = "  branch-forbidden-substrings "
ADD = (
    "  workflow-name-binding-patterns '[\"^(?:The |A )?(?:GitHub Actions |CI )?workflow (?:file )?(?:should|must) (?:have|use|be given) (?:a )?(?:meaningful |descriptive |clear )?(?:file ?)?name,? (?:like|such as|e\\\\.g\\\\.) ([^ ]+)$\","
    "\"^(?:Name|Call) (?:the |a )?(?:GitHub Actions |CI )?workflow (?:file )?(?:something )?(?:like|such as) ([^ ]+)$\","
    "\"^(?:Workflow|Рабочий процесс) (?:должен|должна) (?:иметь|называться) (?:осмысленное |понятное )?(?:имя|название)?,? (?:например|как) ([^ ]+)$\"]'\n"
    "  workflow-name-excluded-patterns '[]'\n"
    "  workflow-path-file \".github/workflows/{file}\"\n"
)
for path in ["data/meta/work-item-steps.lino", "rust/embedded/data/meta/work-item-steps.lino"]:
    p = pathlib.Path(path)
    text = p.read_text()
    if "workflow-name-binding-patterns" in text:
        continue
    assert text.count(ANCHOR) == 1, path
    p.write_text(text.replace(ANCHOR, ADD + ANCHOR))
    print("updated", path)
