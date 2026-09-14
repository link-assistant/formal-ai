node_path=2.1.2.2

Extracted `result=` values:
.agent-ladder/verified-children/node-2.1.2.2.1.lino:
result=Line 70: L23	Verify the ladder can execute the 32 smallest leaves before moving to larger composite nodes.	experiments/issue_1028_agent_cli_ladder/run.sh	levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]
.agent-ladder/verified-children/node-2.1.2.2.2.lino:
result=Line 71: L24	Verify the ladder order for all mode is 32, 16, 8, 4, 2, then the root.	experiments/issue_1028_agent_cli_ladder/run.sh	levels=list(range(5,-1,-1)) if mode=='all' else [int(mode)]
