//! Native provenance primitives backed by official SIMD/rayon BLAKE3 and Ed25519.
//! Signature verification authenticates a pinned installation key, never authorship.
use ring::rand::SystemRandom;
use ring::signature::{self, Ed25519KeyPair, KeyPair};
use serde_json::Value;
use std::io::{self, Read};

/// CPU implementation selection; all variants have exactly the same digest.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpuHashMode {
    /// Runtime-dispatched SIMD within the calling thread.
    Serial,
    /// Rayon parallel chunk hashing with runtime-dispatched SIMD.
    Parallel,
    /// Keep small messages on the caller; parallelize larger buffers.
    Auto,
}
const PARALLEL_THRESHOLD: usize = 128 * 1024;
const STREAM_BUFFER: usize = 8 * 1024 * 1024;
const SPKI_PREFIX: &[u8] = &[
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

fn update(hasher: &mut blake3::Hasher, bytes: &[u8], mode: CpuHashMode) {
    if mode == CpuHashMode::Parallel
        || mode == CpuHashMode::Auto && bytes.len() >= PARALLEL_THRESHOLD
    {
        hasher.update_rayon(bytes);
    } else {
        hasher.update(bytes);
    }
}
/// Hash bytes without changing their representation or ownership.
#[must_use]
pub fn digest(bytes: &[u8], mode: CpuHashMode) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    update(&mut hasher, bytes, mode);
    *hasher.finalize().as_bytes()
}
/// Stream large sources with a bounded eight-MiB buffer and caller-owned reader.
pub fn digest_reader(mut reader: impl Read, mode: CpuHashMode) -> io::Result<([u8; 32], u64)> {
    let mut hasher = blake3::Hasher::new();
    let mut buffer = vec![0; STREAM_BUFFER];
    let mut total = 0_u64;
    loop {
        let count = match reader.read(&mut buffer) {
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            other => other?,
        };
        if count == 0 {
            break;
        }
        total = total
            .checked_add(count as u64)
            .ok_or_else(|| io::Error::other("source_size_overflow"))?;
        update(&mut hasher, &buffer[..count], mode);
    }
    Ok((*hasher.finalize().as_bytes(), total))
}
/// Use the maintained shared typed-Links-Notation codec, including canonical checks.
pub fn canonical_hash(domain: &str, value: &Value) -> io::Result<[u8; 32]> {
    if !domain.ends_with('\0') {
        return Err(io::Error::other("domain_separator_required"));
    }
    let line = crate::dialog_log::structured_log::encode(value)?;
    let decoded = crate::dialog_log::structured_log::decode(&line)?;
    if decoded != *value {
        return Err(io::Error::other("canonical_roundtrip_failed"));
    }
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(line.as_bytes());
    Ok(*hasher.finalize().as_bytes())
}
/// Generate an unencrypted PKCS#8 v2 key with the OS random source.
/// Filesystem custody and explicit trust policy belong to the installation owner.
pub fn generate_private_key() -> io::Result<Vec<u8>> {
    Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())
        .map(|key| key.as_ref().to_vec())
        .map_err(|_| io::Error::other("installation_key_generation_failed"))
}
fn key_pair(private_key: &[u8]) -> io::Result<Ed25519KeyPair> {
    // Node exports RFC8410 PKCS#8 v1 (seed only); ring also accepts v2 with consistency checks.
    Ed25519KeyPair::from_pkcs8_maybe_unchecked(private_key)
        .map_err(|_| io::Error::other("installation_private_key_invalid"))
}
/// Export the exact RFC8410 SPKI bytes used by the JavaScript trust store.
pub fn public_key(private_key: &[u8]) -> io::Result<Vec<u8>> {
    let pair = key_pair(private_key)?;
    let mut result = SPKI_PREFIX.to_vec();
    result.extend_from_slice(pair.public_key().as_ref());
    Ok(result)
}
/// Sign the exact caller-provided message without implicit JSON or string conversion.
pub fn sign_message(private_key: &[u8], message: &[u8]) -> io::Result<Vec<u8>> {
    Ok(key_pair(private_key)?.sign(message).as_ref().to_vec())
}
/// Verify a caller-pinned SPKI key; evidence-carried keys grant no trust.
pub fn verify_message(
    trusted_public_key: &[u8],
    message: &[u8],
    signature_bytes: &[u8],
) -> io::Result<()> {
    if trusted_public_key.len() != SPKI_PREFIX.len() + 32
        || !trusted_public_key.starts_with(SPKI_PREFIX)
        || signature_bytes.len() != 64
    {
        return Err(io::Error::other("installation_seal_encoding"));
    }
    signature::UnparsedPublicKey::new(
        &signature::ED25519,
        &trusted_public_key[SPKI_PREFIX.len()..],
    )
    .verify(message, signature_bytes)
    .map_err(|_| io::Error::other("installation_signature_invalid"))
}

#[cfg(test)]
#[path = "../tests/unit/provenance_native.rs"]
mod tests;
