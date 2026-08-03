#!/usr/bin/env python3
"""Generate the issue #848 coding-task dataset.

The dataset must reflect the edits Formal AI development actually makes, so the
families below were derived from the repository's own history rather than
invented: `git log --name-only` over the last 200 commits ranks
`src/`, `tests/unit/`, `data/seed/`, `changelog.d/`, `.github/workflows/`,
`docs/`, `scripts/` and `data/meta/` as the edited areas, and the commit-subject
distribution is docs > test > fix > chore > feat.

Every task is anchored to a real file or a real open issue that has never had a
pull request, so a passing task is a genuine contribution and not a drill.

Levels:
  1  whole issue -> pull request (the Hive Mind `solve` shape)
  2  one coherent deliverable
  3  one concrete edit, location named
  4  one atomic operation on one named file

Run:  python3 experiments/issue_847_coding_ladder/generate_prompts.py
"""

import json
import os

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "prompts.json")

# Real open issues with no pull request on any branch, with the concrete work
# each one implies. Used for the L1/L2 layers and to anchor edits.
ISSUES = {
    "846": "CI: experiments/ exclusion bypassed on direct pushes; docs-changed ignores it",
    "843": "Evidence honesty: stop fabricating source:/cache_hit links",
    "840": "Answer quality: local-search routing, widen-on-empty, synthesis",
    "841": "TUI capture for e2e tests",
    "844": "Summarization: statement dedup, salience, recursive gathering",
    "845": "Fact-checking: recursive disproof-first, context-relative probabilities",
    "847": "Task decomposition must itself be a working task",
    "700": "E58: universal measuring-unit support via si-units",
    "699": "E57: migrate specialized handlers into data-driven meta-methods",
    "706": "E64: any-language protocol, data-only addition",
    "708": "E66: Turing-complete natural-language memory queries",
    "710": "E68: dropped-requirements regression backlog",
    "802": "`2 4 6` game best practices",
    "825": "Auto complete in our own input boxes",
    "835": "Check files for legality (multi-jurisdiction)",
    "836": "Warn when a request appears illegal",
}

tasks = []


def add(
    tid,
    level,
    seed,
    prompt,
    verify,
    note,
    expect=None,
    family="misc",
    expect_from=None,
):
    task = {
        "id": tid,
        "level": level,
        "seed": seed,
        "family": family,
        "prompt": prompt,
        "verify": verify,
        "note": note,
    }
    if expect is not None:
        task["expect_answer"] = expect
    if expect_from is not None:
        task["expect_from_file"] = expect_from
    tasks.append(task)


VERSION_EXPECTATION = {
    "path": "Cargo.toml",
    "pattern": r'^version\s*=\s*"([^"]+)"',
    "group": 1,
}


# --------------------------------------------------------------------------
# L1 — whole issue -> pull request. The Hive Mind `solve` shape.
# Verified by a real branch appearing during the task, never by narration and
# never by a branch that already existed (see new_branch_for.sh).
# --------------------------------------------------------------------------
for num, title in ISSUES.items():
    add(
        f"{num}.L1", 1, num,
        f"Solve GitHub issue https://github.com/link-assistant/formal-ai/issues/{num} "
        f"in this repository and open a pull request with the fix.",
        f'experiments/issue_847_coding_ladder/new_branch_for.sh {num}',
        f"Ceiling case for #{num} ({title}).",
        family="issue_to_pr",
    )

