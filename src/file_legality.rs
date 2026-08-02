//! Evidence-oriented, multi-jurisdiction file-legality assessments.
//!
//! This module composes metadata, provider observations, and versioned policy
//! packs. It deliberately reports category-level signals and required actions,
//! not a universal legal verdict.

use std::collections::BTreeSet;
use std::fs::File;
use std::io::{self, BufReader, Read};
use std::path::Path;

use exif::{Reader as ExifReader, Tag, Value};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalCategory {
    NationalSecurity,
    ForbiddenContent,
    CopyrightAndIp,
}

impl LegalCategory {
    pub const ALL: [Self; 3] = [
        Self::NationalSecurity,
        Self::ForbiddenContent,
        Self::CopyrightAndIp,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LegalVerdict {
    NotProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaFamily {
    Image,
    Document,
    Audio,
    Video,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssessmentStatus {
    Unknown,
    NoRiskSignalDetected,
    RiskSignal,
    ConfirmedProhibitedMatch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredAction {
    ObtainEvidence,
    NoAutomatedAction,
    LegalReview,
    RightsReview,
    RefuseAndEscalateToAuthorizedProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafetyDisposition {
    ContinueAssessment,
    RefuseAndEscalateToAuthorizedProvider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportLimitation {
    NoGlobalVerdict,
    NotLegalAdvice,
    ProviderEvidenceNotIndependentlyVerified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderRunStatus {
    Completed,
    Failed,
    SkippedFailClosed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataField {
    Author,
    Copyright,
    CameraMake,
    CameraModel,
    CapturedAt,
    GpsLatitude,
    GpsLongitude,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataProvenance {
    pub source: String,
    pub locator: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MetadataEntry {
    pub field: MetadataField,
    pub value: String,
    pub provenance: MetadataProvenance,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExtractedMetadata {
    pub entries: Vec<MetadataEntry>,
}

impl ExtractedMetadata {
    #[must_use]
    pub fn get(&self, field: MetadataField) -> Option<&MetadataEntry> {
        self.entries.iter().find(|entry| entry.field == field)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceProvenance {
    pub provider: String,
    pub source_uri: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DetectorObservation {
    pub id: String,
    pub category: LegalCategory,
    pub detected: bool,
    pub confidence: f64,
    pub provenance: EvidenceProvenance,
    #[serde(default)]
    pub restriction_codes: Vec<String>,
    #[serde(default)]
    pub jurisdictions: Vec<String>,
}

impl DetectorObservation {
    pub fn risk(
        id: impl Into<String>,
        category: LegalCategory,
        provider: impl Into<String>,
        confidence: f64,
        source_uri: impl Into<String>,
    ) -> Self {
        Self::new(id, category, provider, confidence, source_uri, true)
    }

    pub fn no_match(
        id: impl Into<String>,
        category: LegalCategory,
        provider: impl Into<String>,
        confidence: f64,
        source_uri: impl Into<String>,
    ) -> Self {
        Self::new(id, category, provider, confidence, source_uri, false)
    }

    fn new(
        id: impl Into<String>,
        category: LegalCategory,
        provider: impl Into<String>,
        confidence: f64,
        source_uri: impl Into<String>,
        detected: bool,
    ) -> Self {
        Self {
            id: id.into(),
            category,
            detected,
            confidence: confidence.clamp(0.0, 1.0),
            provenance: EvidenceProvenance {
                provider: provider.into(),
                source_uri: source_uri.into(),
            },
            restriction_codes: Vec::new(),
            jurisdictions: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_restriction_code(mut self, code: impl Into<String>) -> Self {
        self.restriction_codes.push(code.into());
        self
    }

    #[must_use]
    pub fn for_jurisdiction(mut self, jurisdiction: impl Into<String>) -> Self {
        self.jurisdictions
            .push(canonical_jurisdiction(&jurisdiction.into()));
        self
    }

    #[must_use]
    pub fn for_jurisdictions<I, S>(mut self, jurisdictions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.jurisdictions.extend(
            jurisdictions
                .into_iter()
                .map(Into::into)
                .map(|value| canonical_jurisdiction(&value)),
        );
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JurisdictionPolicy {
    pub id: String,
    pub version: String,
    pub jurisdiction: String,
    pub category: LegalCategory,
    pub action: RequiredAction,
    pub source_uri: String,
    #[serde(default)]
    pub trigger_codes: Vec<String>,
}

impl JurisdictionPolicy {
    pub fn new(
        id: impl Into<String>,
        version: impl Into<String>,
        jurisdiction: impl Into<String>,
        category: LegalCategory,
        action: RequiredAction,
        source_uri: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            version: version.into(),
            jurisdiction: canonical_jurisdiction(&jurisdiction.into()),
            category,
            action,
            source_uri: source_uri.into(),
            trigger_codes: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_trigger_code(mut self, code: impl Into<String>) -> Self {
        self.trigger_codes.push(code.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorizedHashMatch {
    pub provider: String,
    pub provider_reference: String,
    pub report_uri: String,
    pub confirmed: bool,
}

impl AuthorizedHashMatch {
    pub fn confirmed(
        provider: impl Into<String>,
        provider_reference: impl Into<String>,
        report_uri: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            provider_reference: provider_reference.into(),
            report_uri: report_uri.into(),
            confirmed: true,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct FileLegalityConfig {
    #[serde(default)]
    pub jurisdictions: Vec<String>,
    #[serde(default)]
    pub policies: Vec<JurisdictionPolicy>,
    #[serde(default)]
    pub observations: Vec<DetectorObservation>,
    #[serde(default)]
    pub authorized_hash_matches: Vec<AuthorizedHashMatch>,
}

/// Adapter boundary for object, symbol, rights, or other external detectors.
///
/// Implementations inspect the file in their own trust boundary and return
/// evidence observations only. One adapter failure is retained in the report
/// and does not prevent other category adapters from running.
pub trait LegalityEvidenceProvider {
    fn id(&self) -> &str;
    fn categories(&self) -> &[LegalCategory];
    fn inspect(&self, path: &Path) -> io::Result<Vec<DetectorObservation>>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderRun {
    pub provider: String,
    pub categories: Vec<LegalCategory>,
    pub status: ProviderRunStatus,
    pub observation_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_kind: Option<String>,
}

impl FileLegalityConfig {
    pub fn for_jurisdictions<I, S>(jurisdictions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            jurisdictions: jurisdictions
                .into_iter()
                .map(Into::into)
                .map(|value| canonical_jurisdiction(&value))
                .collect(),
            ..Self::default()
        }
    }

    pub fn add_policy(&mut self, policy: JurisdictionPolicy) {
        self.policies.push(policy);
    }

    pub fn add_observation(&mut self, observation: DetectorObservation) {
        self.observations.push(observation);
    }

    pub fn add_authorized_hash_match(&mut self, hash_match: AuthorizedHashMatch) {
        self.authorized_hash_matches.push(hash_match);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileIdentity {
    pub name: String,
    pub media_type: String,
    pub media_family: MediaFamily,
    pub size_bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CategoryAssessment {
    pub jurisdiction: String,
    pub category: LegalCategory,
    pub status: AssessmentStatus,
    pub action: RequiredAction,
    pub confidence: f64,
    pub evidence_ids: Vec<String>,
    pub policy_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FileLegalityReport {
    pub file: FileIdentity,
    pub metadata: ExtractedMetadata,
    pub assessments: Vec<CategoryAssessment>,
    pub observations: Vec<DetectorObservation>,
    pub policies: Vec<JurisdictionPolicy>,
    pub authorized_hash_matches: Vec<AuthorizedHashMatch>,
    pub provider_runs: Vec<ProviderRun>,
    pub verdict: LegalVerdict,
    pub safety_disposition: SafetyDisposition,
    pub limitations: Vec<ReportLimitation>,
}

impl FileLegalityReport {
    #[must_use]
    pub fn assessment(
        &self,
        jurisdiction: &str,
        category: LegalCategory,
    ) -> Option<&CategoryAssessment> {
        let jurisdiction = canonical_jurisdiction(jurisdiction);
        self.assessments.iter().find(|assessment| {
            assessment.jurisdiction == jurisdiction && assessment.category == category
        })
    }
}

pub fn check_file_legality(path: impl AsRef<Path>) -> io::Result<FileLegalityReport> {
    check_file_legality_with(path, &FileLegalityConfig::default())
}

pub fn check_file_legality_with_providers(
    path: impl AsRef<Path>,
    config: &FileLegalityConfig,
    providers: &[&dyn LegalityEvidenceProvider],
) -> io::Result<FileLegalityReport> {
    let path = path.as_ref();
    if config
        .authorized_hash_matches
        .iter()
        .any(|hash_match| hash_match.confirmed)
    {
        let mut report = check_file_legality_with(path, config)?;
        report.provider_runs = providers
            .iter()
            .map(|provider| ProviderRun {
                provider: provider.id().to_owned(),
                categories: provider.categories().to_vec(),
                status: ProviderRunStatus::SkippedFailClosed,
                observation_ids: Vec::new(),
                error_kind: None,
            })
            .collect();
        return Ok(report);
    }

    let mut composed = config.clone();
    let mut provider_runs = Vec::with_capacity(providers.len());
    for provider in providers {
        match provider.inspect(path) {
            Ok(observations) => {
                let allowed_categories = provider.categories();
                let observations: Vec<DetectorObservation> = observations
                    .into_iter()
                    .filter(|observation| allowed_categories.contains(&observation.category))
                    .map(|mut observation| {
                        provider
                            .id()
                            .clone_into(&mut observation.provenance.provider);
                        observation
                    })
                    .collect();
                provider_runs.push(ProviderRun {
                    provider: provider.id().to_owned(),
                    categories: allowed_categories.to_vec(),
                    status: ProviderRunStatus::Completed,
                    observation_ids: observations
                        .iter()
                        .map(|observation| observation.id.clone())
                        .collect(),
                    error_kind: None,
                });
                composed.observations.extend(observations);
            }
            Err(error) => provider_runs.push(ProviderRun {
                provider: provider.id().to_owned(),
                categories: provider.categories().to_vec(),
                status: ProviderRunStatus::Failed,
                observation_ids: Vec::new(),
                error_kind: Some(io_error_kind(error.kind()).to_owned()),
            }),
        }
    }
    let mut report = check_file_legality_with(path, &composed)?;
    report.provider_runs = provider_runs;
    Ok(report)
}

pub fn check_file_legality_with(
    path: impl AsRef<Path>,
    config: &FileLegalityConfig,
) -> io::Result<FileLegalityReport> {
    let path = path.as_ref();
    let confirmed_matches: Vec<&AuthorizedHashMatch> = config
        .authorized_hash_matches
        .iter()
        .filter(|hash_match| hash_match.confirmed)
        .collect();
    let fail_closed = !confirmed_matches.is_empty();
    let inspected = inspect_file(path, fail_closed)?;

    let jurisdictions = normalized_jurisdictions(&config.jurisdictions);
    let mut assessments = Vec::with_capacity(jurisdictions.len() * LegalCategory::ALL.len());
    for jurisdiction in &jurisdictions {
        for category in LegalCategory::ALL {
            assessments.push(assess_category(
                jurisdiction,
                category,
                config,
                &confirmed_matches,
            ));
        }
    }

    Ok(FileLegalityReport {
        file: FileIdentity {
            name: path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default(),
            media_type: inspected.media_type,
            media_family: inspected.media_family,
            size_bytes: inspected.size_bytes,
            sha256: inspected.sha256,
        },
        metadata: inspected.metadata,
        assessments,
        observations: config.observations.clone(),
        policies: config.policies.clone(),
        authorized_hash_matches: config.authorized_hash_matches.clone(),
        provider_runs: Vec::new(),
        verdict: LegalVerdict::NotProvided,
        safety_disposition: if fail_closed {
            SafetyDisposition::RefuseAndEscalateToAuthorizedProvider
        } else {
            SafetyDisposition::ContinueAssessment
        },
        limitations: vec![
            ReportLimitation::NoGlobalVerdict,
            ReportLimitation::NotLegalAdvice,
            ReportLimitation::ProviderEvidenceNotIndependentlyVerified,
        ],
    })
}

fn assess_category(
    jurisdiction: &str,
    category: LegalCategory,
    config: &FileLegalityConfig,
    confirmed_matches: &[&AuthorizedHashMatch],
) -> CategoryAssessment {
    if category == LegalCategory::ForbiddenContent && !confirmed_matches.is_empty() {
        return CategoryAssessment {
            jurisdiction: jurisdiction.to_owned(),
            category,
            status: AssessmentStatus::ConfirmedProhibitedMatch,
            action: RequiredAction::RefuseAndEscalateToAuthorizedProvider,
            confidence: 1.0,
            evidence_ids: confirmed_matches
                .iter()
                .map(|hash_match| hash_match.provider_reference.clone())
                .collect(),
            policy_ids: Vec::new(),
        };
    }

    let observations: Vec<&DetectorObservation> = config
        .observations
        .iter()
        .filter(|observation| {
            observation.category == category
                && (observation.jurisdictions.is_empty()
                    || observation
                        .jurisdictions
                        .iter()
                        .any(|value| value == "*" || value.eq_ignore_ascii_case(jurisdiction)))
        })
        .collect();
    if observations.is_empty() {
        return CategoryAssessment {
            jurisdiction: jurisdiction.to_owned(),
            category,
            status: AssessmentStatus::Unknown,
            action: RequiredAction::ObtainEvidence,
            confidence: 0.0,
            evidence_ids: Vec::new(),
            policy_ids: Vec::new(),
        };
    }

    let detected: Vec<&DetectorObservation> = observations
        .iter()
        .copied()
        .filter(|observation| observation.detected)
        .collect();
    let selected = if detected.is_empty() {
        &observations
    } else {
        &detected
    };
    let confidence = selected
        .iter()
        .map(|observation| observation.confidence)
        .fold(0.0_f64, f64::max);
    let evidence_ids = selected
        .iter()
        .map(|observation| observation.id.clone())
        .collect();

    if detected.is_empty() {
        return CategoryAssessment {
            jurisdiction: jurisdiction.to_owned(),
            category,
            status: AssessmentStatus::NoRiskSignalDetected,
            action: RequiredAction::NoAutomatedAction,
            confidence,
            evidence_ids,
            policy_ids: Vec::new(),
        };
    }

    let policies: Vec<&JurisdictionPolicy> = config
        .policies
        .iter()
        .filter(|policy| {
            policy.category == category
                && (policy.jurisdiction == "*"
                    || policy.jurisdiction.eq_ignore_ascii_case(jurisdiction))
                && detected
                    .iter()
                    .any(|observation| policy_applies(policy, observation))
        })
        .collect();
    let action = policies
        .iter()
        .map(|policy| policy.action)
        .max_by_key(|action| action_priority(*action))
        .unwrap_or_else(|| default_review_action(category));

    CategoryAssessment {
        jurisdiction: jurisdiction.to_owned(),
        category,
        status: AssessmentStatus::RiskSignal,
        action,
        confidence,
        evidence_ids,
        policy_ids: policies.iter().map(|policy| policy.id.clone()).collect(),
    }
}

fn policy_applies(policy: &JurisdictionPolicy, observation: &DetectorObservation) -> bool {
    policy.trigger_codes.is_empty()
        || policy
            .trigger_codes
            .iter()
            .any(|trigger| observation.restriction_codes.contains(trigger))
}

const fn default_review_action(category: LegalCategory) -> RequiredAction {
    match category {
        LegalCategory::CopyrightAndIp => RequiredAction::RightsReview,
        LegalCategory::NationalSecurity | LegalCategory::ForbiddenContent => {
            RequiredAction::LegalReview
        }
    }
}

const fn action_priority(action: RequiredAction) -> u8 {
    match action {
        RequiredAction::NoAutomatedAction => 0,
        RequiredAction::ObtainEvidence => 1,
        RequiredAction::RightsReview => 2,
        RequiredAction::LegalReview => 3,
        RequiredAction::RefuseAndEscalateToAuthorizedProvider => 4,
    }
}

fn normalized_jurisdictions(values: &[String]) -> Vec<String> {
    if values.is_empty() {
        return vec!["unspecified".to_owned()];
    }
    values
        .iter()
        .map(|value| canonical_jurisdiction(value))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn canonical_jurisdiction(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.eq_ignore_ascii_case("unspecified") {
        "unspecified".to_owned()
    } else if trimmed == "*" {
        "*".to_owned()
    } else {
        trimmed.to_ascii_uppercase()
    }
}

const fn io_error_kind(kind: io::ErrorKind) -> &'static str {
    match kind {
        io::ErrorKind::NotFound => "not_found",
        io::ErrorKind::PermissionDenied => "permission_denied",
        io::ErrorKind::ConnectionRefused => "connection_refused",
        io::ErrorKind::ConnectionReset => "connection_reset",
        io::ErrorKind::ConnectionAborted => "connection_aborted",
        io::ErrorKind::NotConnected => "not_connected",
        io::ErrorKind::AddrInUse => "address_in_use",
        io::ErrorKind::AddrNotAvailable => "address_not_available",
        io::ErrorKind::BrokenPipe => "broken_pipe",
        io::ErrorKind::AlreadyExists => "already_exists",
        io::ErrorKind::WouldBlock => "would_block",
        io::ErrorKind::InvalidInput => "invalid_input",
        io::ErrorKind::InvalidData => "invalid_data",
        io::ErrorKind::TimedOut => "timed_out",
        io::ErrorKind::WriteZero => "write_zero",
        io::ErrorKind::Interrupted => "interrupted",
        io::ErrorKind::Unsupported => "unsupported",
        io::ErrorKind::UnexpectedEof => "unexpected_eof",
        io::ErrorKind::OutOfMemory => "out_of_memory",
        _ => "other",
    }
}

struct InspectedFile {
    media_type: String,
    media_family: MediaFamily,
    size_bytes: u64,
    sha256: Option<String>,
    metadata: ExtractedMetadata,
}

fn inspect_file(path: &Path, suppress_derivatives: bool) -> io::Result<InspectedFile> {
    let mut file = File::open(path)?;
    let size_bytes = file.metadata()?.len();
    let mut signature = [0_u8; 64];
    let signature_len = file.read(&mut signature)?;
    let (media_family, media_type) = classify_file(path, &signature[..signature_len]);

    if suppress_derivatives {
        return Ok(InspectedFile {
            media_type: media_type.to_owned(),
            media_family,
            size_bytes,
            sha256: None,
            metadata: ExtractedMetadata::default(),
        });
    }

    let mut digest = Sha256::new();
    digest.update(&signature[..signature_len]);
    io::copy(&mut file, &mut DigestWriter(&mut digest))?;
    let sha256 = Some(format!("{:x}", digest.finalize()));
    let metadata = extract_exif(path);

    Ok(InspectedFile {
        media_type: media_type.to_owned(),
        media_family,
        size_bytes,
        sha256,
        metadata,
    })
}

struct DigestWriter<'a>(&'a mut Sha256);

impl io::Write for DigestWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn classify_file(path: &Path, signature: &[u8]) -> (MediaFamily, &'static str) {
    if signature.starts_with(&[0xff, 0xd8, 0xff]) {
        return (MediaFamily::Image, "image/jpeg");
    }
    if signature.starts_with(b"\x89PNG\r\n\x1a\n") {
        return (MediaFamily::Image, "image/png");
    }
    if signature.starts_with(b"GIF87a") || signature.starts_with(b"GIF89a") {
        return (MediaFamily::Image, "image/gif");
    }
    if signature.starts_with(b"II*\0") || signature.starts_with(b"MM\0*") {
        return (MediaFamily::Image, "image/tiff");
    }
    if signature.starts_with(b"%PDF-") {
        return (MediaFamily::Document, "application/pdf");
    }
    if signature.starts_with(b"ID3") || signature.starts_with(&[0xff, 0xfb]) {
        return (MediaFamily::Audio, "audio/mpeg");
    }
    if signature.get(4..8) == Some(b"ftyp") {
        return (MediaFamily::Video, "video/mp4");
    }

    match path
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" | "png" | "gif" | "tif" | "tiff" | "webp" | "heic" => {
            (MediaFamily::Image, "application/octet-stream")
        }
        "pdf" | "txt" | "md" | "doc" | "docx" | "odt" => {
            (MediaFamily::Document, "application/octet-stream")
        }
        "mp3" | "wav" | "flac" | "ogg" | "m4a" => (MediaFamily::Audio, "application/octet-stream"),
        "mp4" | "mov" | "mkv" | "webm" | "avi" => (MediaFamily::Video, "application/octet-stream"),
        _ => (MediaFamily::Other, "application/octet-stream"),
    }
}

fn extract_exif(path: &Path) -> ExtractedMetadata {
    let Ok(file) = File::open(path) else {
        return ExtractedMetadata::default();
    };
    let Ok(exif) = ExifReader::new().read_from_container(&mut BufReader::new(file)) else {
        return ExtractedMetadata::default();
    };
    let mut metadata = ExtractedMetadata::default();
    for (tag, field) in [
        (Tag::Artist, MetadataField::Author),
        (Tag::Copyright, MetadataField::Copyright),
        (Tag::Make, MetadataField::CameraMake),
        (Tag::Model, MetadataField::CameraModel),
        (Tag::DateTimeOriginal, MetadataField::CapturedAt),
    ] {
        if let Some(value) = exif_value(&exif, tag) {
            metadata.entries.push(MetadataEntry {
                field,
                value,
                provenance: MetadataProvenance {
                    source: "embedded_exif".to_owned(),
                    locator: format!("exif:{tag}"),
                },
            });
        }
    }
    if let Some(value) = gps_coordinate(&exif, Tag::GPSLatitude, Tag::GPSLatitudeRef) {
        metadata.entries.push(gps_entry(
            MetadataField::GpsLatitude,
            value,
            Tag::GPSLatitude,
        ));
    }
    if let Some(value) = gps_coordinate(&exif, Tag::GPSLongitude, Tag::GPSLongitudeRef) {
        metadata.entries.push(gps_entry(
            MetadataField::GpsLongitude,
            value,
            Tag::GPSLongitude,
        ));
    }
    metadata
}

fn exif_value(exif: &exif::Exif, tag: Tag) -> Option<String> {
    let field = exif.fields().find(|field| field.tag == tag)?;
    let value = match &field.value {
        Value::Ascii(parts) => String::from_utf8_lossy(parts.first()?)
            .trim_matches(char::from(0))
            .trim()
            .to_owned(),
        value => value.display_as(tag).to_string(),
    };
    (!value.is_empty()).then(|| value.chars().take(512).collect())
}

fn gps_coordinate(exif: &exif::Exif, coordinate_tag: Tag, reference_tag: Tag) -> Option<String> {
    let field = exif.fields().find(|field| field.tag == coordinate_tag)?;
    let Value::Rational(parts) = &field.value else {
        return None;
    };
    if parts.len() < 3 {
        return None;
    }
    let mut decimal = parts[0].to_f64() + parts[1].to_f64() / 60.0 + parts[2].to_f64() / 3600.0;
    let reference = exif_value(exif, reference_tag).unwrap_or_default();
    if reference.eq_ignore_ascii_case("S") || reference.eq_ignore_ascii_case("W") {
        decimal = -decimal;
    }
    Some(format!("{decimal:.6}"))
}

fn gps_entry(field: MetadataField, value: String, tag: Tag) -> MetadataEntry {
    MetadataEntry {
        field,
        value,
        provenance: MetadataProvenance {
            source: "embedded_exif".to_owned(),
            locator: format!("exif:{tag}"),
        },
    }
}
