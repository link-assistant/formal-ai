//! Shared release lookups: uncertainty must never authorize publication.
//!
//! ```cargo
//! [dependencies]
//! ureq = "3.5.2"
//! semver = "1"
//! serde_json = "1.0.154"
//! ```
#![allow(dead_code)] // Each standalone script uses a different subset of this module.

use std::time::Duration;

// Keep rust-script from wrapping this shared module as an expression.
#[cfg(test)]
fn main() {}

#[derive(Debug)]
pub enum Failure {
    Transient(String),
    Permanent(String),
    InvalidResponse(String),
}

impl Failure {
    fn message(&self) -> &str {
        match self {
            Self::Transient(message)
            | Self::Permanent(message)
            | Self::InvalidResponse(message) => message,
        }
    }
}

pub fn with_retries<T>(
    attempts: u32,
    delay: Duration,
    mut operation: impl FnMut() -> Result<T, Failure>,
) -> Result<T, String> {
    if !(1..=10).contains(&attempts) {
        return Err("Registry attempts must be between 1 and 10".into());
    }
    for attempt in 1..=attempts {
        match operation() {
            Ok(value) => return Ok(value),
            Err(Failure::Permanent(error)) => return Err(error),
            Err(Failure::Transient(error) | Failure::InvalidResponse(error))
                if attempt == attempts =>
            {
                return Err(format!("{error} (after {attempts} attempts)"));
            }
            Err(Failure::Transient(error) | Failure::InvalidResponse(error)) => {
                eprintln!("Registry lookup retry {}/{attempts}: {error}", attempt + 1);
                std::thread::sleep(delay * 2_u32.pow(attempt - 1));
            }
        }
    }
    unreachable!()
}

fn registry_agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(10)))
        .timeout_per_read(Some(Duration::from_secs(20)))
        .timeout_per_write(Some(Duration::from_secs(20)))
        .timeout_global(Some(Duration::from_secs(30)))
        .http_status_as_error(false)
        .build()
        .into()
}

fn fetch_body(
    agent: &ureq::Agent,
    url: &str,
    headers: &[(&str, &str)],
) -> Result<Option<String>, Failure> {
    let mut request = agent
        .get(url)
        .header("User-Agent", "rust-template-release-check/1.0");
    for (key, value) in headers {
        request = request.header(*key, *value);
    }
    let mut response = request
        .call()
        .map_err(|error| Failure::Transient(format!("{url}: {error}")))?;
    match response.status().as_u16() {
        200 => {
            let unreadable = |error| {
                Failure::InvalidResponse(format!("Unreadable response from {url}: {error}"))
            };
            let bytes = response
                .body_mut()
                .read_to_vec()
                .map_err(|error| unreadable(error.to_string()))?;
            String::from_utf8(bytes)
                .map(Some)
                .map_err(|error| unreadable(error.to_string()))
        }
        404 => Ok(None),
        status if status == 429 || status >= 500 => {
            Err(Failure::Transient(format!("{url}: HTTP {status}")))
        }
        status => Err(Failure::Permanent(format!(
            "{url}: unexpected HTTP {status}"
        ))),
    }
}

fn get_body_with_policy(
    url: &str,
    headers: &[(&str, &str)],
    attempts: u32,
    delay: Duration,
) -> Result<Option<String>, String> {
    let agent = registry_agent();
    with_retries(attempts, delay, || fetch_body(&agent, url, headers))
}

pub fn get_body(url: &str, headers: &[(&str, &str)]) -> Result<Option<String>, String> {
    get_body_with_policy(url, headers, 4, Duration::from_secs(1))
}

pub fn or_exit<T>(lookup: &str, result: Result<T, String>) -> T {
    result.unwrap_or_else(|error| {
        eprintln!("::error::{lookup} failed: {error}. Re-run after the service recovers.");
        std::process::exit(1);
    })
}

