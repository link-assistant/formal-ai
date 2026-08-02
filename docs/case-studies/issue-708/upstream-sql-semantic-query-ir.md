# Proposal: shared executable SQL semantic query IR and vendor normalization

## Background

While implementing link-foundation/formal-ai#708, we audited meta-language v0.56.0
at `17fc9591dd48c1f5240d87baa1b535ada122b8fe`. Issue #11 added SQL syntax
grammar support under `sql-ansi`. That is useful for full-match syntax
validation, but a consumer still has to build its own semantic SQL parser,
vendor-profile mapping, and query model after the CST is returned.

## Gap

There is no public, language-neutral executable query representation that an SQL
frontend can lower into. In particular, a downstream consumer cannot currently
ask meta-language for canonical semantics for:

- `SELECT`, `INSERT`, `UPDATE`, and `DELETE`;
- projection, predicates, grouping, ordering, limits, and offsets;
- `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, population variance, and population
  standard deviation;
- equivalent queries submitted using common ANSI, PostgreSQL, MySQL, SQLite,
  SQL Server, Oracle, BigQuery, and Snowflake profiles; or
- source-CST evidence tied to the resulting executable plan.

This prevents SQL and non-SQL query surfaces from sharing one semantic layer and
makes each consumer repeat dialect normalization and safety validation.

## Proposed generalized capability

Add a public query-plan concept/links representation and an adapter from the SQL
grammars into that representation. The IR should be independent of any database
engine and extensible through the existing public language/registry mechanisms.
It should retain source-language and CST provenance while canonicalizing the
common operation, expression, projection, grouping, sort, pagination, mutation,
and aggregate concepts. Vendor-specific syntax can remain explicit extensions;
the common subset should normalize to identical canonical plans.

This representation should be reusable by other query languages rather than
being SQL-specific.

## Suggested acceptance criteria

- A documented public semantic IR covers the CRUD and query concepts above.
- `sql-ansi` lowers full-match CSTs into the IR with source evidence preserved.
- Supported vendor language keys normalize their common subset to the same IR,
  with a clear extension point for vendor-only constructs.
- Malformed or semantically invalid statements fail closed; syntax acceptance is
  not treated as authorization to execute.
- Rust and JavaScript implementations have conformance fixtures proving semantic
  parity.
- At least one fixture proves that equivalent SQL and a second query frontend can
  produce the same canonical plan.

## Current downstream workaround

formal-ai#708 / formal-ai#883 maps its supported vendor profiles onto
`sql-ansi`, validates the complete input with meta-language, then uses a local
typed semantic parser and shared query plan before lowering the plan to bounded
Links Notation substitution programs. The registry is extensible enough for
this workaround, so this issue requests a reusable upstream semantic layer
rather than a consumer-specific API.
