pub(super) fn base_url_with_port(base_url: &str, port: Option<u16>) -> String {
    let trimmed = base_url.trim().trim_end_matches('/').to_string();
    let Some(port) = port else {
        return trimmed;
    };
    replace_url_port(&trimmed, port)
}

fn replace_url_port(url: &str, port: u16) -> String {
    let Some((scheme, rest)) = url.split_once("://") else {
        return format!("{url}:{port}");
    };
    let (authority, path) = rest.split_once('/').unwrap_or((rest, ""));
    let host = authority.strip_prefix('[').map_or_else(
        || unbracketed_authority_host(authority),
        |stripped| bracketed_authority_host(authority, stripped),
    );
    if path.is_empty() {
        format!("{scheme}://{host}:{port}")
    } else {
        format!("{scheme}://{host}:{port}/{path}")
    }
}

fn bracketed_authority_host(authority: &str, stripped: &str) -> String {
    stripped.split_once(']').map_or_else(
        || authority.to_string(),
        |(inside, _after)| format!("[{inside}]"),
    )
}

fn unbracketed_authority_host(authority: &str) -> String {
    authority
        .split_once(':')
        .map_or_else(|| authority.to_string(), |(host, _)| host.to_string())
}

pub(super) fn join_url_path(base_url: &str, endpoint_path: &str) -> String {
    let base = base_url.trim_end_matches('/');
    if base.ends_with(endpoint_path) {
        return base.to_string();
    }
    format!("{base}/{}", endpoint_path.trim_start_matches('/'))
}
