use super::*;
use base64::Engine as _;
use serde_json::json;
#[test]
fn javascript_vectors_match_all_cpu_paths_and_streaming() {
    let vectors: Value =
        serde_json::from_str(include_str!("../fixtures/provenance-native-vectors.json")).unwrap();
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
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/provenance-native-vectors.json")).unwrap();
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
    let fixture: Value =
        serde_json::from_str(include_str!("../fixtures/provenance-native-vectors.json")).unwrap();
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
