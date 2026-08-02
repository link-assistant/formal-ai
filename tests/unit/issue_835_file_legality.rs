//! Multi-jurisdiction file-legality capability coverage for issue #835.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::file_legality::{
    check_file_legality, check_file_legality_with, check_file_legality_with_providers,
    AssessmentStatus, AuthorizedHashMatch, DetectorObservation, FileLegalityConfig, LegalCategory,
    LegalVerdict, LegalityEvidenceProvider, MediaFamily, MetadataField, ProviderRunStatus,
    RequiredAction, SafetyDisposition,
};

fn fixture_path(name: &str, bytes: &[u8]) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "formal-ai-file-legality-{}-{nonce}-{name}",
        std::process::id()
    ));
    fs::write(&path, bytes).expect("write legality fixture");
    path
}

fn remove_fixture(path: &Path) {
    fs::remove_file(path).expect("remove legality fixture");
}

#[test]
fn default_check_is_category_complete_but_never_returns_a_blanket_verdict() {
    let path = fixture_path("brief.pdf", b"%PDF-1.7\nsynthetic fixture\n");

    let report = check_file_legality(&path).expect("inspect file");

    assert_eq!(report.verdict, LegalVerdict::NotProvided);
    assert_eq!(report.file.media_family, MediaFamily::Document);
    assert_eq!(report.assessments.len(), LegalCategory::ALL.len());
    for category in LegalCategory::ALL {
        let assessment = report
            .assessment("unspecified", category)
            .expect("one assessment per category");
        assert_eq!(assessment.status, AssessmentStatus::Unknown);
        assert_eq!(assessment.action, RequiredAction::ObtainEvidence);
        assert!(assessment.evidence_ids.is_empty());
    }

    remove_fixture(&path);
}

#[test]
fn observations_are_assessed_independently_per_category_and_jurisdiction() {
    let path = fixture_path("checkpoint.jpg", &[0xff, 0xd8, 0xff, 0xd9]);
    let mut config = FileLegalityConfig::for_jurisdictions(["DE", "GB"]);
    config.add_policy(
        formal_ai::file_legality::JurisdictionPolicy::new(
            "de-sensitive-sites",
            "2026.1",
            "DE",
            LegalCategory::NationalSecurity,
            RequiredAction::LegalReview,
            "https://authority.example/de/policy",
        )
        .with_trigger_code("sensitive-installation"),
    );
    config.add_observation(
        DetectorObservation::risk(
            "scene-1",
            LegalCategory::NationalSecurity,
            "scene-classifier",
            0.92,
            "https://provider.example/scene-1",
        )
        .with_restriction_code("sensitive-installation")
        .for_jurisdiction("DE"),
    );
    config.add_observation(
        DetectorObservation::no_match(
            "symbol-1",
            LegalCategory::ForbiddenContent,
            "symbol-classifier",
            0.88,
            "https://provider.example/symbol-1",
        )
        .for_jurisdictions(["DE", "GB"]),
    );

    let report = check_file_legality_with(&path, &config).expect("run configured pipeline");

    assert_eq!(report.assessments.len(), 6);
    let de_security = report
        .assessment("DE", LegalCategory::NationalSecurity)
        .unwrap();
    assert_eq!(de_security.status, AssessmentStatus::RiskSignal);
    assert_eq!(de_security.action, RequiredAction::LegalReview);
    assert_eq!(de_security.policy_ids, ["de-sensitive-sites"]);
    assert_eq!(de_security.evidence_ids, ["scene-1"]);
    assert!((de_security.confidence - 0.92).abs() < f64::EPSILON);

    let gb_security = report
        .assessment("GB", LegalCategory::NationalSecurity)
        .unwrap();
    assert_eq!(gb_security.status, AssessmentStatus::Unknown);
    assert!(gb_security.evidence_ids.is_empty());

    for jurisdiction in ["DE", "GB"] {
        let forbidden = report
            .assessment(jurisdiction, LegalCategory::ForbiddenContent)
            .unwrap();
        assert_eq!(forbidden.status, AssessmentStatus::NoRiskSignalDetected);
        assert_eq!(forbidden.evidence_ids, ["symbol-1"]);
    }

    remove_fixture(&path);
}