pub fn parse_semver(version: &str) -> Option<(u32, u32, u32)> {
    let parts: Vec<_> = version.split(['-', '+']).next()?.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    Some((
        parts[0].parse().ok()?,
        parts[1].parse().ok()?,
        parts[2].parse().ok()?,
    ))
}

pub fn parse_max_version_name(body: &str) -> Result<Option<String>, String> {
    let data: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("Invalid crates.io JSON: {e}"))?;
    let versions = data
        .get("versions")
        .and_then(serde_json::Value::as_array)
        .ok_or("Missing crates.io versions array")?;
    let mut max: Option<((u32, u32, u32), &str)> = None;
    for version in versions {
        let number = version
            .get("num")
            .and_then(serde_json::Value::as_str)
            .ok_or("Missing version number")?;
        let yanked = version
            .get("yanked")
            .and_then(serde_json::Value::as_bool)
            .ok_or("Missing yanked status")?;
        if !yanked {
            let parsed = parse_semver(number)
                .ok_or_else(|| format!("Unsupported published version: {number}"))?;
            if max.is_none_or(|(current, _)| parsed > current) {
                max = Some((parsed, number));
            }
        }
    }
    Ok(max.map(|(_, number)| number.to_string()))
}

pub fn parse_max_version(body: &str) -> Result<Option<(u32, u32, u32)>, String> {
    Ok(parse_max_version_name(body)?.and_then(|number| parse_semver(&number)))
}

pub fn max_published_name(crate_name: &str) -> Result<Option<String>, String> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}");
    get_body(&url, &[])?
        .map(|body| parse_max_version_name(&body))
        .transpose()
        .map(Option::flatten)
}

pub fn max_published(crate_name: &str) -> Result<Option<(u32, u32, u32)>, String> {
    Ok(max_published_name(crate_name)?.and_then(|number| parse_semver(&number)))
}

pub fn version_exists(crate_name: &str, version: &str) -> Result<bool, String> {
    let url = format!("https://crates.io/api/v1/crates/{crate_name}/{version}");
    let Some(body) = get_body(&url, &[])? else {
        return Ok(false);
    };
    validate_version_response(&body, crate_name, version).map(|()| true)
}

fn validate_version_response(body: &str, crate_name: &str, version: &str) -> Result<(), String> {
    let data: serde_json::Value =
        serde_json::from_str(body).map_err(|e| format!("Invalid crates.io JSON: {e}"))?;
    let record = &data["version"];
    let crate_matches = record.get("crate").is_none_or(|name| {
        name.as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(crate_name))
    });
    if record["num"].as_str() != Some(version) || !crate_matches {
        return Err(format!(
            "Invalid crates.io version response for {crate_name}@{version}"
        ));
    }
    Ok(())
}

/// The result of consulting both Cargo's sparse index and the version API.
#[derive(Debug, PartialEq, Eq)]
pub enum VersionVisibility {
    Published,
    NotPublishedYet,
    Unknown(String),
}

fn sparse_index_path(crate_name: &str) -> Result<String, String> {
    let name = crate_name.to_ascii_lowercase();
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("Invalid sparse registry crate name".into());
    }
    Ok(match name.len() {
        1 => format!("1/{name}"),
        2 => format!("2/{name}"),
        3 => format!("3/{}/{name}", &name[..1]),
        _ => format!("{}/{}/{name}", &name[..2], &name[2..4]),
    })
}

fn sparse_body_has_version(body: &str, crate_name: &str, version: &str) -> Result<bool, String> {
    let mut present = false;
    let mut records = 0;
    for line in body.lines() {
        let record: serde_json::Value = serde_json::from_str(line)
            .map_err(|error| format!("Invalid sparse index JSON: {error}"))?;
        let name = record.get("name").and_then(serde_json::Value::as_str);
        if !name.is_some_and(|name| name.eq_ignore_ascii_case(crate_name)) {
            return Err("Invalid sparse index record identity".into());
        }
        let found = record["vers"]
            .as_str()
            .ok_or("Missing sparse index record version")?;
        semver::Version::parse(found)
            .map_err(|error| format!("Invalid sparse index version: {error}"))?;
        present |= found == version;
        records += 1;
    }
    if records == 0 {
        return Err("Empty sparse index response".into());
    }
    Ok(present)
}

