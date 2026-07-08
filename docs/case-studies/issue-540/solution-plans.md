# Issue 540 Solution Plans

## Plan A: Pure Planner First

Add a deterministic planner over the existing `MemoryEvent` log. It classifies
events, estimates reclaimable bytes, recalculates usage counts from evidence,
and emits an ordered action list. This is the selected plan because it gives
tests and users an inspectable result before any mutation.

Acceptance gates:

- duplicate recomputable events are compared by recalculated usage;
- retained raw/learning events are never selected;
- the 20% free-space target can be evaluated with capacity/free inputs;
- insufficient reclaimable space is reported as a storage migration need.

## Plan B: Explicit Apply

Apply the planner only through an explicit helper and CLI switch. This preserves
the repository's existing safety rule: normal memory is append-only, physical
deletion requires confirmation, and callers should make a full-memory backup.

Acceptance gates:

- `formal-ai memory dream` prints a plan by default;
- `--apply` refuses to write without `--confirm`;
- `--backup` writes the same `formal_ai_bundle` backup shape used by reset and
  purge-deleted;
- tests prove apply removes only selected event ids.

## Plan C: Desktop Background Scheduler

Start a plan-only scheduler in the Electron main process. It should run after a
delay, repeat infrequently, unref timers and child processes, and use OS niceness
where practical. The desktop task must not require the renderer to be open or a
user to click a maintenance button.

Acceptance gates:

- default desktop status includes the dreaming scheduler;
- `FORMAL_AI_DESKTOP_DREAMING=off` disables it;
- Unix-like platforms wrap the CLI with `nice -n 19`;
- scheduler tests run without launching Electron.

## Deferred Scale Work

The current implementation plans and can apply maintenance for the `.lino`
memory projection. Future work can push the same policy deeper into the native
doublets store, add persisted usage histograms, and integrate live filesystem
free-space probes per platform. Those are scale improvements on top of the
deterministic policy introduced here, not prerequisites for the issue #540
safety contract.