#[test]
fn confirmed_child_safety_hash_match_is_fail_closed_and_content_free() {
    let mut bytes = minimal_exif_tiff();
    bytes.extend_from_slice(b"synthetic-known-illegal-payload-marker");
    let path = fixture_path("unsafe.bin", &bytes);
    let mut config = FileLegalityConfig::for_jurisdictions(["US", "GB"]);
    config.add_authorized_hash_match(AuthorizedHashMatch::confirmed(
        "approved-provider",
        "provider-case-42",
        "https://provider.example/report/provider-case-42",
    ));

    let report = check_file_legality_with(&path, &config).expect("fail-closed report");
    let serialized = serde_json::to_string(&report).expect("serialize report");

    assert_eq!(
        report.safety_disposition,
        SafetyDisposition::RefuseAndEscalateToAuthorizedProvider
    );
    assert!(
        report.file.sha256.is_none(),
        "matched content hash is redacted"
    );
    assert!(
        report.metadata.entries.is_empty(),
        "matched content metadata is not extracted"
    );
    assert!(!serialized.contains("synthetic-known-illegal-payload-marker"));
    assert!(!serialized.contains("FormalCam"));
    for jurisdiction in ["US", "GB"] {
        let forbidden = report
            .assessment(jurisdiction, LegalCategory::ForbiddenContent)
            .unwrap();
        assert_eq!(forbidden.status, AssessmentStatus::ConfirmedProhibitedMatch);
        assert_eq!(
            forbidden.action,
            RequiredAction::RefuseAndEscalateToAuthorizedProvider
        );
        assert_eq!(forbidden.evidence_ids, ["provider-case-42"]);
    }

    remove_fixture(&path);
}

#[test]
fn the_same_pipeline_accepts_images_documents_audio_video_and_other_files() {
    let fixtures: &[(&str, &[u8], MediaFamily)] = &[
        ("photo.jpg", &[0xff, 0xd8, 0xff, 0xd9], MediaFamily::Image),
        ("paper.pdf", b"%PDF-1.7\n", MediaFamily::Document),
        ("sound.mp3", b"ID3\x04\0\0\0\0\0\0", MediaFamily::Audio),
        (
            "clip.mp4",
            b"\0\0\0\x18ftypisom\0\0\0\0isom",
            MediaFamily::Video,
        ),
        ("archive.data", b"\x01\x02\x03\x04", MediaFamily::Other),
    ];

    for (name, bytes, expected_family) in fixtures {
        let path = fixture_path(name, bytes);
        let report = check_file_legality(&path).expect("inspect generalized file");
        assert_eq!(
            report.file.media_family, *expected_family,
            "wrong media family for {name}"
        );
        assert_eq!(report.assessments.len(), 3);
        remove_fixture(&path);
    }
}

#[test]
fn exif_metadata_is_extracted_with_field_level_provenance() {
    let path = fixture_path("metadata.tiff", &minimal_exif_tiff());

    let report = check_file_legality(&path).expect("inspect Exif fixture");

    for (field, value) in [
        (MetadataField::Author, "Ada Example"),
        (MetadataField::Copyright, "CC0 Example"),
        (MetadataField::CameraMake, "FormalCam"),
        (MetadataField::CameraModel, "FC-1"),
        (MetadataField::CapturedAt, "2026:08:02 12:34:56"),
        (MetadataField::GpsLatitude, "40.500000"),
        (MetadataField::GpsLongitude, "-74.000000"),
    ] {
        let entry = report.metadata.get(field).expect("expected Exif field");
        assert_eq!(entry.value, value);
        assert_eq!(entry.provenance.source, "embedded_exif");
        assert!(!entry.provenance.locator.is_empty());
    }

    remove_fixture(&path);
}

#[test]
fn composed_report_retains_versioned_policy_and_evidence_provenance() {
    let path = fixture_path("work.txt", b"original synthetic work");
    let mut config = FileLegalityConfig::for_jurisdictions(["FR"]);
    config.add_policy(formal_ai::file_legality::JurisdictionPolicy::new(
        "fr-copyright-review",
        "2026-08-02",
        "FR",
        LegalCategory::CopyrightAndIp,
        RequiredAction::RightsReview,
        "https://authority.example/fr/copyright",
    ));
    config.add_observation(DetectorObservation::risk(
        "reverse-search-7",
        LegalCategory::CopyrightAndIp,
        "reverse-search-provider",
        0.81,
        "https://provider.example/match/7",
    ));

    let report = check_file_legality_with(&path, &config).expect("run whole pipeline");
    let copyright = report
        .assessment("FR", LegalCategory::CopyrightAndIp)
        .unwrap();

    assert_eq!(report.verdict, LegalVerdict::NotProvided);
    assert_eq!(copyright.status, AssessmentStatus::RiskSignal);
    assert_eq!(copyright.action, RequiredAction::RightsReview);
    assert_eq!(copyright.policy_ids, ["fr-copyright-review"]);
    assert_eq!(copyright.evidence_ids, ["reverse-search-7"]);
    assert_eq!(report.policies[0].version, "2026-08-02");
    assert_eq!(
        report.observations[0].provenance.source_uri,
        "https://provider.example/match/7"
    );

    remove_fixture(&path);
}

