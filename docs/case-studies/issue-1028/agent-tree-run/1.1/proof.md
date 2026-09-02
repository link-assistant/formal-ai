node_path=1.1

Extracted `result=` values:
.agent-ladder/verified-children/node-1.1.1.lino:
result=Line 79:     pub children: Vec<Self>,; Line 180:     pub max_depth: u8,; together these exact observations compose the result.; Line 89:             && !self.completion_criterion.starts_with("unresolved_"); Line 152:             pairs.push(("child", child.id.clone()));; together these exact observations compose the result.; together these exact observations compose the result.
.agent-ladder/verified-children/node-1.1.2.lino:
result=Line 169:             children: self.children.iter().map(Self::to_recursive_task).collect(),; Line 284:     decompose_task_with_ledger(task, max_depth, &TaskStrategyLedger::shipped()); together these exact observations compose the result.; Line 374:         decomposition.leaves().len() >= 3,; Observed repository result: Line 385:             "unresolved_single_need",; together these exact observations compose the result.; together these exact observations compose the result.
