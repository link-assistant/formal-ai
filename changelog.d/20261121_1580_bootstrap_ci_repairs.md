### Fixed

- Reject stale worker inputs in local JavaScript gates without compiling Rust;
  remove an unused worker import that stopped warnings-as-errors compilation.
- Preserve consumed immutable fixtures in sparse CI checkouts and use the strict
  artifact extraction adapter for native contracts.
- Retain first-wave scheduling for measured heavy suites and transport both
  receipt-bound native companion binaries required by existing integration tests.
- Schedule all four complete Hive workflow sessions independently under the
  existing execution budgets, and retain real failures in the BuildKit outage
  control without false failure annotations after successful recovery.
- Select the maintained SQLite runtime explicitly and remove the experimental
  timer mock while preserving warning reporting and deadline assertions.
