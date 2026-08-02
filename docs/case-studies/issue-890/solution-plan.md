# Issue 890 Solution Plan

## R890-1: Separate Meaning From Presentation

Introduce a small proof intermediate representation for the interval family
already solved by issue #403. Store the variable, lower and upper bounds,
inclusivity, satisfiability result, and witness. Give that value a canonical
statement and stable meaning slug; neither operation should depend on the
requested programming language.

## R890-2 and R890-3: General Translation And Execution

Extend `CodeMeaning` with the proof value. Formalize the canonical proof
statement before the existing function recognizers, then render the meaning by
target slug. Rust and Python presentation templates live in
`data/seed/proof-program-templates.lino` and each emit a complete program that:

1. assigns the proof witness;
2. asserts both semantic bounds; and
3. prints the witness.

The translation handler continues to call `translate_program`, so the route is
`proof statement -> CodeMeaning::FormalProof -> target`, never a direct
natural-language/target pair. Compile Rust with `rustc`; run both results and
assert identical output.

## R890-4 and R890-5: Language Matrix And Browser Parity

Place proof extraction before the command-word gate, because Hindi and Chinese
may put the translation command after the quoted proof. Resolve target aliases
with script-aware boundaries so a Latin alias beside Han characters works
without making substrings such as `trust` match Rust. Load the same proof
presentation templates in the browser worker and mirror only the proof
formalizer and slot expansion, then exercise English, Russian, Hindi, and
Chinese requests against the live language registry and the local Playwright
demo.

## R890-6 and R890-7: Traceability And Self-Hosting

Preserve GitHub snapshots, related issues, official technical sources, the
requirement map, architecture, roadmap, and a minor changelog fragment. Drive a
small documentation leaf through the real Agent CLI against `formal-ai serve`,
capture the raw stream and server log, commit the exact artifact with session
trailers, and rerun the same byte comparison in CI.
