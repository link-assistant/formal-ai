---
bump: patch
---

### Fixed

- Stop every former of an open-web query at the end of the request. Three
  functions still read a whole prompt when they picked what to search for --
  the stated research subject, an explicit search request, and the planner's
  last-resort route for a request nothing else understood. A prompt whose second
  paragraph only places the worker was therefore sent to a search engine with
  that paragraph attached: "a two node decomposition at depth one this is
  recursive binary tree node 1 2 1 2 1 at depth 5 solve only this node s task in
  this fresh temporary repository ...", a query no source answers. Each now
  reads the block that states the subject, the same scope the search and
  workspace routes already use (#1066).
- Never report a research round that was not run. When the last completed tool
  call belonged to another route -- a workspace search, a file read -- the
  research route took it for a round of its own and composed "Research completed
  for ..., but the tool returned no content." over the result the agent was
  already holding. Five of the #1066 ladder's thirty-two leaves recorded exactly
  that as their evidence, with the matching `grep` output unused in the same
  transcript. The route now plans a further round or stands aside, so the search
  that actually ran is what gets reported (#1066).
