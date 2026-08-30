#!/usr/bin/env bash
# Falsify every capability guard issue #1066 added.
#
# Each guard claims that a ladder node can no longer fail the way it failed. A
# guard that has never been observed failing is a claim, not evidence, so this
# neutralises each fix in turn -- one early return, in the one function that
# decides -- and asserts the matching test goes red, then restores the file and
# asserts the whole set goes green again.
#
# The neutralisation is deliberately blunt. It is not a subtle mutation: it is
# the fix switched off, which is the state the repository was in when the ladder
# reported sixty-three green nodes and thirty-two of them were hollow.
#
# Usage: experiments/issue_1066_ladder_offline/falsify-node-capabilities.sh [substring ...]
#
# With no arguments every guard is falsified. An argument selects the cases
# whose test name contains it, which is what a reader checking one newly added
# guard needs; the final green run then covers only the selected cases.
#
# Runtime note: the repository compiles its tests at opt-level 2 (issue #1053),
# which is right for one run of a long suite and wrong for a loop that rebuilds
# once per case. This uses its own target directory at opt-level 0 so the main
# build cache is neither invalidated nor waited on.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"

export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$root/target/falsify}"
export CARGO_PROFILE_TEST_OPT_LEVEL=0
export CI=1

logs="${LOGS:-/tmp/issue-1066-falsify}"
mkdir -p "$logs"

# file | anchor line the neutralisation is injected after | injection | test that must go red
cases=(
  "src/agentic_coding/file_path_shape.rs|pub(super) fn is_dotted_number(token: &str) -> bool {|if true { return false; }|issue_1066_ladder_capability::a_dotted_number_is_never_mistaken_for_a_file_to_read"
  "src/agentic_coding/file_path_shape.rs|    let mut current = token;|if true { return token; }|issue_1066_ladder_capability::a_path_that_closes_a_sentence_is_still_a_path"
  "src/agentic_coding/write_request.rs|pub(super) fn honouring_pinned_first_line(request: &str, content: &str) -> Option<String> {|if true { return None; }|issue_1066_ladder_capability::a_written_file_starts_with_the_line_the_same_request_pinned"
  "src/agentic_coding/write_request.rs|pub(super) fn is_stated_write_target(request: &str, path: &str) -> bool {|if true { return false; }|issue_1066_ladder_capability::a_file_the_request_asks_for_is_never_opened_for_reading"
  "src/agentic_coding/note_composition.rs|    let specification = parse_specification(task)?;|if true { return None; }|issue_1066_ladder_capability::a_document_specification_is_composed_instead_of_transcribed"
  "src/agentic_coding/workspace_inspection.rs|pub(super) fn asks_about_the_workspace(prompt: &str) -> bool {|if true { return false; }|issue_1066_ladder_capability::a_question_about_the_repository_is_answered_by_reading_the_repository"
  "src/agentic_coding/file_read.rs|fn read_path_named_beside_its_cue(prompt: &str) -> Option<String> {|if true { return None; }|issue_1066_ladder_capability::a_read_cue_selects_the_path_in_its_own_sentence"
  "src/agentic_coding/evidence_record.rs|fn symbolic_answer(residual: &str) -> Option<String> {|if true { return None; }|issue_1066_ladder_capability::an_answer_only_the_symbolic_engine_reaches_is_still_delivered_to_the_named_file"
  "src/agentic_coding/task_structure.rs|pub(super) fn plan_task_structure_step(task: &str) -> Option<AgenticPlan> {|if true { return None; }|issue_1066_ladder_capability::a_question_about_a_task_is_answered_by_thinking_about_the_task"
  "src/agentic_coding/general_planner.rs|fn names_deferred_work_product(content: &str) -> bool {|if true { return false; }|issue_1066_ladder_capability::a_payload_that_names_the_work_product_is_not_written_as_the_body"
  "src/engine.rs|    pub fn defers_to_the_open_web(&self) -> bool {|if true { return false; }|issue_907::a_turn_that_carries_a_task_gets_the_task"
  "src/agentic_coding/evidence_record.rs|            && !carries_authoring_task(&crate::engine::normalize_prompt(sentence.text))|// neutralised|issue_907::a_turn_that_carries_a_task_gets_the_task"
  "src/engine.rs|    pub fn announces_a_list_it_does_not_make(&self) -> bool {|if true { return false; }|issue_1066_hollow_answers::an_answer_is_never_a_heading_with_no_list"
  "src/task_decomposition.rs|    pub fn unenumerable_reason(&self) -> Option<AtomicityReason> {|if true { return None; }|issue_1066_hollow_answers::an_answer_that_announces_sub_tasks_never_lists_none"
  "src/task_decomposition/stated_task.rs|pub fn without_sentence_end(task: &str) -> &str {|if true { return task; }|issue_1066_hollow_answers::a_listed_sub_task_keeps_the_text_that_says_what_to_do"
  "src/seed/meanings.rs|    pub fn mentions_role_separated(&self, role: &str, normalized: &str) -> bool {|if true { return false; }|issue_1066_ladder_capability::a_question_about_a_task_is_answered_by_thinking_about_the_task"
  "src/agentic_coding/evidence_record.rs|fn work_before_delivery(sentence: &str) -> Option<&str> {|if true { return None; }|issue_1066_ladder_capability::work_coordinated_into_its_delivery_sentence_is_not_thrown_away_with_it"
  "src/task_decomposition/stated_task.rs|fn after_introducing_colon(prompt: &str, asks: &dyn Fn(&str) -> bool) -> Option<String> {|if true { let colon = prompt.rfind(INTRODUCING_COLON)?; let tail = prompt[colon..].chars().skip(1).collect::<String>().trim().to_owned(); return (!tail.is_empty()).then_some(tail); }|issue_1066_hollow_answers::a_colon_in_a_later_sentence_does_not_become_the_task"
  "src/task_decomposition/stated_task.rs|fn asking_blocks(prompt: &str, asks: &dyn Fn(&str) -> bool) -> String {|if true { return prompt.trim().to_owned(); }|issue_1066_hollow_answers::framing_addressed_to_the_solver_is_not_a_sub_task"
  "src/computer_use/planner.rs|fn advertises_computer_use(tool_names: &[&str]) -> bool {|if true { return true; }|issue_1066_ladder_capability::a_client_that_speaks_no_computer_use_is_not_told_a_primitive_is_missing"
  "src/calculation.rs|fn sentence_end_from(prompt: &str, from: usize) -> usize {|if true { return prompt.len(); }|issue_1066_hollow_answers::a_calculator_verb_in_the_framing_does_not_claim_the_whole_prompt"
  "src/calculation.rs|fn sentence_end_from(prompt: &str, from: usize) -> usize {|if true { let tail = &prompt[from..]; if let Some(offset) = tail.find(\"\\n\\n\") { return from + offset; } }|issue_1066_hollow_answers::a_calculator_verb_does_not_claim_the_rest_of_its_paragraph"
)

