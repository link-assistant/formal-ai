use super::OutputPolicy;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

static DEFAULT_OUTPUT: OnceLock<OutputConfig> = OnceLock::new();

pub fn configured_output() -> OutputConfig {
    *DEFAULT_OUTPUT.get_or_init(OutputConfig::default)
}

pub fn configure_output(configuration: OutputConfig) -> Result<(), &'static str> {
    DEFAULT_OUTPUT
        .set(configuration)
        .map_err(|_| "cnl_output_already_configured")
}

pub fn deserialize_standard<'de, D: serde::Deserializer<'de>>(
    input: D,
) -> Result<Option<String>, D::Error> {
    let value = Option::<String>::deserialize(input)?;
    output_configuration(value.as_deref(), None, super::cnl_policy())
        .map_err(serde::de::Error::custom)?;
    Ok(value)
}

pub fn deserialize_density<'de, D: serde::Deserializer<'de>>(
    input: D,
) -> Result<Option<String>, D::Error> {
    let value = Option::<String>::deserialize(input)?;
    output_configuration(None, value.as_deref(), super::cnl_policy())
        .map_err(serde::de::Error::custom)?;
    Ok(value)
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutputStandard {
    #[default]
    CnlDefault,
    AsdSte100,
    Ace,
    Gellish,
    Sbvr,
    PlainLanguage,
    MinimalEnglish,
    EPrime,
    BasicEnglish,
    SpecialEnglish,
    Globish,
}

impl OutputStandard {
    pub fn slug(self) -> &'static str {
        match self {
            Self::CnlDefault => "cnl-default",
            Self::AsdSte100 => "asd-ste100",
            Self::Ace => "ace",
            Self::Gellish => "gellish",
            Self::Sbvr => "sbvr",
            Self::PlainLanguage => "plain-language",
            Self::MinimalEnglish => "minimal-english",
            Self::EPrime => "e-prime",
            Self::BasicEnglish => "basic-english",
            Self::SpecialEnglish => "special-english",
            Self::Globish => "globish",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        [
            Self::CnlDefault,
            Self::AsdSte100,
            Self::Ace,
            Self::Gellish,
            Self::Sbvr,
            Self::PlainLanguage,
            Self::MinimalEnglish,
            Self::EPrime,
            Self::BasicEnglish,
            Self::SpecialEnglish,
            Self::Globish,
        ]
        .into_iter()
        .find(|item| item.slug() == value)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum OutputDensity {
    #[default]
    Standard,
    Primes,
    MaxPrecision,
    Top(u64),
}

impl OutputDensity {
    pub fn slug(self) -> String {
        match self {
            Self::Standard => "standard".to_owned(),
            Self::Primes => "primes".to_owned(),
            Self::MaxPrecision => "max-precision".to_owned(),
            Self::Top(count) => format!("top-{count}"),
        }
    }
    fn parse(value: &str) -> Option<Self> {
        match value {
            "standard" => Some(Self::Standard),
            "primes" => Some(Self::Primes),
            "dense" | "max-precision" => Some(Self::MaxPrecision),
            _ => frequency_tier(value).map(Self::Top),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct OutputConfig {
    pub standard: OutputStandard,
    pub density: OutputDensity,
}

/// The same safe-integer range as JavaScript, independent of the Rust target.
pub fn frequency_tier(value: &str) -> Option<u64> {
    let digits = value.strip_prefix("top-")?;
    if digits.starts_with('0')
        || digits.is_empty()
        || !digits.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let count: u64 = digits.parse().ok()?;
    if count > 9_007_199_254_740_991 {
        return None;
    }
    Some(count)
}

pub fn output_configuration(
    named: Option<&str>,
    density: Option<&str>,
    policy: &OutputPolicy,
) -> Result<OutputConfig, &'static str> {
    let named = named.unwrap_or("cnl-default").to_lowercase();
    let density_alias = named != "standard" && OutputDensity::parse(&named).is_some();
    let profile = if density_alias {
        policy.profiles.iter().find(|item| item.id == "cnl-default")
    } else {
        policy.profiles.iter().find(|item| {
            item.id == named
                || item
                    .aliases
                    .iter()
                    .any(|alias| alias.to_lowercase() == named)
        })
    }
    .ok_or("cnl_unknown_standard")?;
    let standard = OutputStandard::parse(&profile.id).ok_or("cnl_unknown_standard")?;
    let density = OutputDensity::parse(
        &density
            .unwrap_or(if density_alias { &named } else { "standard" })
            .to_lowercase(),
    )
    .ok_or("cnl_unknown_density")?;
    Ok(OutputConfig { standard, density })
}

/// CLI startup establishes the process defaults once; request overrides are local.
pub fn configure_options(standard: Option<&str>, density: Option<&str>) -> std::io::Result<()> {
    let configuration = output_configuration(standard, density, super::cnl_policy())
        .map_err(std::io::Error::other)?;
    configure_output(configuration).map_err(std::io::Error::other)
}