#[test]
fn detector_adapters_run_independently_and_preserve_failures() {
    struct StaticProvider {
        id: &'static str,
        categories: Vec<LegalCategory>,
        result: Result<Vec<DetectorObservation>, std::io::ErrorKind>,
    }

    impl LegalityEvidenceProvider for StaticProvider {
        fn id(&self) -> &str {
            self.id
        }

        fn categories(&self) -> &[LegalCategory] {
            &self.categories
        }

        fn inspect(&self, _path: &Path) -> std::io::Result<Vec<DetectorObservation>> {
            self.result
                .as_ref()
                .map(Clone::clone)
                .map_err(|kind| std::io::Error::from(*kind))
        }
    }

    let path = fixture_path("adapters.jpg", &[0xff, 0xd8, 0xff, 0xd9]);
    let scene = StaticProvider {
        id: "scene-provider",
        categories: vec![LegalCategory::NationalSecurity],
        result: Ok(vec![DetectorObservation::risk(
            "scene-adapter-1",
            LegalCategory::NationalSecurity,
            "untrusted-provider-name",
            0.75,
            "https://provider.example/scene-adapter-1",
        )]),
    };
    let symbols = StaticProvider {
        id: "symbol-provider",
        categories: vec![LegalCategory::ForbiddenContent],
        result: Err(std::io::ErrorKind::TimedOut),
    };
    let rights = StaticProvider {
        id: "rights-provider",
        categories: vec![LegalCategory::CopyrightAndIp],
        result: Ok(vec![DetectorObservation::no_match(
            "rights-adapter-1",
            LegalCategory::CopyrightAndIp,
            "untrusted-provider-name",
            0.84,
            "https://provider.example/rights-adapter-1",
        )]),
    };
    let providers: [&dyn LegalityEvidenceProvider; 3] = [&scene, &symbols, &rights];

    let report = check_file_legality_with_providers(
        &path,
        &FileLegalityConfig::for_jurisdictions(["CA"]),
        &providers,
    )
    .expect("run independent providers");

    assert_eq!(report.provider_runs[0].status, ProviderRunStatus::Completed);
    assert_eq!(report.provider_runs[1].status, ProviderRunStatus::Failed);
    assert_eq!(
        report.provider_runs[1].error_kind.as_deref(),
        Some("timed_out")
    );
    assert_eq!(report.provider_runs[2].status, ProviderRunStatus::Completed);
    assert_eq!(
        report
            .assessment("CA", LegalCategory::NationalSecurity)
            .unwrap()
            .status,
        AssessmentStatus::RiskSignal
    );
    assert_eq!(
        report
            .assessment("CA", LegalCategory::ForbiddenContent)
            .unwrap()
            .status,
        AssessmentStatus::Unknown
    );
    assert_eq!(
        report
            .assessment("CA", LegalCategory::CopyrightAndIp)
            .unwrap()
            .status,
        AssessmentStatus::NoRiskSignalDetected
    );
    assert_eq!(report.observations[0].provenance.provider, "scene-provider");
    assert_eq!(
        report.observations[1].provenance.provider,
        "rights-provider"
    );

    remove_fixture(&path);
}

#[test]
fn real_agent_cli_artifact_pins_the_provider_safety_leaf_byte_for_byte() {
    let artifact = include_str!(
        "../../docs/case-studies/issue-835/agent-cli-evidence/agent-authored-provider-safety-boundary.md"
    );
    assert_eq!(
        artifact,
        "Confirmed child-safety hash matches must come from an authorized provider receipt, \
         suppress local derivatives, stop ordinary detector execution, and escalate through \
         the provider's reporting channel."
    );
    let session =
        include_str!("../../docs/case-studies/issue-835/agent-cli-evidence/session-id.txt");
    assert_eq!(session, "ses_03d2a3c95ffe1gfVxnh24MtxFi\n");
}

