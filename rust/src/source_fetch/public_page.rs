//! Restricted transport for URLs supplied by a search index.

use std::net::{IpAddr, ToSocketAddrs as _};

use url::{Host, Url};

use super::{FetchError, SourceTransport, fetch_with_curl};

pub(crate) fn search_page_url(raw: &str) -> Option<Url> {
    let mut url = Url::parse(raw).ok()?;
    let Host::Domain(domain) = url.host()? else {
        return None;
    };
    let domain = domain.trim_end_matches('.');
    if url.scheme() != "https"
        || url.port().is_some()
        || !url.username().is_empty()
        || url.password().is_some()
        || !domain.rsplit_once('.').is_some_and(|(_, suffix)| {
            suffix
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
        })
        || ["localhost", "local", "internal", "home.arpa"]
            .iter()
            .any(|suffix| {
                domain == *suffix
                    || domain
                        .strip_suffix(*suffix)
                        .is_some_and(|prefix| prefix.ends_with('.'))
            })
    {
        return None;
    }
    url.set_fragment(None);
    Some(url)
}

fn public_address(address: IpAddr) -> bool {
    match address {
        IpAddr::V4(address) => {
            let [first, second, third, _] = address.octets();
            !matches!(
                (first, second, third),
                (0 | 10 | 127 | 224..=255, _, _)
                    | (100, 64..=127, _)
                    | (169, 254, _)
                    | (172, 16..=31, _)
                    | (192, 0, 0 | 2)
                    | (192, 88, 99)
                    | (192, 168, _)
                    | (198, 18 | 19, _)
                    | (198, 51, 100)
                    | (203, 0, 113)
            )
        }
        IpAddr::V6(address) => {
            let [first, second, ..] = address.segments();
            (0x2000..=0x3fff).contains(&first)
                && !matches!(
                    (first, second),
                    (0x2001, 0..=0x1ff | 0xdb8) | (0x2002 | 0x3fff, _)
                )
        }
    }
}

pub(crate) struct PublicPageTransport;

impl SourceTransport for PublicPageTransport {
    fn get(&self, source: &str) -> Result<Vec<u8>, FetchError> {
        let url =
            search_page_url(source).ok_or_else(|| FetchError::InvalidUrl(source.to_owned()))?;
        let domain = url
            .host_str()
            .ok_or_else(|| FetchError::InvalidUrl(source.to_owned()))?;
        let addresses = (domain, 443)
            .to_socket_addrs()
            .map_err(|error| FetchError::Transport(error.to_string()))?;
        let targets = addresses
            .map(|address| address.ip())
            .filter(|address| public_address(*address))
            .take(8)
            .map(|address| match address {
                IpAddr::V4(_) => address.to_string(),
                IpAddr::V6(_) => format!("[{address}]"),
            })
            .collect::<Vec<_>>();
        if targets.is_empty() {
            return Err(FetchError::Transport(
                "source_has_no_public_address".to_owned(),
            ));
        }
        let pinned = format!("{domain}:443:{}", targets.join(","));
        fetch_with_curl(
            url.as_str(),
            &[
                "--disable",
                "--globoff",
                "--noproxy",
                "*",
                "--proto",
                "=https",
                "--resolve",
                &pinned,
            ],
            "4",
            true,
        )
    }
}