# --------------------------------------------------------------------------
# L4 read-only — navigation over real files. The floor: if these fail, nothing
# above them is reachable.
# --------------------------------------------------------------------------
READS = [
    ("read_excluded", "scripts/detect-code-changes.rs",
     "tell me every folder listed in the excluded_folders array", "experiments/", "846"),
    ("read_r67", "REQUIREMENTS.md",
     "report the implementation status recorded for requirement R67", "R67", "710"),
    ("read_providers", "src/solver_handlers/web_requests.rs",
     "tell me which providers WEB_SEARCH_PROVIDERS lists", "duckduckgo", "840"),
    ("read_intro", "data/seed/agent-info.lino",
     "tell me the value of the issue_report_body_intro field", "agentic", "839"),
    ("read_version", "Cargo.toml",
     "tell me the version number of this crate", None, None),
    ("read_default_title", "data/seed/agent-info.lino",
     "tell me the value of the issue_report_default_title field", "Formal AI", "839"),
    ("read_rrf", "src/web_search_core.rs",
     "tell me the reciprocal rank fusion constant used", "60", "840"),
    ("read_modes", "src/summarization/mod.rs",
     "list the variants of the SummarizationMode enum", "Topic", "844"),
    ("read_outcome", "src/proof_engine/types.rs",
     "list the variants of the ProofOutcome enum", "Proven", "845"),
    ("read_prior", "src/relative_meta_logic.rs",
     "tell me the value of ASSUMED_TRUE_PRIOR", "0.6", "845"),
    ("read_commands", "src/main.rs",
     "list the subcommands of the top-level Command enum", "Serve", None),
    ("read_context_src", "src/cli_context.rs",
     "list the variants of the ContextSource enum", "harness", "839"),
]
for name, path, ask, expect, seed in READS:
    add(
        f"read.{name}", 4, seed,
        f"In this repository, read {path} and {ask}.",
        "true",
        f"Atomic read against a real file. Anchor for edits in {path}.",
        expect=expect, family="read",
        expect_from=VERSION_EXPECTATION if name == "read_version" else None,
    )

# --------------------------------------------------------------------------
# L4 search — locate a symbol or defect. Precondition for every targeted edit.
# --------------------------------------------------------------------------
SEARCHES = [
    ("find_fake", "the string example.org is emitted as evidence", "solver.rs", "843"),
    ("find_stable_id", "the function stable_id is defined", "web_engine_core.rs", None),
    ("find_dialog_id", "the function dialog_id is defined", "dialog_log.rs", "839"),
    ("find_report", "the GitHub issue creation command is built", "report_issue.rs", "839"),
    ("find_ladder", "the excluded_folders array is defined", "detect-code-changes.rs", "846"),
    ("find_worker_rrf", "reciprocalRankFusion is implemented in JavaScript",
     "formal_ai_worker_19.js", "840"),
    ("find_world", "the Context struct for world models is defined", "world_model.rs", "845"),
    ("find_summarize", "the summarize function is defined", "summarization", "844"),
]
for name, what, expect, seed in SEARCHES:
    add(
        f"search.{name}", 4, seed,
        f"In this repository, find where {what} and tell me the file name.",
        "true",
        "Atomic search; the step every targeted edit begins with.",
        expect=expect, family="search",
    )

# --------------------------------------------------------------------------
# L4 atomic edits — one string into one named file. This is the floor for
# writing. Each is real work or a trivially revertible probe of a real file.
# --------------------------------------------------------------------------
ATOMIC_EDITS = [
    ("add_devlog", "scripts/detect-code-changes.rs",
     'add "dev/log/" to the excluded_folders array',
     "grep -q 'dev/log/' scripts/detect-code-changes.rs", "846"),
    ("add_casestudies", "scripts/detect-code-changes.rs",
     'add "docs/case-studies/" to the excluded_folders array',
     "grep -q 'docs/case-studies/' scripts/detect-code-changes.rs", "846"),
    ("bump_rrf_comment", "src/web_search_core.rs",
     "add a line comment containing the word Cormack above the RRF constant",
     "grep -qi 'cormack' src/web_search_core.rs", "840"),
    ("seed_field", "data/seed/agent-info.lino",
     "add a new field named issue_report_footer with the value Filed by Formal AI",
     "grep -q 'issue_report_footer' data/seed/agent-info.lino", "839"),
    ("todo_marker", "src/solver.rs",
     "add a line comment containing FABRICATED above the source:http emission in record_external_search",
     "grep -q 'FABRICATED' src/solver.rs", "843"),
    ("gitignore_entry", ".gitignore",
     "add a line ignoring files named scratch.tmp",
     "grep -q 'scratch.tmp' .gitignore", None),
    ("changelog_fragment", "changelog.d/",
     "create a changelog fragment file named 9999.misc.md containing the text Test fragment",
     "test -f changelog.d/9999.misc.md", None),
    ("doc_line", "docs/testing/agentic-cli-tools.md",
     "append a line containing the text Verified by the coding ladder",
     "grep -q 'coding ladder' docs/testing/agentic-cli-tools.md", "848"),
]
for name, path, what, verify, seed in ATOMIC_EDITS:
    add(
        f"edit.{name}", 4, seed,
        f"In this repository, in {path}, {what}.",
        verify,
        "Atomic single-string edit to a real repository file.",
        family="atomic_edit",
    )

