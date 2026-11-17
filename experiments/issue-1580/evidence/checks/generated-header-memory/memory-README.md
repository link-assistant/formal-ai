# Generated-file inventory memory regression

The complete merged-head gate run failed only its generated-file checker under a 256 MiB Node heap cap. The checker loaded every eligible file in full and retained the whole array, although generator detection uses only the first twelve lines. Compressed evidence is eligible under the existing filter, so full-file retention includes large binary captures decoded as text.

The existing repository-wide generated-file test now runs the actual CLI with a 96 MiB Node heap and a 30-second child deadline. With the original checker it fails after 3.407 seconds: three unit cases pass and the full inventory aborts with heap exhaustion. Header-only reads without lazy consumption still fail: six cases pass and the CLI exhausts its 96 MiB cap after 16.273 seconds. Both negative logs are retained.

The final checker reads complete UTF-8 headers in chunks and consumes one file at a time. It retains the original twelve-line detection rule, exclusions, unreadable-file handling, CI registration checks and stale-exception failures. The complete eight-case suite passes with zero skips in 8.594 seconds; recorded child peak RSS is 113,332 KiB. Cases cover a UTF-8 character split across chunks, line twelve versus thirteen, CRLF, EOF, empty/missing files, unchecked generators and stale exceptions through a lazy inventory.

This is a supplied Codex implementation and command result, with zero independent Formal AI authorship credit. No Rust compilation or hosted result is claimed here. Full negative and positive logs and raw SHA-256 identities accompany this document.