/// Each outer waiter attempt makes at most one bounded request per endpoint.
/// A failed observation never becomes a claim that publication failed.
pub fn crate_version_visibility(crate_name: &str, version: &str) -> VersionVisibility {
    let path = match sparse_index_path(crate_name) {
        Ok(path) => path,
        Err(error) => return VersionVisibility::Unknown(error),
    };
    let agent = registry_agent();
    let observe = |url: &str, validate: fn(&str, &str, &str) -> Result<bool, String>| {
        let Some(body) = fetch_body(&agent, url, &[])? else {
            return Ok(false);
        };
        validate(&body, crate_name, version)
            .map_err(|error| Failure::InvalidResponse(format!("{url}: {error}")))
    };
    let index = observe(
        &format!("https://index.crates.io/{path}"),
        sparse_body_has_version,
    );
    if matches!(index, Ok(true)) {
        return VersionVisibility::Published;
    }
    let api = observe(
        &format!("https://crates.io/api/v1/crates/{crate_name}/{version}"),
        |body, name, version| validate_version_response(body, name, version).map(|()| true),
    );
    match (index, api) {
        (_, Ok(true)) => VersionVisibility::Published,
        (Err(index), Err(api)) => {
            VersionVisibility::Unknown(format!("{}; {}", index.message(), api.message()))
        }
        (Err(Failure::InvalidResponse(reason)), _) | (_, Err(Failure::InvalidResponse(reason))) => {
            VersionVisibility::Unknown(reason)
        }
        (Ok(false), _) | (_, Ok(false)) => VersionVisibility::NotPublishedYet,
        (Ok(true), _) => unreachable!(),
    }
}