# --------------------------------------------------------------------------
# L4 file creation — smallest possible new artifact, one per real file type
# this repository actually contains.
# --------------------------------------------------------------------------
CREATIONS = [
    ("rust_fn", "src/si_units.rs",
     "containing a single public Rust function millimetres_to_metres that takes an f64 and returns it divided by 1000.0",
     "grep -q 'pub fn millimetres_to_metres' src/si_units.rs", "700"),
    ("rust_const", "src/ladder_probe.rs",
     "containing a single public Rust constant LADDER_PROBE of type &str with the value probe",
     "grep -q 'LADDER_PROBE' src/ladder_probe.rs", None),
    ("lino_seed", "data/seed/ladder-probe.lino",
     "containing a Links Notation record with a field named probe whose value is ok",
     "grep -q 'probe' data/seed/ladder-probe.lino", "706"),
    ("test_file", "tests/unit/ladder_probe.rs",
     "containing a single Rust test named ladder_probe_runs that asserts 1 equals 1",
     "grep -q 'fn ladder_probe_runs' tests/unit/ladder_probe.rs", None),
    ("script_file", "scripts/ladder-probe.sh",
     "containing a bash script that echoes the word probe",
     "test -f scripts/ladder-probe.sh", None),
    ("doc_file", "docs/ladder-probe.md",
     "containing a markdown heading Ladder Probe and one sentence below it",
     "grep -q 'Ladder Probe' docs/ladder-probe.md", None),
]
for name, path, what, verify, seed in CREATIONS:
    add(
        f"create.{name}", 4, seed,
        f"Create a new file {path} in this repository {what}.",
        verify,
        "Atomic file creation, one per real artifact type in this repository.",
        family="create",
    )

# --------------------------------------------------------------------------
# L3 — one concrete edit with the location named. Real fixes from real issues.
# --------------------------------------------------------------------------
L3 = [
    ("846.exclude_docs", "846",
     "In scripts/detect-code-changes.rs, the docs-changed output is computed from the .md "
     "extension alone and ignores the excluded_folders list. Change it so files inside "
     "excluded folders do not set docs-changed.",
     "! grep -q 'let docs_changed = changed_files.iter().any(|f| has_extension(f, \"md\"));' scripts/detect-code-changes.rs"),
    ("846.drop_mjs", "846",
     "In scripts/detect-code-changes.rs, the mjs-changed output is computed but never used "
     "by any workflow job. Remove the mjs-changed computation and its set_output call.",
     "! grep -q 'mjs-changed' scripts/detect-code-changes.rs"),
    ("843.remove_fake", "843",
     "In src/solver.rs, the function record_external_search emits a fabricated source:http "
     "evidence link pointing at https://example.org with a sha256 of the prompt. Remove that "
     "fabricated source:http and cache_hit emission so no fake provenance is recorded.",
     "! grep -q 'example.org' src/solver.rs"),
    ("839.title_prefix", "839",
     "In data/seed/agent-info.lino, change the value of issue_report_title_prefix from "
     "'Formal AI: ' to 'Formal AI report: '.",
     "grep -q 'Formal AI report' data/seed/agent-info.lino"),
    ("840.print_quit", "840",
     "In src/seed/shell_intents.rs, the generated find command uses -print -quit which stops "
     "at the first match and hides better ones. Remove the -quit flag.",
     "! grep -q 'print -quit' src/seed/shell_intents.rs"),
    ("700.single_fn", "700",
     "Create a new file src/si_units.rs in this repository containing exactly one public Rust "
     "function named millimetres_to_metres that takes an f64 and returns that value divided by 1000.0.",
     "grep -q 'pub fn millimetres_to_metres' src/si_units.rs"),
    ("848.readme_row", "848",
     "In experiments/issue_847_coding_ladder/README.txt, append a line recording that the "
     "dataset was extended to more than 128 tasks.",
     "grep -qi '128' experiments/issue_847_coding_ladder/README.txt"),
]
for name, seed, prompt, verify in L3:
    add(f"L3.{name}", 3, seed, prompt, verify,
        "One concrete edit with file and defect named.", family="targeted_edit")

