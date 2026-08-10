# Proposal: lower GraphQL operations into a shared executable query IR

## Background

While implementing link-foundation/formal-ai#708, we audited meta-language v0.56.0
at `17fc9591dd48c1f5240d87baa1b535ada122b8fe`. Issue #50 added a GraphQL syntax
grammar and schema fixture. That supports full-match CST validation, but it does
not map a GraphQL query or mutation to executable, language-neutral query
concepts.

## Gap

A consumer currently has to implement its own semantic interpretation of root
fields, arguments, input objects, projections, filters, sorting, pagination,
aggregations, and mutations. Consequently, a GraphQL operation cannot share a
canonical executable plan with an equivalent SQL statement, and source-CST
evidence is not connected to execution semantics.

## Proposed generalized capability

Add a public GraphQL semantic adapter that lowers validated operations into the
same language-neutral query-plan concept/links representation used by SQL and
future query frontends. Issue #187 proposes that shared representation and its
SQL adapter. The mapping from schema/root/field names to canonical operations and
domain fields should be an explicit registry extension rather than hardcoded
into the grammar.

The shared representation should cover queries and mutations, projection,
boolean filters, ordering, pagination, grouping, and common aggregates while
retaining GraphQL CST, operation, and source-location evidence.

## Suggested acceptance criteria

- A documented extension point maps GraphQL schema/root/field names to shared
  query-plan concepts.
- Validated queries and mutations lower into the common executable IR.
- Boolean filters, projection, sorting, pagination, grouping, and common
  aggregates have canonical mappings.
- Unsupported or ambiguous schema mappings fail closed.
- CST/source provenance remains attached to the canonical plan.
- Rust and JavaScript implementations have semantic-parity fixtures.
- Conformance fixtures prove that equivalent GraphQL and SQL operations lower to
  the same canonical plan.

## Current downstream workaround

formal-ai#708 / formal-ai#883 validates complete GraphQL input with meta-language,
then applies a local schema-aware semantic adapter and lowers the resulting typed
plan to bounded Links Notation substitution programs. The public registry is
sufficient to add this locally, but a shared upstream adapter would prevent each
consumer from recreating the same semantic bridge.