pub fn artifact_exists(
    url: &str,
    headers: &[(&str, &str)],
    field: &str,
    expected: &str,
) -> Result<bool, String> {
    let Some(body) = get_body(url, headers)? else {
        return Ok(false);
    };
    let data: serde_json::Value = serde_json::from_str(&body)
        .map_err(|e| format!("Invalid artifact JSON from {url}: {e}"))?;
    if data.get(field).and_then(serde_json::Value::as_str) != Some(expected) {
        return Err(format!(
            "Invalid artifact response from {url}: expected {field}={expected}"
        ));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    #[test]
    fn version_response_requires_matching_version_and_crate_identity() {
        for (record, valid) in [
            (r#"{"num":"1.2.3","crate":"Fixture"}"#, true),
            (r#"{"num":"1.2.3"}"#, true),
            (r#"{"num":"1.2.3","crate":"other"}"#, false),
            (r#"{"num":"1.2.3","crate":null}"#, false),
            (r#"{"num":"1.2.30","crate":"fixture"}"#, false),
        ] {
            let body = format!(r#"{{"version":{record}}}"#);
            assert_eq!(
                validate_version_response(&body, "fixture", "1.2.3").is_ok(),
                valid
            );
        }
    }

    #[test]
    fn retries_transient_errors_but_not_permanent_errors() {
        let mut calls = 0;
        assert_eq!(
            with_retries(4, Duration::ZERO, || {
                calls += 1;
                if calls < 3 {
                    Err(Failure::Transient("reset".into()))
                } else {
                    Ok(42)
                }
            })
            .unwrap(),
            42
        );
        assert_eq!(calls, 3);
        let error = with_retries::<()>(4, Duration::ZERO, || {
            Err(Failure::Transient("HTTP 503".into()))
        })
        .unwrap_err();
        assert!(error.contains("after 4 attempts"));
        calls = 0;
        assert!(
            with_retries::<()>(4, Duration::ZERO, || {
                calls += 1;
                Err(Failure::Permanent("HTTP 403".into()))
            })
            .is_err()
        );
        assert_eq!(calls, 1);
    }

    #[test]
    fn http_errors_and_dropped_connections_are_not_absence() {
        for statuses in [
            vec![429, 503, 0, 200],
            vec![404],
            vec![403],
            vec![503, 503, 503, 503],
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            listener.set_nonblocking(true).unwrap();
            let url = format!("http://{}", listener.local_addr().unwrap());
            let expected = statuses.last().copied().unwrap();
            let server = std::thread::spawn(move || {
                let deadline = std::time::Instant::now() + Duration::from_secs(10);
                for status in statuses {
                    let mut stream = loop {
                        if let Ok((stream, _)) = listener.accept() {
                            break stream;
                        }
                        assert!(
                            std::time::Instant::now() < deadline,
                            "Registry fixture deadline"
                        );
                        std::thread::sleep(Duration::from_millis(1));
                    };
                    stream
                        .set_read_timeout(Some(Duration::from_secs(1)))
                        .unwrap();
                    let _ = stream.read(&mut [0_u8; 4096]);
                    if status != 0 {
                        write!(stream, "HTTP/1.1 {status} Fixture\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok").unwrap();
                    }
                }
            });
            let result = get_body_with_policy(&url, &[], 4, Duration::ZERO);
            server.join().unwrap();
            match expected {
                200 => assert_eq!(result.unwrap(), Some("ok".into())),
                404 => assert_eq!(result.unwrap(), None),
                _ => assert!(result.is_err()),
            }
        }
    }

    #[test]
    fn sparse_records_require_complete_identity_and_exact_version() {
        let valid = "{\"name\":\"fixture\",\"vers\":\"1.2.3\"}";
        assert!(sparse_body_has_version(valid, "fixture", "1.2.3").unwrap());
        assert!(!sparse_body_has_version(valid, "fixture", "1.2").unwrap());
        for body in [
            "",
            "{}",
            "{\"name\":\"other\",\"vers\":\"1.2.3\"}",
            "{\"name\":\"fixture\",\"note\":\"\\\"vers\\\":\\\"1.2.3\\\"\"}",
            &format!("{valid}\nnot json"),
            &format!("{valid}\n{{\"name\":\"fixture\",\"vers\":\"not-a-version\"}}"),
            &format!("{valid}\n\n{valid}"),
        ] {
            assert!(sparse_body_has_version(body, "fixture", "1.2.3").is_err());
        }
        for (name, expected) in [
            ("a", "1/a"),
            ("ab", "2/ab"),
            ("abc", "3/a/abc"),
            ("Serde", "se/rd/serde"),
        ] {
            assert_eq!(sparse_index_path(name).unwrap(), expected);
        }
        for invalid in ["", "../serde", "s\u{e9}rde"] {
            assert!(sparse_index_path(invalid).is_err());
        }
    }

    #[test]
    fn maximum_version_requires_a_complete_readable_registry_answer() {
        assert_eq!(parse_max_version(r#"{"versions":[{"num":"1.9.0","yanked":false},{"num":"2.0.0","yanked":true},{"num":"1.10.1","yanked":false}]}"#).unwrap(), Some((1,10,1)));
        assert_eq!(parse_max_version(r#"{"versions":[]}"#).unwrap(), None);
        assert_eq!(
            parse_max_version_name(r#"{"versions":[{"num":"2.0.0-rc.1+build","yanked":false}]}"#)
                .unwrap(),
            Some("2.0.0-rc.1+build".into())
        );
        for body in ["not json", "{}", r#"{"versions":[{"num":"1.2.3"}]}"#] {
            assert!(parse_max_version(body).is_err());
        }
    }
}
