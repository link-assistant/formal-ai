# Serving path vs. in-process planner (#1066)

`examples/issue_1066_ladder_node_offline` replays `plan_chat_step` in process.
`experiments/issue_1028_agent_cli_ladder/run.sh` drives the same planner through
`formal-ai serve --agent-mode` and a real Agent CLI. When the two disagree about
a node, the disagreement is either a real difference in the serving path or a
binary that no longer matches the tree it is being compared against, and telling
those apart needs a probe that asks the *binary* for its plan.

`probe.sh` starts the release binary in a throwaway directory, advertises the
same fourteen tools the Agent CLI advertises, posts one prompt, and prints the
assistant turn it plans together with every `general_change_plan` trace the run
emitted (`FORMAL_AI_TRACE_REQUESTS=1`).

    cargo build --release --bin formal-ai
    printf 'Write `notes/x.md` containing alpha. The first line must be exactly id=7.\n' > /tmp/p.txt
    bash experiments/issue_1066_served_route/probe.sh /tmp/p.txt

The request, the response and the server log are kept under `$OUT`
(default `/tmp/issue-1066-served-route`) so a reader can check the plan rather
than trust the summary.
