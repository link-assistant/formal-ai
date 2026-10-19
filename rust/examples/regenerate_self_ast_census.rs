//! Historical entrypoint: count-only source documents are no longer generated.
//! Full recursive source models are produced by generate_source_content and
//! independently reconstructed and reparsed before CI uploads each shard.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("CountOnlySourceAuthorityRetired: run generate_source_content OUTPUT SHARD_INDEX SHARDS; no repository counters were written".into())
}
