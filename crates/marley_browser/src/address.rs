//! What the address bar's text navigates to.
//!
//! The rules stay small enough to predict: a URL with a scheme Chromium navigates to loads as
//! typed; a host, with its port and path, loads over `http` when it is loopback and `https`
//! otherwise; anything else is a search at `duckduckgo.com`.

use std::net::{Ipv4Addr, Ipv6Addr};

/// The schemes typed text keeps as they are.
const SCHEMES: &[&str] = &[
    "http",
    "https",
    "file",
    "about",
    "data",
    "chrome",
    "view-source",
];

/// Where a search goes: `duckduckgo.com` needs no account and shows no consent page.
const SEARCH: &str = "https://duckduckgo.com/?q=";

/// The URL an agent may send the Browser tab to (#492): `http` and `https` only, since an
/// agent's `file:` URL reads the user's files and a `javascript:` one runs script in the page.
///
/// # Errors
///
/// For text that is no URL, or a URL of another scheme.
pub fn agent_url(text: &str) -> Result<String, String> {
    let url =
        url::Url::parse(text.trim()).map_err(|error| format!("{text:?} is no URL: {error}"))?;
    match url.scheme() {
        "http" | "https" => Ok(url.into()),
        scheme => Err(format!(
            "an agent may open http and https URLs only, not {scheme}:"
        )),
    }
}

/// The URL `text` navigates to, or nothing for text that is only spaces.
#[must_use]
pub fn url_for(text: &str) -> Option<String> {
    let text = text.trim();
    if text.is_empty() {
        return None;
    }
    if let Some((scheme, _)) = text.split_once(':')
        && SCHEMES
            .iter()
            .any(|known| known.eq_ignore_ascii_case(scheme))
    {
        return Some(text.to_string());
    }
    if let Some(loopback) = host_kind(text) {
        let scheme = if loopback { "http" } else { "https" };
        return Some(format!("{scheme}://{text}"));
    }
    let query: String = url::form_urlencoded::byte_serialize(text.as_bytes()).collect();
    Some(format!("{SEARCH}{query}"))
}

/// Whether `text` is a host with an optional port and path, and if so whether the host is
/// loopback: `localhost` or a name under it, an IPv4 address, a bracketed IPv6 address, or a
/// name with a dot whose last label is not a number.
fn host_kind(text: &str) -> Option<bool> {
    if text.chars().any(char::is_whitespace) {
        return None;
    }
    let authority = text
        .split(['/', '?', '#'])
        .next()
        .filter(|authority| !authority.is_empty())?;
    if let Some(rest) = authority.strip_prefix('[') {
        let (address, after) = rest.split_once(']')?;
        let address = address.parse::<Ipv6Addr>().ok()?;
        let port_fits = after.is_empty() || after.strip_prefix(':').is_some_and(is_port);
        return port_fits.then_some(address.is_loopback());
    }
    let host = match authority.rsplit_once(':') {
        Some((host, port)) if is_port(port) => host,
        Some(_) => return None,
        None => authority,
    };
    if let Ok(address) = host.parse::<Ipv4Addr>() {
        return Some(address.is_loopback());
    }
    let name = host.to_lowercase();
    if name == "localhost" || name.ends_with(".localhost") {
        return Some(true);
    }
    let labels: Vec<&str> = name.split('.').collect();
    let is_label = |label: &&str| {
        !label.is_empty()
            && !label.starts_with('-')
            && !label.ends_with('-')
            && label
                .chars()
                .all(|character| character.is_alphanumeric() || character == '-')
    };
    let numeric_last = labels
        .last()
        .is_some_and(|last| last.chars().all(|character| character.is_ascii_digit()));
    (labels.len() > 1 && labels.iter().all(is_label) && !numeric_last).then_some(false)
}

/// Whether `port` is a port number: one to five digits.
fn is_port(port: &str) -> bool {
    (1..=5).contains(&port.len()) && port.chars().all(|character| character.is_ascii_digit())
}
