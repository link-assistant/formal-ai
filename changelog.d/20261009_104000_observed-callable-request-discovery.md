# Observed callable request discovery

Requests declaring a callable and a source destination now read the destination, immutable source operands and named acceptance file before reporting unsupported contracts. The internal outcome retains request-local content identities and typed read failures; it cannot be delivered as executable source. File names and declaration parameters cannot supply authoring cues. Existing arithmetic module/test synthesis is preserved.

The original composition remains unimplemented; this change stops at an honest MissingContract outcome and does not certify callable types, effects, optional guards or rollback.

Validation:21 closest JavaScript checks and5 existing arithmetic route tests pass; both Rust modules pass standalone formatting. Native regression pins await CI compilation.
