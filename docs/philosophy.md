# Philosophy: intelligence as linked transformation

This document distinguishes Formal AI's design theses from mathematical facts
and present implementation. The shorthand is deliberately ambitious; the
labels keep it falsifiable.

## AI = data + algorithm

This is a **design thesis, not a theorem.** Formal AI models an intelligent system as data
plus algorithms that transform data. Code is executable data in one context;
rules, traces, tests, and source claims are data in another. This decomposition
is useful because both sides can be inspected and linked. It is not offered as
a universally accepted definition of artificial intelligence or consciousness.

In this project, durable knowledge is not a sentence detached from its origin.
It is an associative link network connecting a statement to meanings, sources,
conditions, probabilities, transformations, tests, and consequences. The
network is a meta-algorithm when some links describe how other links may be
read, rewritten, composed, or rejected.

## Everything is a link

**Representation principle.** “Everything is a link” means every object that
must participate in native reasoning should have a link identity and should be
related through doublet links. A number, file, statement, rule, execution, and
larger network can therefore be referenced through the same primitive.

“A link is a fractal of links” is a structural metaphor: a named link can stand
for a network whose components are themselves links, and that nesting can be
repeated. It is not a claim that the mathematical definition of a fractal
applies or that a finite computer materializes an infinite structure.

## Learning is controlled self-modification

**Target behavior.** Learning means a verified change to the data or algorithm
that affects a later solution. Depending on scope, that can be a new source
association, probability, substitution rule, test, handler, or code change.
Merely appending a transcript is experience capture, not demonstrated learning.

Self-modification is human-gated. A failed or surprising execution may propose
a rule or code change; held-out tests, provenance checks, and human review decide
whether it becomes durable. Formal AI does not silently rewrite production code
or promote external model output.

## Transformation and substitution networks

An algorithm can be represented as a transformation network: input-state links
connect through applicable rule links to output-state links. An alternative
algorithm can then be expressed by substituting one verified transformation
subnetwork for another with the same declared boundary. This makes algorithm
transformation inspectable and testable rather than an opaque weight update.

A global transformation can be split into smaller transformations repeatedly
when a useful decomposition exists. **Operational qualification:** the design
space may be unbounded, but every actual run, stored network, and proof attempt
is finite and resource-bounded. “Can split infinitely” describes recursive
decomposability, not a completed infinite computation.

## What Markov algorithms do—and do not—prove

A normal Markov algorithm is an ordered finite collection of string
substitution formulas with a specified execution procedure. The formalism is
computationally universal: complete normal algorithms can express the same
computable functions as other standard universal models. See the
[Encyclopedia of Mathematics entry on normal algorithms](https://encyclopediaofmath.org/wiki/Normal_algorithm)
and A. A. Markov's cited foundational treatment.

The stronger sentence “each Markov substitution is Turing-complete” is false
when read literally. One rewrite rule is only one rule; universality belongs to
the algorithm formed by an adequate ordered set of rules and its control
semantics. Formal AI instead adopts this representation invariant:

> Each recorded substitution occurrence has one link identity, and that link
> connects to the rule, input, output, order, condition, and verification
> networks needed to interpret it.

“Exactly one link” therefore means one canonical identity for an occurrence,
represented by one doublet link in the identity layer—not that all of its
semantics fit inside two bare endpoints or that a single
substitution is universal.

## Relative knowledge

Statements are evaluated relative to sources, time, scope, assumptions, and
other statements. The repository's relative-meta-logic approach records a
probability instead of turning uncertainty into an unsupported binary fact. A
dependent statement cannot be more credible than an unresolved antecedent on
which its interpreted meaning depends.

For prose such as “Formal AI uses symbolic rules. It records their evidence,”
the audit first links “It” to the closest compatible preceding subject inside
the same document, records the resolved claim, and then weighs evidence. The
closest-reference rule is deterministic and reviewable, but it is a conservative
parser heuristic—not complete natural-language understanding. Ambiguity remains
a finding for human review.

## Present boundary

Implemented today: link-native storage, Links Notation data, deterministic
solver paths, transformation traces, statement extraction, local reference
links, evidence-weighted probabilities, and human-gated learning proposals.

Still a direction: representing every algorithm uniformly as substitutable link
networks, generally synthesizing those networks from arbitrary requirements,
and proving broad semantic equivalence between substitutions. The philosophy
guides tests and architecture; it must not be used to claim capabilities the
current code does not implement.
