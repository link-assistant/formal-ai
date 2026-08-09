# Issue 917 Solution Plan

## R917-1 and R917-3: Statement Meaning And Round Trips

Represent the smallest useful semantic statement as predicate, subject, and
object meaning IDs. Formalize the supported natural statements through their
multilingual lexemes and semantic roles, render FOL from the same statement,
then parse the FOL result and require every natural target to recover the
original meaning. Keep predicate/entity role qualification during inverse
Wikidata lookup so duplicate grounding IDs cannot select the wrong meaning.

## R917-2 and R917-5: Seed-Defined Concrete Syntax

Add a projection catalog that declares formal-language names, aliases, and
templates plus each natural language's statement word order and relation
surface. Interpret placeholders generically. A new concrete syntax should add
one formalizer and renderer definition, never an arm for every source-target
pair.

## R917-4: Whole-Engine And Browser Parity

Recognize formal aliases in translation requests before the atomic-meaning and
proof-program paths. Return the projected surface with stable meaning and
language evidence. Compile the corresponding interpreter to WASM and pass the
same seeds across the browser worker boundary. Exercise both directions in the
native engine and a real local Chromium session.

## R917-6 and R917-7: Traceability And Self-Hosting

Preserve raw GitHub feedback, related work, official design references, the
requirement map, architecture, roadmap, and a minor changelog fragment. Drive
one invariant documentation leaf through `formal-ai serve` and the installed
Agent CLI, retain the raw session evidence, and replay its exact byte comparison
in CI.
