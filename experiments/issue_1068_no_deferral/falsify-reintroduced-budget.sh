#!/usr/bin/env bash
# Falsify `an_ineligible_cycle_is_blocked_from_the_first_push` (issue #1068).
#
# A guard that never fails is not a guard. This reintroduces the smallest
# possible version of the removed budget -- a day threshold that appends its own
# text to the refusal, which is exactly the shape #1065 had -- and asserts the
# test goes red on it, then restores the file and asserts it goes green again.
#
# Usage: experiments/issue_1068_no_deferral/falsify-reintroduced-budget.sh
set -euo pipefail

root=$(git rev-parse --show-toplevel)
policy="$root/scripts/self-development-loop.rs"
test_name=specification::self_hosting_metric::an_ineligible_cycle_is_blocked_from_the_first_push
backup=$(mktemp)
cp "$policy" "$backup"
restore() { cp "$backup" "$policy"; rm -f "$backup"; }
trap restore EXIT

python3 - "$policy" <<'PATCH'
import sys

path = sys.argv[1]
source = open(path, encoding="utf-8").read()

# The threshold: past seven days the refusal grows budget text, so the answer
# depends on the cycle's age again.
helper = '''
fn reintroduced_budget(repo: &Path, since: &str, reason: String) -> SelfDevelopmentReleaseStatus {
    let Ok(stamp) = git(repo, &["log", "-1", "--format=%ct", since]) else {
        return SelfDevelopmentReleaseStatus::Blocked(reason);
    };
    let Ok(tagged) = stamp.trim().parse::<u64>() else {
        return SelfDevelopmentReleaseStatus::Blocked(reason);
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_secs();
    let days = now.saturating_sub(tagged) / 86_400;
    if days >= 7 {
        return SelfDevelopmentReleaseStatus::Blocked(format!(
            "{reason}. This deferral has outlived its budget: {days} days"
        ));
    }
    SelfDevelopmentReleaseStatus::Blocked(reason)
}

pub fn self_development_release_status('''

anchor = "\npub fn self_development_release_status("
assert source.count(anchor) == 1, "helper anchor is not unique"
source = source.replace(anchor, helper, 1)

# `Ok(Blocked(format!(..)))` and `Ok(reintroduced_budget(repo, since, format!(..)))`
# close the same number of parentheses, so only the head has to be rewritten.
call = "Ok(SelfDevelopmentReleaseStatus::Blocked(format!("
assert source.count(call) == 2, f"expected two refusal sites, found {source.count(call)}"
source = source.replace(call, "Ok(reintroduced_budget(repo, since, format!(", 1)

open(path, "w", encoding="utf-8").write(source)
print("patched: threshold reintroduced at the first refusal site")
PATCH

grep -q "Ok(reintroduced_budget(repo, since, format!(" "$policy" ||
  { echo "patch did not reach the call site" >&2; exit 1; }

echo "== with the budget reintroduced: the guard must go RED =="
if CI=1 cargo test --test unit -- "$test_name" --exact >/tmp/issue-1068-red.log 2>&1; then
  echo "FALSIFICATION FAILED: the test passed with a budget reintroduced" >&2
  tail -30 /tmp/issue-1068-red.log >&2
  exit 1
fi
grep -E "panicked at|left:|right:|test result" /tmp/issue-1068-red.log | head -10

restore
trap - EXIT

echo "== with the budget removed again: the guard must go GREEN =="
CI=1 cargo test --test unit -- "$test_name" --exact 2>&1 | tail -3