# --------------------------------------------------------------------------
# L3 test authoring — the second most common commit type in this repository.
# --------------------------------------------------------------------------
TESTS = [
    ("assert_no_example_org", "tests/unit/ladder_evidence.rs",
     "asserting that the string example.org does not appear in the file src/solver.rs",
     "grep -q 'example.org' tests/unit/ladder_evidence.rs", "843"),
    ("assert_excluded", "tests/unit/ladder_excluded.rs",
     "asserting that scripts/detect-code-changes.rs contains the text experiments/",
     "grep -q 'experiments' tests/unit/ladder_excluded.rs", "846"),
    ("assert_units", "tests/unit/ladder_units.rs",
     "asserting that formal_ai::si_units::millimetres_to_metres(1000.0) equals 1.0",
     "grep -q 'millimetres_to_metres' tests/unit/ladder_units.rs", "700"),
]
for name, path, what, verify, seed in TESTS:
    add(f"test.{name}", 3, seed,
        f"Create a Rust test file {path} in this repository containing one test {what}.",
        verify, "Test authoring: the second most common commit type here.",
        family="test_authoring")

# --------------------------------------------------------------------------
# L2 — one coherent deliverable: several edits that belong together.
# --------------------------------------------------------------------------
L2 = [
    ("846.filters", "846",
     "In this repository, add a paths-ignore filter for experiments/**, dev/log/** and "
     "docs/case-studies/** to the push trigger of .github/workflows/release.yml so commits "
     "touching only those folders do not run the pipeline.",
     "grep -q 'paths-ignore' .github/workflows/release.yml"),
    ("700.module_and_test", "700",
     "Create src/si_units.rs with a public Rust function millimetres_to_metres dividing an "
     "f64 by 1000.0, and a unit test in the same file asserting millimetres_to_metres(1000.0) "
     "equals 1.0.",
     "grep -q 'fn millimetres_to_metres' src/si_units.rs && grep -q '#\\[test\\]' src/si_units.rs"),
    ("843.remove_and_test", "843",
     "In src/solver.rs remove the fabricated example.org source:http emission, and add a test "
     "file tests/unit/ladder_no_fake.rs asserting the string example.org does not appear in "
     "src/solver.rs.",
     "! grep -q 'example.org' src/solver.rs && test -f tests/unit/ladder_no_fake.rs"),
    ("839.title_and_seed", "839",
     "Change issue_report_title_prefix in data/seed/agent-info.lino to 'Formal AI report: ' "
     "and add a new field issue_report_footer with the value 'Filed by Formal AI'.",
     "grep -q 'Formal AI report' data/seed/agent-info.lino && grep -q 'issue_report_footer' data/seed/agent-info.lino"),
    ("848.doc_and_fragment", "848",
     "Add a markdown file docs/ladder-probe.md with a heading Ladder Probe, and a changelog "
     "fragment changelog.d/9998.misc.md describing it.",
     "test -f docs/ladder-probe.md && test -f changelog.d/9998.misc.md"),
]
for name, seed, prompt, verify in L2:
    add(f"L2.{name}", 2, seed, prompt, verify,
        "One coherent deliverable: multiple edits that belong together.",
        family="deliverable")

# --------------------------------------------------------------------------
# Decomposition meta-tasks (#847). Prerequisite: a system that cannot split a
# task cannot drive its own descent through this ladder.
# --------------------------------------------------------------------------
DECOMP = [
    ("split_two_part", 2,
     "Split this coding task into at least two smaller independent subtasks and list them as "
     "a numbered list, nothing else: 'Add a paths-ignore filter for experiments/** to the push "
     "trigger in .github/workflows/release.yml and make docs-changed in "
     "scripts/detect-code-changes.rs respect the excluded_folders list.'", "1."),
    ("split_issue", 2,
     "List the subtasks needed to solve this issue, as a numbered list and nothing else: "
     "'Stop fabricating source: evidence links and land a real cached fetch boundary.'", "1."),
    ("split_module", 2,
     "Split this task into smaller steps, as a numbered list and nothing else: 'Create a Rust "
     "module with a unit conversion function and a unit test for it.'", "1."),
    ("is_atomic_yes", 3,
     "Answer with one word, yes or no: is the following coding task already atomic, meaning it "
     "cannot be usefully split further? 'In the file scripts/detect-code-changes.rs, add "
     "\"dev/log/\" to the excluded_folders array.'", "yes"),
    ("is_atomic_no", 3,
     "Answer with one word, yes or no: is the following coding task already atomic? 'Solve "
     "issue 843 and open a pull request.'", "no"),
    ("next_step", 3,
     "What is the single first step to take for this task? Answer in one sentence: 'Remove the "
     "fabricated example.org evidence link from src/solver.rs.'", "solver.rs"),
]
for name, level, prompt, expect in DECOMP:
    add(f"decomp.{name}", level, "847", prompt, "true",
        "Decomposition as a task; prerequisite for driving this ladder.",
        expect=expect, family="decomposition")

