Parent: #651

## Motivation and evidence

Issue #529 required "**fully Turing-complete**" natural-language memory queries — arbitrary link-cli-style substitution programs over the memory store, expressed in natural language. PR #597 (the third attempt, after the maintainer wrote "My requirements are ignored 2-nd time" on #590's line) delivered exactly **two** directive shapes: append and single substitution. The general capability — NL requests that compose into arbitrary read/transform/write programs over links (bounded, permissioned) — remains undelivered. Related dropped asks: #254/#302 general NL memory queries from arbitrary prompts; #144/#145 list/read/update behavior rules via chat.

## Requirements

1. **Query/program algebra over links**: define the closed set of primitive operations (match pattern, create, update, delete-with-retraction-protocol, map over matches, filter, compose sequentially, bounded iterate-to-fixpoint) — the `link-cli` substitution vocabulary made compositional. Each primitive is seed-registered and permission-tagged (reads free; writes gated as today).
2. **NL → program compilation**: natural-language memory requests compile through the meta language into programs of these primitives (reusing the #674/E55 skill-compilation machinery), with the compiled program shown in the trace and a `program_gap` honest failure when a step has no primitive.
3. **Boundedness**: every program carries explicit bounds (max matches, max iterations from `max_decomposition_depth`-style knobs) so Turing-completeness in expressiveness never becomes an unbounded runtime; exceeding a bound is an honest reported stop.
4. **Multilingual**: the same request in en/ru/hi/zh compiles to the same program links (issue #386 convention).
5. **Round-trip with link-cli**: programs serialize to/parse from the `replace x y` / `when n do m` Links Notation shapes so a power user can inspect, edit, and re-run them as data.

## Acceptance criteria

- ≥ 15 query families pass, including at least: "list every fact I contributed about X and rename X to Y in all of them", "count events per topic this week and store the summary", "for every meaning without a Russian label, add a todo link" — each showing the compiled program in the trace.
- The same family phrased in 4 languages compiles to identical program links (fixture-pinned).
- A deliberately unbounded request stops at its bound with an honest report; a destructive request without permission is refused with the standard gate.

## Dependencies

- Blocked by #674 (E55 skill compilation is the NL→program engine).
- Related: #663/E44, #559 (closed), the E57-style handler migration (memory queries become data-driven programs, not handlers).

## Process

Collect data to `docs/case-studies/issue-{id}` (link-cli operation census, prior art: query-by-natural-language over triple stores, Datalog boundedness results); single PR until every requirement is addressed.