wanted=("$@")
selected=0
for case in "${cases[@]}"; do
  IFS='|' read -r file anchor injection test_name <<<"$case"
  if [[ ${#wanted[@]} -gt 0 ]]; then
    keep=0
    for want in "${wanted[@]}"; do [[ "$test_name" == *"$want"* ]] && keep=1; done
    [[ $keep -eq 1 ]] || continue
  fi
  selected=$((selected + 1))
  echo "== neutralising ${file}: ${injection} =="
  backup=$(mktemp)
  cp "$file" "$backup"
  restore() { cp "$backup" "$file"; rm -f "$backup"; }
  trap restore EXIT

  python3 - "$file" "$anchor" "$injection" <<'PATCH'
import sys

path, anchor, injection = sys.argv[1], sys.argv[2], sys.argv[3]
source = open(path, encoding="utf-8").read()
count = source.count(anchor)
assert count == 1, f"anchor is not unique in {path}: found {count}"
if injection == "// neutralised":
    source = source.replace(anchor, f"            && true // {anchor.strip()}", 1)
else:
    source = source.replace(anchor, f"{anchor}\n    {injection}", 1)
open(path, "w", encoding="utf-8").write(source)
print(f"patched {path}")
PATCH

  log="$logs/$(echo "$test_name" | tr ':' '-').log"
  if cargo test --test unit -- "$test_name" --exact >"$log" 2>&1; then
    echo "FALSIFICATION FAILED: ${test_name} passed with the fix neutralised" >&2
    tail -30 "$log" >&2
    exit 1
  fi
  grep -E "panicked at|assertion|test result" "$log" | head -6

  restore
  trap - EXIT
done

[[ $selected -gt 0 ]] || { echo "no case matched: ${wanted[*]}" >&2; exit 2; }

echo "== every fix restored: the whole set must go GREEN =="
if [[ ${#wanted[@]} -gt 0 ]]; then
  cargo test --test unit -- "${wanted[@]}" 2>&1 | tail -6
else
  cargo test --test unit -- issue_1066_ladder_capability issue_1066_hollow_answers 2>&1 | tail -6
fi