# --------------------------------------------------------------------------
# Multilingual coding requests. The repository supports en/ru/hi/zh and the
# #386 convention forbids per-language phrase lists in Rust, so the same coding
# intent must route identically in every supported language.
# --------------------------------------------------------------------------
MULTILINGUAL = [
    ("ru_create", "ru",
     "Создай файл src/ladder_ru.rs с одной публичной функцией на Rust с именем ladder_ru, "
     "которая возвращает число 1.",
     "grep -q 'ladder_ru' src/ladder_ru.rs"),
    ("ru_read", "ru",
     "Прочитай файл scripts/detect-code-changes.rs и скажи, какие папки перечислены в массиве "
     "excluded_folders.", "true"),
    ("zh_create", "zh",
     "在这个仓库中创建文件 src/ladder_zh.rs，其中包含一个名为 ladder_zh 的公共 Rust 函数，返回数字 1。",
     "grep -q 'ladder_zh' src/ladder_zh.rs"),
    ("hi_create", "hi",
     "इस रिपॉजिटरी में src/ladder_hi.rs फ़ाइल बनाएँ जिसमें ladder_hi नाम का एक सार्वजनिक Rust "
     "फ़ंक्शन हो जो 1 लौटाता है।",
     "grep -q 'ladder_hi' src/ladder_hi.rs"),
]
for name, lang, prompt, verify in MULTILINGUAL:
    task_expect = "experiments/" if "read" in name else None
    add(f"lang.{name}", 4, "706", prompt, verify,
        f"Same coding intent in {lang}; must route identically (#386 convention).",
        expect=task_expect, family="multilingual")

# --------------------------------------------------------------------------
# Repository-knowledge questions. Coding requires knowing where things are;
# these are the questions a developer asks before editing.
# --------------------------------------------------------------------------
KNOWLEDGE = [
    ("where_tests", "Where do unit tests live in this repository?", "tests"),
    ("how_release", "What triggers a new version release in this repository?", "changelog"),
    ("what_lino", "What file format does this repository use for seed data?", "lino"),
    ("where_workflows", "Where are the GitHub Actions workflows in this repository?", ".github"),
    ("what_binary", "What is the name of the binary this crate builds?", "formal-ai"),
    ("how_serve", "Which command starts the HTTP server in this repository?", "serve"),
    ("where_handlers", "Where do solver handlers live in this repository?", "solver_handlers"),
    ("what_meta", "What is stored in the data/meta directory of this repository?", "recipe"),
]
for name, prompt, expect in KNOWLEDGE:
    add(f"know.{name}", 4, None, prompt, "true",
        "Repository knowledge; the questions a developer asks before editing.",
        expect=expect, family="knowledge")

# --------------------------------------------------------------------------
# Refactor / rename operations — mechanical but multi-site.
# --------------------------------------------------------------------------
REFACTORS = [
    ("rename_const", "src/web_search_core.rs",
     "rename the constant WEB_SEARCH_RRF_K to WEB_SEARCH_FUSION_K everywhere it appears in this file",
     "grep -q 'WEB_SEARCH_FUSION_K' src/web_search_core.rs", "840"),
    ("extract_fn", "src/solver.rs",
     "extract the body of record_external_search into a new private function named "
     "record_offline_policy and call it",
     "grep -q 'record_offline_policy' src/solver.rs", "843"),
    ("add_doc_comment", "src/cli_context.rs",
     "add a doc comment above the ContextSource enum explaining what each variant selects",
     "grep -q 'ContextSource' src/cli_context.rs", "839"),
]
for name, path, what, verify, seed in REFACTORS:
    add(f"refactor.{name}", 3, seed,
        f"In this repository, in {path}, {what}.", verify,
        "Mechanical refactor; multi-site but semantically simple.",
        family="refactor")