/// Build a compact little-endian TIFF with IFD0 and Exif IFD fields. The bytes
/// are generated by this test and carry no third-party image content.
fn minimal_exif_tiff() -> Vec<u8> {
    const ASCII: u16 = 2;
    const LONG: u16 = 4;
    let mut bytes = vec![b'I', b'I', 42, 0, 8, 0, 0, 0];
    push_u16(&mut bytes, 6);
    let make_pointer = push_ifd_entry(&mut bytes, 0x010f, ASCII, 10, 0);
    let model_pointer = push_ifd_entry(&mut bytes, 0x0110, ASCII, 5, 0);
    let author_pointer = push_ifd_entry(&mut bytes, 0x013b, ASCII, 12, 0);
    let copyright_pointer = push_ifd_entry(&mut bytes, 0x8298, ASCII, 12, 0);
    let exif_pointer = push_ifd_entry(&mut bytes, 0x8769, LONG, 1, 0);
    let gps_pointer = push_ifd_entry(&mut bytes, 0x8825, LONG, 1, 0);
    push_u32(&mut bytes, 0);

    let make = append_data(&mut bytes, b"FormalCam\0");
    let model = append_data(&mut bytes, b"FC-1\0");
    let author = append_data(&mut bytes, b"Ada Example\0");
    let copyright = append_data(&mut bytes, b"CC0 Example\0");
    patch_u32(&mut bytes, make_pointer, make);
    patch_u32(&mut bytes, model_pointer, model);
    patch_u32(&mut bytes, author_pointer, author);
    patch_u32(&mut bytes, copyright_pointer, copyright);

    let exif_ifd = u32::try_from(bytes.len()).unwrap();
    patch_u32(&mut bytes, exif_pointer, exif_ifd);
    push_u16(&mut bytes, 1);
    let captured_pointer = push_ifd_entry(&mut bytes, 0x9003, ASCII, 20, 0);
    push_u32(&mut bytes, 0);
    let captured = append_data(&mut bytes, b"2026:08:02 12:34:56\0");
    patch_u32(&mut bytes, captured_pointer, captured);

    const ASCII_INLINE_NORTH: u32 = u32::from_le_bytes([b'N', 0, 0, 0]);
    const ASCII_INLINE_WEST: u32 = u32::from_le_bytes([b'W', 0, 0, 0]);
    const RATIONAL: u16 = 5;
    let gps_ifd = u32::try_from(bytes.len()).unwrap();
    patch_u32(&mut bytes, gps_pointer, gps_ifd);
    push_u16(&mut bytes, 4);
    push_ifd_entry(&mut bytes, 0x0001, ASCII, 2, ASCII_INLINE_NORTH);
    let latitude_pointer = push_ifd_entry(&mut bytes, 0x0002, RATIONAL, 3, 0);
    push_ifd_entry(&mut bytes, 0x0003, ASCII, 2, ASCII_INLINE_WEST);
    let longitude_pointer = push_ifd_entry(&mut bytes, 0x0004, RATIONAL, 3, 0);
    push_u32(&mut bytes, 0);
    let latitude = append_rationals(&mut bytes, &[(40, 1), (30, 1), (0, 1)]);
    let longitude = append_rationals(&mut bytes, &[(74, 1), (0, 1), (0, 1)]);
    patch_u32(&mut bytes, latitude_pointer, latitude);
    patch_u32(&mut bytes, longitude_pointer, longitude);
    bytes
}

fn push_ifd_entry(bytes: &mut Vec<u8>, tag: u16, field_type: u16, count: u32, value: u32) -> usize {
    push_u16(bytes, tag);
    push_u16(bytes, field_type);
    push_u32(bytes, count);
    let value_position = bytes.len();
    push_u32(bytes, value);
    value_position
}

fn append_data(bytes: &mut Vec<u8>, data: &[u8]) -> u32 {
    let offset = u32::try_from(bytes.len()).unwrap();
    bytes.extend_from_slice(data);
    offset
}

fn append_rationals(bytes: &mut Vec<u8>, values: &[(u32, u32)]) -> u32 {
    let offset = u32::try_from(bytes.len()).unwrap();
    for (numerator, denominator) in values {
        push_u32(bytes, *numerator);
        push_u32(bytes, *denominator);
    }
    offset
}

fn patch_u32(bytes: &mut [u8], position: usize, value: u32) {
    bytes[position..position + 4].copy_from_slice(&value.to_le_bytes());
}

fn push_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}
