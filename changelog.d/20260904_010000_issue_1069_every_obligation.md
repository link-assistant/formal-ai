---
bump: patch
---

### Fixed
- A request that asks for a change *and* for records of that change now produces all of them. The agentic planner's change routes matched such a request whole and answered as soon as the edit landed, so a ladder node made its edit correctly and still failed verification with `missing_proof`, having never written the effect and proof files the same prompt asked for. The route that peels "do this, and leave the answer in FILE" into a delivery plus a residual now runs ahead of the change routes, and the residual carries the change on to them.
- Delivery no longer claims a file the request asked to *edit* as somewhere to put its own answer. The same cues introduce a delivery destination and the file work happens *in*, so "In the file `src/x.rs`, add \"toward \" to the list" used to be read as a place to put an answer -- and the planner wrote its status line over the source it was asked to edit. What separates them is order, the adjacency rule that already binds a cue to a path: a destination is named *after* the write action, an operand before it. Across the 1 118 recorded request sentences that name a file and carry a cue, that reads 21.56% as destinations rather than all of them.
- A route that changes a member list now states the change it made instead of reporting that a file was written. The read step already tells it which of the requested values were missing, so the answer names them; a caller that asked for a record of the change no longer records "Created or updated and observed `path`".
