node_path=2.2.1.1

Extracted `result=` values:
.agent-ladder/verified-children/node-2.2.1.1.1.lino:
result=Line 72: L25	Verify every selected node runs in a fresh temporary repository copy.	experiments/issue_1028_agent_cli_ladder/run.sh	work=$(mktemp -d)
.agent-ladder/verified-children/node-2.2.1.1.2.lino:
result=Line 73: L26	Verify every selected node uses the real Agent CLI against the real Formal AI server.	experiments/issue_1028_agent_cli_ladder/run.sh	"$AGENT" --model formalai/formal-ai
