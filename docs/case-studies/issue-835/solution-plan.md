# Issue 835 solution plan

## Root cause

Formal AI had attachment context and general provenance primitives, but no
file-oriented legal-risk schema, no Exif reader, no jurisdiction policy pack,
and no provider boundary. Returning a boolean from those pieces would conflate
three unrelated questions and falsely imply that absent evidence means legal.

## Implemented sequence

1. Commit a failing contract test for the public function, three categories,
   multiple jurisdictions, Exif provenance, media families, safe confirmed-hash
   behavior, and composed evidence.
2. Add a typed report whose only global verdict is `not_provided`; make each
   category/jurisdiction status and action independently addressable.
3. Inspect the actual file with byte signatures, a streaming SHA-256 for normal
   handling, and `kamadak-exif` for bounded embedded metadata fields and GPS.
4. Compose caller-supplied, versioned jurisdiction policies with detector
   observations. Never turn a negative detector result into a legal clearance.
5. Add the `LegalityEvidenceProvider` adapter boundary. Run adapters
   independently, pin their declared categories and provider identity, and
   retain machine-readable failures.
6. Put an authorized confirmed child-safety receipt ahead of ordinary
   processing: read only enough bytes to identify the media family, suppress
   hash and metadata derivatives, skip adapters, and require provider
   escalation.
7. Expose JSON configuration/reporting through `formal-ai file-legality`, add a
   safe synthetic example, and verify both library and CLI paths.

## Deliberate boundaries

- Formal AI does not ship a global law database, a prohibited-content corpus,
  a reverse-image index, or jurisdiction rules disguised as source code.
- `unknown` means evidence was not supplied or a provider failed; it never
  means permitted. `no_risk_signal_detected` describes only a detector result.
- Policies and observations carry IDs, versions, confidence, provider/source
  URIs, and jurisdictions so callers can review and refresh them.
- The authorized hash receipt is an integration input from a qualified
  provider. This repository neither implements nor imitates PhotoDNA and does
  not accept or preserve prohibited sample content.
- Reports are operational triage artifacts, not legal advice or authorization.
