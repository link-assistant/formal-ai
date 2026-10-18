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
mod tests {
    use super::*;
    use base64::Engine as _;
    use serde_json::json;
    #[test]
    fn javascript_vectors_match_all_cpu_paths_and_streaming() {
        let vectors: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/provenance-native-vectors.json"
        ))
        .unwrap();
        for vector in vectors["hashes"].as_array().unwrap() {
            let bytes: Vec<u8> = (0..vector["size"].as_u64().unwrap())
                .map(|index| (index % 251) as u8)
                .collect();
            for mode in [
                CpuHashMode::Serial,
                CpuHashMode::Parallel,
                CpuHashMode::Auto,
            ] {
                let actual = crate::source_fetch::hex_lower(&digest(&bytes, mode));
                assert_eq!(actual, vector["digest"].as_str().unwrap());
                let (streamed, count) = digest_reader(bytes.as_slice(), mode).unwrap();
                assert_eq!(streamed, digest(&bytes, mode));
                assert_eq!(count, bytes.len() as u64);
            }
        }
    }
    #[test]
    fn real_javascript_ed25519_fixture_is_bidirectional_and_tamper_evident() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/provenance-native-vectors.json"
        ))
        .unwrap();
        let decode = |key: &str| {
            base64::engine::general_purpose::STANDARD
                .decode(fixture[key].as_str().unwrap())
                .unwrap()
        };
        let private = decode("privateKey");
        let public = decode("publicKey");
        let message = decode("message");
        let signature = decode("signature");
        assert_eq!(public_key(&private).unwrap(), public);
        verify_message(&public, &message, &signature).unwrap();
        assert_eq!(sign_message(&private, &message).unwrap(), signature);
        let mut corrupt = message.clone();
        corrupt[0] ^= 1;
        assert!(verify_message(&public, &corrupt, &signature).is_err());
        assert!(
            verify_message(
                &public_key(&generate_private_key().unwrap()).unwrap(),
                &message,
                &signature
            )
            .is_err()
        );
        assert!(verify_message(&public[..public.len() - 1], &message, &signature).is_err());
        assert!(verify_message(&public, &message, &signature[..63]).is_err());
    }
    #[test]
    fn shared_codec_domain_and_unicode_parity() {
        let fixture: Value = serde_json::from_str(include_str!(
            "../tests/fixtures/provenance-native-vectors.json"
        ))
        .unwrap();
        assert_eq!(
            crate::source_fetch::hex_lower(
                &canonical_hash(
                    "formal-ai-provenance-record-v1\0",
                    &fixture["canonicalValue"]
                )
                .unwrap()
            ),
            fixture["canonicalDigest"]
        );
        assert_ne!(
            canonical_hash("different\0", &json!({"x": 1})).unwrap(),
            canonical_hash("formal-ai-provenance-record-v1\0", &json!({"x": 1})).unwrap()
        );
        assert!(canonical_hash("unterminated", &json!({})).is_err());
    }
}
