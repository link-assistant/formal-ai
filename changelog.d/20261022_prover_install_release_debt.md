Temporarily make real Lean/Rocq CI integration opt-in through
`FORMAL_AI_REAL_PROVER_INTEGRATION=true` after both official Ubuntu mirrors
exceeded its bounded cold-install budget on main. Preserve the original
compiler tests and deadlines, avoid optional APT recommendations when enabled,
and record restoration conditions as release debt. Production, coding and
released-client checks remain required.

Follow-up: [PR 1394](https://github.com/link-assistant/formal-ai/pull/1394).
