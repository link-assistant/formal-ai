node_path=1.1.2.1.2

The `grep` command completed. Output:

```text
Found 11 matches
/tmp/tmp.LocX0JLKCx/tests/unit/specification/task_decomposition.rs:
  Line 116:         ["task_strategy_verified_change"]
  Line 556:         "task_strategy_verified_change",
  Line 557:         "task_strategy_unreviewed_change",

/tmp/tmp.LocX0JLKCx/src/task_decomposition/learning.rs:
  Line 48:         push_lino_node(&mut out, 0, "task_strategy_proposal", Some(&self.id));
  Line 231:         let id = stable_id("task_strategy_ledger", &body);
  Line 233:         push_lino_node(&mut out, 0, "task_strategy_ledger", Some(&id));
  Line 247:             .find(|node| node.name == "task_strategy_ledger")
  Line 355:         let id = stable_id("task_strategy_review", &identity);
  Line 386:     stable_id("task_strategy_proposal", &identity)

/tmp/tmp.LocX0JLKCx/data/meta/task-decomposition-strategies.lino:
  Line 3:   strategy task_strategy_verified_change
  Line 20:   approved_strategy task_strategy_verified_change
```
