# Formatter report publication

The preflight records current source identities, the full 158-item duplicate/history inventory and published regression-byte checks. The immutable downstream test and logs are published before the report is filed. The posted body is verified byte-for-byte and recorded with its SHA-256; the source defect remains unpatched upstream.

The first fresh-check command incorrectly requested paginated commit objects as a single JSON value. It failed while decoding metadata, before any issue creation. The corrected preflight reads the exact Git reference, paginates only the issue list and succeeds. Both command transcripts are retained; this harness correction is not a product finding.