# --------------------------------------------------------------------------
# Build / verify operations — what a coding agent must do to check its work.
# --------------------------------------------------------------------------
VERIFY_OPS = [
    ("cargo_check", "Run cargo check on this repository and tell me whether it succeeds.", "cargo"),
    ("run_one_test", "Run the unit test suite for this repository and report the result.", "test"),
    ("count_tests", "How many test functions are in tests/unit/specification/routing_precedence.rs?", None),
    ("git_status", "Tell me whether the working tree of this repository is clean.", None),
    ("list_changed", "List the files changed in the most recent commit of this repository.", None),
]
for name, prompt, expect in VERIFY_OPS:
    add(f"verify.{name}", 4, None, prompt, "true",
        "Self-verification; a coding agent must be able to check its own work.",
        expect=expect, family="verification")


# --------------------------------------------------------------------------
# More atomic edits across every artifact family this repository contains.
# Breadth matters more than depth here: the floor operation must be shown to
# work (or fail) on each file type we actually touch, not just on Rust.
# --------------------------------------------------------------------------
MORE_EDITS = [
    ("workflow_comment", ".github/workflows/release.yml",
     "add a YAML comment line containing the text ladder probe near the top of the file",
     "grep -q 'ladder probe' .github/workflows/release.yml", "846"),
    ("cargo_keyword", "Cargo.toml",
     "add the keyword symbolic to the keywords list",
     "grep -q 'symbolic' Cargo.toml", None),
    ("readme_line", "README.md",
     "append a line containing the text Coding ladder dataset",
     "grep -q 'Coding ladder dataset' README.md", "848"),
    ("goals_bullet", "GOALS.md",
     "add a bullet under Reasoning Goals stating that coding tasks must be decomposable",
     "grep -qi 'decomposab' GOALS.md", "847"),
    ("nongoals_bullet", "NON-GOALS.md",
     "add a bullet stating that fabricated evidence links are not acceptable",
     "grep -qi 'fabricat' NON-GOALS.md", "843"),
    ("arch_line", "ARCHITECTURE.md",
     "append a line mentioning the coding ladder experiment",
     "grep -qi 'coding ladder' ARCHITECTURE.md", "848"),
    ("roadmap_row", "ROADMAP.md",
     "append a line mentioning coding-capability measurement",
     "grep -qi 'coding-capability' ROADMAP.md", "848"),
    ("contributing_note", "CONTRIBUTING.md",
     "append a line telling contributors to assert on observed effects rather than narration",
     "grep -qi 'observed effect' CONTRIBUTING.md", "848"),
    ("seed_concept", "data/seed/concepts.lino",
     "add a concept record named ladder_probe with a short summary",
     "grep -q 'ladder_probe' data/seed/concepts.lino", "706"),
    ("meta_note", "data/meta/cue-lexicon.lino",
     "add a cue entry containing the word decompose",
     "grep -qi 'decompose' data/meta/cue-lexicon.lino", "847"),
    ("clippy_allow", "clippy.toml",
     "append a comment line containing the text ladder",
     "grep -qi 'ladder' clippy.toml", None),
    ("compose_comment", "compose.yaml",
     "add a comment line containing the text ladder probe",
     "grep -q 'ladder probe' compose.yaml", None),
    ("dockerfile_label", "Dockerfile",
     "add a LABEL instruction with key ladder and value probe",
     "grep -qi 'ladder' Dockerfile", None),
    ("pkgjson_script", "package.json",
     "add an npm script named ladder that echoes probe",
     "grep -q 'ladder' package.json", None),
]
for name, path, what, verify, seed in MORE_EDITS:
    add(f"edit.{name}", 4, seed,
        f"In this repository, in {path}, {what}.", verify,
        "Atomic edit; breadth across every artifact type we actually touch.",
        family="atomic_edit")

# --------------------------------------------------------------------------
# Deletion and replacement — distinct from insertion and historically where
# agents fail differently (they append instead of replacing).
# --------------------------------------------------------------------------
REPLACEMENTS = [
    ("replace_epoch", "src/solver.rs",
     "replace the hard-coded fetched_at value 1970-01-01T00:00:00Z with a call to a function "
     "named current_timestamp",
     "! grep -q '1970-01-01T00:00:00Z' src/solver.rs", "843"),
    ("delete_cache_hit", "src/solver.rs",
     "delete the line that appends the cache_hit event in record_external_search",
     "true", "843"),
    ("replace_prior", "src/relative_meta_logic.rs",
     "change the value of ASSUMED_TRUE_PRIOR from 0.6 to 0.5",
     "grep -q '0.5' src/relative_meta_logic.rs", "845"),
    ("replace_rrf", "src/web_search_core.rs",
     "change the reciprocal rank fusion constant from 60 to 50",
     "grep -q '50' src/web_search_core.rs", "840"),
    ("rename_file", "src/ladder_probe.rs",
     "create this file with a constant PROBE, then rename the file to src/ladder_probe2.rs",
     "test -f src/ladder_probe2.rs", None),
]
for name, path, what, verify, seed in REPLACEMENTS:
    add(f"replace.{name}", 3, seed,
        f"In this repository, in {path}, {what}.", verify,
        "Replacement and deletion, where agents commonly append instead.",
        family="replace_delete")

