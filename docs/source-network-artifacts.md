# Complete source network artifacts

The source network jobs in Layered CI serialize every tracked production source under rust/src, js and ts through the shared pinned meta-language JavaScript API. Vendored files, deployed seeds and real distribution bundles are outside that owned-source set. This is a lossless representation of source bytes; it does not claim that every source construct translates to another programming language.

Each of eight disjoint shards parses its sources, writes the full canonical network with toLino, decodes it with fromLino and checks exact UTF-8 bytes. The artifact includes a .lino.gz packet for each source and a receipt with source, serialized and compressed byte lengths and SHA-256 identities. verifySourcePacket decompresses and decodes a downloaded packet and refuses any identity or reconstructed-byte mismatch. A digest alone cannot satisfy the check.

The packet preserves comments, whitespace, source text and the network metadata. Compressed packets are distribution artifacts; the ordinary source files remain readable. The job starts immediately, has a fifteen-minute limit and leaves native compilation to CI.

The native default-feature suite independently verifies every embedded Rust source through network_lino, render_network_source and exact bytes in eight exhaustive shards. Engine-disabled builds explicitly refuse network evidence. The existing twenty-file sample and optional in-memory exhaustive test remain available.

Run the checked producer from JavaScript:

    node scripts/check-source-networks.mjs --meta-language /path/to/pinned/meta-language --shard-count 8 --shard-index 0

To persist full packets, add --write --output /path/to/packet-directory. The output path contains full serialized documents plus source-network-receipts.json. The script verifies the upstream commit against every serializer checkout in Layered CI and rejects inconsistent pins. It records working-tree inputs separately from a committed source head.

Stable releases rebuild the eight packets from the resolved published tag, verify all shard identities, gzip/network hashes and exact source coverage against that tag, then attach formal-ai-source-networks-0.tar.gz through formal-ai-source-networks-7.tar.gz. The workflow listens to both release publication and completion of the main CI/CD Pipeline, so a release created with GITHUB_TOKEN still schedules its packets. Pull requests only produce check artifacts. Actual stable attachment remains to be observed after merge.

Current verification: eight nearest JavaScript contract checks and four actual upstream module round trips pass. The new exhaustive CI jobs and native shards still require an observed successful run before the requirement can be declared delivered. The committed self-AST census continues to describe signatures and selected AST histograms; it is distinct from these complete serialization packets.
