## Fixed

- Preserve every verified edit in a request sequence when the result is written
  to a structured effect and its supporting proof.
- Accept complete source reads using the host workspace's absolute paths when
  verifying list edits.
- Resolve numbered source requirements to declared Rust lists and canonical
  seed files, and verify each edit before delivering its evidence.