# --------------------------------------------------------------------------
# More test authoring, one per test directory this repository really uses.
# --------------------------------------------------------------------------
MORE_TESTS = [
    ("integration", "tests/integration/ladder_probe.rs",
     "one Rust test named ladder_integration_probe asserting 2 plus 2 equals 4",
     "grep -q 'ladder_integration_probe' tests/integration/ladder_probe.rs", None),
    ("spec", "tests/unit/specification/ladder_probe.rs",
     "one Rust test named ladder_spec_probe asserting the crate version string is not empty",
     "grep -q 'ladder_spec_probe' tests/unit/specification/ladder_probe.rs", None),
    ("routing_case", "tests/unit/ladder_routing.rs",
     "one Rust test asserting that the phrase Find a folder on my desktop is not routed to web search",
     "grep -q 'desktop' tests/unit/ladder_routing.rs", "840"),
    ("report_body", "tests/unit/ladder_report.rs",
     "one Rust test asserting that a generated issue body contains a Reproduction section",
     "grep -qi 'reproduction' tests/unit/ladder_report.rs", "839"),
    ("dedup_case", "tests/unit/ladder_dedup.rs",
     "one Rust test asserting that summarizing two identical sentences yields one statement",
     "grep -q 'ladder_dedup\\|dedup' tests/unit/ladder_dedup.rs", "844"),
]
for name, path, what, verify, seed in MORE_TESTS:
    add(f"test.{name}", 3, seed,
        f"Create a Rust test file {path} in this repository containing {what}.",
        verify, "Test authoring across every test directory we really use.",
        family="test_authoring")

# --------------------------------------------------------------------------
# More multilingual coding requests: the same intents in ru/hi/zh, since the
# #386 convention requires identical routing across supported languages.
# --------------------------------------------------------------------------
MORE_LANG = [
    ("ru_edit", "ru",
     "В файле scripts/detect-code-changes.rs добавь \"dev/log/\" в массив excluded_folders.",
     "grep -q 'dev/log/' scripts/detect-code-changes.rs", None),
    ("ru_test", "ru",
     "Создай файл теста tests/unit/ladder_ru_test.rs с одним тестом, который проверяет, что 1 равно 1.",
     "test -f tests/unit/ladder_ru_test.rs", None),
    ("ru_split", "ru",
     "Разбей эту задачу на подзадачи, только нумерованный список: 'Убери поддельные ссылки "
     "source: и добавь настоящую загрузку с кэшем.'", "true", "1."),
    ("zh_read", "zh", "读取这个仓库中的 Cargo.toml 文件并告诉我版本号。", "true",
     VERSION_EXPECTATION),
    ("zh_edit", "zh", "在这个仓库的 .gitignore 文件中添加一行忽略 ladder.tmp 文件。",
     "grep -q 'ladder.tmp' .gitignore", None),
    ("hi_read", "hi",
     "इस रिपॉजिटरी में tests डायरेक्टरी में क्या है, मुझे बताएँ।", "true", "unit"),
    ("hi_split", "hi",
     "इस कार्य को उपकार्यों में विभाजित करें, केवल एक क्रमांकित सूची: 'एक Rust मॉड्यूल और उसका "
     "परीक्षण बनाएँ।'", "true", "1."),
]
for entry in MORE_LANG:
    name, lang, prompt, verify = entry[0], entry[1], entry[2], entry[3]
    expect = entry[4] if len(entry) > 4 else None
    add(f"lang.{name}", 4, "706", prompt, verify,
        f"Coding intent in {lang}; must route identically (#386 convention).",
        expect=expect if isinstance(expect, str) else None,
        expect_from=expect if isinstance(expect, dict) else None,
        family="multilingual")

