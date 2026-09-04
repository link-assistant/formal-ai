---
bump: patch
---

### Fixed
- A request that asks for a change *and* for records of that change now produces all of them. The agentic planner's change routes matched such a request whole and answered as soon as the edit landed, so a ladder node made its edit correctly and still failed verification with `missing_proof`, having never written the effect and proof files the same prompt asked for. The route that peels "do this, and leave the answer in FILE" into a delivery plus a residual now runs ahead of the change routes, and the residual carries the change on to them.
- Delivery no longer claims a file the request asked to *edit* as somewhere to put its own answer. Across the 1 118 recorded request sentences that name a file and carry an action cue, 2.68% carry both a write and an edit cue — including delivery sentences — so mention cannot separate them; the leading cue governs the sentence, the same adjacency rule that binds a cue to a path.
