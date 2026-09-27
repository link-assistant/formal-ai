---
bump: patch
---

### Fixed

- The Coverage lane survives a contended host: main run 36269286808
  burned the full 4200s budget with every test passing and no hanger,
  while the identical tree on branch run 36266423193 finished the same
  lane green in 2574s the same day — so `TEST_BUDGET_SECONDS` rose to
  5400 together with the job timeout to 135 minutes, keeping the 66.7%
  budget-to-cap ratio issue #1017 enforces (issue #1149).
