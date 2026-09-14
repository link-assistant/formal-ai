node_path=2.2.1.2

Extracted `result=` values:
.agent-ladder/verified-children/node-2.2.1.2.1.lino:
result=Line 74: L27	Verify every selected node requires an observable proof file with its exact node path.	experiments/issue_1028_agent_cli_ladder/run.sh	grep -q "^node_path=$id$" "$proof"
.agent-ladder/verified-children/node-2.2.1.2.2.lino:
result=Line 3: This is a complete full binary tree, not a flat list. Depth 0 is the root; depths 1–5 contain 2, 4, 8, 16, and 32 nodes respectively. The complete tree therefore contains exactly 63 task formulations (`1 + 2 + 4 + 8 + 16 + 32`).
