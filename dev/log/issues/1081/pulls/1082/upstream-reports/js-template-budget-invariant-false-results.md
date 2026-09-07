# Upstream report 4 - the budget invariant sums alternatives that never run together, and silently ignores budgets written as env-var references

**Target:** `link-foundation/js-ai-driven-development-pipeline-template`

**Severity:** the check in `tests/ci-timeouts.test.js` is the gate that keeps
every step budget expiring before its job cap. It has one false positive that
rejects correct configurations and one false negative that lets an unbounded
step through without a word. Both are in `getStepBudgetSeconds`.

---

## Title

`getStepBudgetSeconds` collects budgets without their step's `if:` condition
and only matches literal integers, so mutually exclusive matrix legs are
double-counted and a `"$VAR"` budget is dropped

## The code

```js
// A step declares its deadline either by wrapping a command in
// run-with-budget-warning.sh or, for `uses:` steps that cannot be wrapped,
// with a step-level timeout-minutes.
function getStepBudgetSeconds(workflow, jobName) {
  const block = getJobBlock(workflow, jobName);
  const wrapped = Array.from(
    block.matchAll(/run-with-budget-warning\.sh\s+(\d+)\s+"([^"]+)"/g),
    (match) => ({ label: match[2], seconds: Number(match[1]) })
  );
  const stepTimeouts = Array.from(
    block.matchAll(/^[ ]{8}timeout-minutes:\s*(\d+)\s*$/gm),
    (match) => ({
      label: 'step timeout-minutes',
      seconds: Number(match[1]) * 60,
    })
  );

  return [...wrapped, ...stepTimeouts];
}
```
(`tests/ci-timeouts.test.js:68`)

and the caller:

```js
      const totalSeconds = budgets.reduce(
        (sum, budget) => sum + budget.seconds,
        0
      );

      if (totalSeconds > allowedSeconds) {
        violations.push(
          `${jobName}: budgets total ${totalSeconds}s, exceeding ${allowedSeconds}s (${MAX_BUDGET_SHARE_PERCENT}% of the ${backstop}min backstop)`
        );
      }
```
(`tests/ci-timeouts.test.js:181`)

Both regexes scan the whole job block. Neither one ever looks at the `if:` that
decides whether a step runs, and the first only matches a literal `\d+`.

---

## Defect A - alternatives are summed as if they were a sequence

### Why it is wrong

The `test` job is a matrix over three runtimes, and each budget is guarded:

```yaml
      - name: Run tests (Node.js)
        if: matrix.runtime == 'node'
        run: bash scripts/run-with-budget-warning.sh 300 "Node.js test suite" npm test
      - name: Run tests (Bun)
        if: matrix.runtime == 'bun'
        run: bash scripts/run-with-budget-warning.sh 200 "Bun test suite" bun test --timeout 30000
      - name: Run tests (Deno)
        if: matrix.runtime == 'deno'
        run: bash scripts/run-with-budget-warning.sh 100 "Deno test suite" deno test --allow-read
```

A single job in that matrix is one runtime. `matrix.runtime == 'node'` and
`matrix.runtime == 'bun'` are never both true in the same job, so those budgets
never share a job clock. The largest a `test` job can ever spend is **300s**.
The check computes **600s**.

The cap is `timeout-minutes: 15`, so `allowedSeconds` is 630. Today 600 <= 630
and the suite is green -- the defect is latent, and the real headroom is
misreported by a factor of eleven: 30s under the check's arithmetic, 330s in
fact.

### Reproducible example

Raise the Node.js budget by 31 seconds -- a change that cannot make any job
exceed anything, because 331 is still far under the 630s allowance for the one
leg that runs:

```console
$ sed -i 's|budget-warning.sh 300 "Node.js test suite"|budget-warning.sh 331 "Node.js test suite"|' \
    .github/workflows/release.yml
$ node --test tests/ci-timeouts.test.js
      error: 'Expected ["test: budgets total 631s, exceeding 630s (70% of the 15min backstop)"] to equal []'
# pass 6
# fail 1
```

The pipeline is rejected for a total that no job can incur.

### How it looks at scale

Downstream, `link-assistant/formal-ai` has a `test` job whose matrix has a
`full` leg and a `specification` leg:

```
release.yml::test  cap=65m(3900s)  naive_sum=5100  grouped_sum=2640  grouped=67.7%
        780s   Check LiNo data integrity              if=matrix.test-suite == 'full'
        300s   Check self-AST census freshness        if=matrix.test-suite == 'full'
       1440s   Run tests                              if=matrix.test-suite == 'full'
       1800s   Build the specification test binary    if=matrix.test-suite == 'specification'
        660s   Run specification tests                if=matrix.test-suite == 'specification'
        120s   Run doc tests                          if=matrix.test-suite == 'full'
```

The real worst case is 2640s, 67.7% of the cap and inside the 70% rule. This
template's arithmetic reports 5100s and rejects it. There is no way to satisfy
the naive rule other than deleting budgets, which is the opposite of what the
rule is for.

