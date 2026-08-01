# Dedicating AI-assisted output to the public domain

Reviewed on 2026-08-01. This is a conservative project workflow, not legal advice. Copyright, contract, privacy, database, publicity, moral-right, patent,
and trademark rules vary by material and jurisdiction.

## Short answer

You can dedicate only rights that you own or are authorized to license. The
repository uses the [Unlicense](../../LICENSE): it attempts a public-domain
dedication and supplies permissive fallback terms where that dedication is not
effective. A contributor can apply it to their human-authored code,
documentation, selection, arrangement, and modifications.

An AI provider's assignment of output does not create rights the provider never
had, clear third-party rights, or cancel the provider contract. Purely generated
material may also lack copyright protection in some jurisdictions. Therefore
“the provider says I own the output” and “I can release this artifact with no
conditions” are different statements.

## OpenAI output: ownership is not training permission

OpenAI's current business agreement says, “As between you and OpenAI,” the
customer owns Output and OpenAI assigns any interest it has. The same agreement
also restricts using Output to develop competing AI models, subject to its
specified permitted exceptions. The current European consumer terms similarly
assign output to the user to the extent permitted by law while retaining
service-use restrictions and warning that output may not be unique.

Consequently, an ownership clause does not override a competing AI model or
automated-extraction restriction. Hosted proprietary OpenAI output must not be
used for Formal AI model training, distillation, or service extraction unless a
written agreement or an exact contractual exception affirmatively covers the
account, route, purpose, and release plan.

The Apache-2.0 release of local `gpt-oss` weights is a separate artifact from a
hosted OpenAI service. Its weight license does not convert hosted API output into
Apache-licensed material. Review the exact acquisition route every time.

Primary terms checked:

- [OpenAI Services Agreement](https://openai.com/policies/services-agreement/),
  updated 2025-12-01 and effective 2026-01-01;
- [OpenAI Europe Terms of Use](https://openai.com/policies/eu-terms-of-use/),
  updated 2026-01-16; and
- [OpenAI's `gpt-oss` announcement](https://openai.com/index/introducing-gpt-oss/),
  published 2025-08-05.

Different account types and negotiated agreements can have different terms.
Preserve the applicable dated terms or digest in the source review.

## Copyright and human authorship

The [U.S. Copyright Office AI initiative](https://www.copyright.gov/ai/) and
its copyrightability report explain that copyright protects human-authored
expression, while prompts alone generally do not make the user the author of
all generated expression. Human selection, coordination, arrangement, or
modification can be protectable when it contains sufficient human authorship.
That United States position is not a worldwide rule.

Even when raw output is not copyrightable, it can reproduce protected text,
code, images, characters, or personal information. A public-domain label does
not erase those third-party rights. Similar outputs may also be generated for
other users, so exclusivity must not be promised.

## Fail-closed release checklist

Before dedicating an AI-assisted artifact:

1. Record the human contributors, generating model, exact version, provider
   route, acquisition date, input sources, and applicable terms.
2. Confirm the contributor owns or can license every human-authored element.
3. Confirm the provider contract permits this use and distribution. For
   parameter-updating use, complete
   [`source-review.md`](source-review.md) and obtain an `approved` registry
   decision first.
4. Review output for copied expression, incompatible code or data licenses,
   names and marks, patents, confidential information, and personal data.
5. Separate third-party components and reproduce their notices and licenses.
   Never relabel them as Unlicense or public domain.
6. Preserve meaningful human review and edits. Describe that contribution
   accurately rather than inventing authorship.
7. Apply the repository Unlicense only to the rights actually controlled by the
   contributors, with a provenance note for generated portions.
8. Apply the fail closed rule if any material right or contractual permission
   is unknown.

Suggested provenance note:

> This artifact was prepared with the identified tool and reviewed and modified
> by the listed contributors. The contributors dedicate only the rights they
> own under the repository Unlicense. Third-party material retains its terms;
> no warranty of copyrightability, exclusivity, or non-infringement is made.

That notice documents intent; it is not a cure for missing rights.