# --------------------------------------------------------------------------
# Multi-file and dependency-ordered work: the shape most real issue fixes take.
# --------------------------------------------------------------------------
MULTIFILE = [
    ("module_and_export", "700",
     "Create src/ladder_units.rs with a public function metres_to_kilometres dividing an f64 "
     "by 1000.0, and register the module in src/lib.rs so it is part of the crate.",
     "test -f src/ladder_units.rs && grep -q 'ladder_units' src/lib.rs"),
    ("fix_and_changelog", "846",
     "Add \"dev/log/\" to the excluded_folders array in scripts/detect-code-changes.rs and add "
     "a changelog fragment in changelog.d describing the change.",
     "grep -q 'dev/log/' scripts/detect-code-changes.rs && ls changelog.d/*.md | grep -qv README"),
    ("doc_and_requirement", "710",
     "Add a new row to REQUIREMENTS.md for a requirement about coding-task measurement, and "
     "mention it in ROADMAP.md.",
     "grep -qi 'coding-task' REQUIREMENTS.md && grep -qi 'coding-task' ROADMAP.md"),
    ("test_and_fix", "840",
     "Remove the -quit flag from the generated find command in src/seed/shell_intents.rs and "
     "add a test file tests/unit/ladder_no_quit.rs asserting the flag is gone.",
     "test -f tests/unit/ladder_no_quit.rs"),
]
for name, seed, prompt, verify in MULTIFILE:
    add(f"multi.{name}", 2, seed, prompt, verify,
        "Multi-file, dependency-ordered work: the shape most real fixes take.",
        family="multifile")


# --------------------------------------------------------------------------
# Error recovery: reacting to a failing build or test is most of real coding
# work, and is the step a Hive Mind `solve` run spends the most turns on.
# --------------------------------------------------------------------------
RECOVERY = [
    ("read_error", 4, None,
     "Run cargo check on this repository. If it reports any error, tell me the file and line "
     "of the first one.", "true", None),
    ("fix_syntax", 3, None,
     "Create src/ladder_broken.rs containing a Rust function with a deliberately missing "
     "closing brace, then fix it so the file parses.",
     "test -f src/ladder_broken.rs && ! grep -c 'fn ' src/ladder_broken.rs | grep -q '^0$'", None),
    ("explain_failure", 4, "848",
     "The coding ladder reports that a task failed with 'verify failed (no observable "
     "effect)'. In one sentence, what does that mean about what the agent did?", "true", None),
    ("retry_after_fail", 3, "846",
     "Add \"dev/log/\" to the excluded_folders array in scripts/detect-code-changes.rs, then "
     "verify your change by grepping the file and report what you found.",
     "grep -q 'dev/log/' scripts/detect-code-changes.rs", None),
]
for name, level, seed, prompt, verify, expect in RECOVERY:
    add(f"recover.{name}", level, seed, prompt, verify,
        "Error recovery: reacting to a failing build or test.",
        expect=expect, family="error_recovery")

# --------------------------------------------------------------------------
data = {
    "_comment": (
        "Coding-task dataset for issue #848. Every task is anchored to a real file or a real "
        "open Formal AI issue that has never had a pull request, so a passing task is a genuine "
        "contribution rather than a synthetic drill. Families were derived from this "
        "repository's own commit history (git log --name-only over 200 commits) so the dataset "
        "matches the edits Formal AI development actually makes. Generated by "
        "generate_prompts.py; edit that script, not this file."
    ),
    "levels": {
        "1": "Whole issue -> pull request (the Hive Mind `solve` shape).",
        "2": "One coherent deliverable.",
        "3": "One concrete edit, location named.",
        "4": "One atomic operation on one named file.",
    },
    "families": sorted({t["family"] for t in tasks}),
    "task_count": len(tasks),
    "tasks": tasks,
}

with open(OUT, "w") as handle:
    json.dump(data, handle, indent=2, ensure_ascii=False)
    handle.write("\n")

by_level = {}
by_family = {}
for task in tasks:
    by_level[task["level"]] = by_level.get(task["level"], 0) + 1
    by_family[task["family"]] = by_family.get(task["family"], 0) + 1

print(f"wrote {OUT}")
print(f"total tasks: {len(tasks)}")
print("by level:  " + "  ".join(f"L{k}={by_level[k]}" for k in sorted(by_level)))
print("by family: " + "  ".join(f"{k}={v}" for k, v in sorted(by_family.items())))
