# Conversation list source headroom

The existing conversation hook module now owns complete list refresh, initial
hydration, deleted-thread visibility and deletion callbacks. Their original
bodies and comments are byte-conserved. App falls from 1,451 to 1,382 lines under
the unchanged 1,400-line warning threshold. Rendered markup is unchanged.

Six focused cases cover source conservation, cached events and titles, deletion
visibility, unavailable/failed memory, selected-thread restoration and unmount,
and append-only deletion. All 307 maintained issue tests, the JSX factory test
and seven affected local gates pass. Cross-realm asynchronous assertions first
failed because two microtasks did not drain the VM callback chain; the corrected
harness drains the event loop once. Initial receipt-marker and non-idempotent
continuation failures remain archived. No Rust build or browser render is claimed.

These supplied edits and checks receive zero independent-authorship credit.
Current hosted CI repairs and the wider issue remain unfinished.
Evidence: `experiments/issue-1580/evidence/checks/conversation-list-hook/`.
Pull request: https://github.com/link-assistant/formal-ai/pull/1581.