---

## Defect B - a budget the regex cannot read is silently dropped

### Why it is wrong

`/run-with-budget-warning\.sh\s+(\d+)\s+"([^"]+)"/` requires a literal integer
immediately after the script path. The rust and python templates -- the same
family, the same script -- write the budget as an env-var reference instead:

```yaml
          bash scripts/run-with-budget-warning.sh "$TEST_BUDGET_SECONDS" "Test suite"
```
(`rust-ai-driven-development-pipeline-template/.github/workflows/release.yml:475`)

That form does not match. `getStepBudgetSeconds` returns nothing for the step,
the reducer adds nothing, and the invariant reports success for a step it never
examined. The enclosing assertion does not save it either: `jobsWithBudgets` is
pinned to `['docker-build', 'docker-publish-build', 'release', 'test']`, so the
job disappears from the list only if *every* budget in it becomes unreadable.
Convert one of three and the job stays in the list, one step lighter, with no
diagnostic.

### Reproducible example

Give the Deno step a 900-second budget in the env-var form. 900s is over the
630s allowance, and it is also equal to the job's entire 15-minute cap -- so
the budget can never expire before the backstop, which is the exact condition
this test exists to prevent:

```console
$ # in .github/workflows/release.yml, replace the Deno step's run: with
$         env:
$           DENO_TEST_BUDGET_SECONDS: '900'
$         run: |
$           bash scripts/run-with-budget-warning.sh \
$             "$DENO_TEST_BUDGET_SECONDS" "Deno test suite" deno test --allow-read

$ node --test tests/ci-timeouts.test.js
# pass 7
# fail 0
```

Green, with a step whose budget cannot fire before the cap it is supposed to
beat.

---

## Workaround

Write every budget as a literal integer on the same line as the script path,
and give every conditional budget enough headroom that the sum of all
alternatives still fits in 70% of the cap. That keeps the check honest at the
cost of sizing budgets against a number that has no operational meaning; it
stops working as soon as a matrix grows a third leg.

## Suggested fix in code

The patch is attached as `js-template-budget-condition-grouping.patch` and
applies to `tests/ci-timeouts.test.js` alone. Three changes:

**1. Keep each budget with its condition.** Split the job block on `\n      - `
so a step's `if:` travels with its budget, and record `condition` on each
entry.

**2. Sum what can actually run together.** Every unconditional budget, plus the
largest single conditional group:

```js
function getConcurrentBudgetSeconds(budgets) {
  const groups = new Map();

  for (const budget of budgets) {
    groups.set(
      budget.condition,
      (groups.get(budget.condition) ?? 0) + budget.seconds
    );
  }

  const unconditional = groups.get('') ?? 0;
  const conditional = Array.from(groups.entries())
    .filter(([condition]) => condition !== '')
    .map(([, seconds]) => seconds);

  return unconditional + (conditional.length > 0 ? Math.max(...conditional) : 0);
}
```

Grouping by the condition *text* is deliberately conservative: two steps
guarded by literally the same expression are summed, and two guarded by
different expressions that happen to be simultaneously true are treated as
alternatives. The second case would need expression evaluation to decide. The
conservative direction here is the safe one -- it can only under-count against
conditions the author wrote as distinct, and the per-budget check
(`budget.seconds > allowedSeconds`) still bounds each one individually.

**3. Report a budget you cannot read, instead of skipping it.** Accept
`"$VAR"`, resolve it from the step's `env:`, then the job's, then the
workflow's -- and if any `run-with-budget-warning.sh` invocation still cannot
be parsed, make that itself a violation:

```js
      for (const budget of budgets) {
        if (!Number.isFinite(budget.seconds)) {
          violations.push(
            `${jobName}: "${budget.label}" declares a budget this check cannot read (${budget.source}); the step is not covered by the invariant`
          );
        }
      }
```

This is the part that matters most. A parser that silently ignores what it does
not understand converts every future syntax change into a gap nobody sees; one
that fails loudly converts it into a one-line fix.

## Verification

Seven cases, run against the template at
`../references/templates/js-template.HEAD`:

| # | scenario | before the patch | after |
|---|---|---|---|
| 1 | pristine template | pass | pass |
| 2 | Node.js budget 300 -> 331 (exclusive legs) | **fail** (false positive) | pass |
| 3 | Node.js budget 300 -> 700 (one leg over the allowance) | fail | fail |
| 4 | `Release dependency install` 240 -> 1500 (unconditional) | fail | fail (grouped total 1920s) |
| 5 | Deno budget as `"$VAR"` = 900s (over cap) | **pass** (false negative) | fail |
| 6 | Deno budget as `"$VAR"` = 100s (within allowance) | pass | pass |
| 7 | `"$VAR"` defined nowhere | **pass** (false negative) | fail, naming the step |

Cases 3 and 4 are the regression check: the fix must not soften the invariant.
Case 4 in particular still fails, and now for the right total -- 1500s
unconditional plus the largest conditional group (420s) rather than a flat sum.
