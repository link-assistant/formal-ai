# Handler parity evidence

The original report records 175 of 178 cases passing, including the three observed failures. The retry records 180 of 180 cases passing after the general repairs and two native boundary cases. These are working-tree JavaScript observations; native compilation and remote-head CI remain separate checks.

The complete JSON reports are stored losslessly as gzip files:

- [Original failure report](pr1188-three-case-original-report.json.gz)
- [Actual complete retry report](production-reader-actual-report.json.gz)

[Compressed report manifest](compressed-reports.json) binds each original logical filename to its stored gzip, decoded byte length and SHA256, and compressed byte length and SHA256. Decode with `gzip -dc FILE.json.gz`; the decoded bytes equal the original reports exactly. In [the source observation](pr1188-handler-parity-observation.json), `originalReportSha256` refers to decoded JSON bytes; `originalReportStoredSha256` refers to the gzip file. No outcome, assertion, source identity or failed transcript was edited.
