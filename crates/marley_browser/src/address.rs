//! What the address bar's text navigates to, and which URLs are local (#503).
//!
//! The rules stay small enough to predict: a URL with a scheme Chromium navigates to loads as
//! typed; a host, with its port and path, loads over `http` when it is loopback and `https`
//! otherwise; anything else is a search at `duckduckgo.com`.
//!
//! A local URL is an http or https one whose host is this machine: `localhost` or a name under
//! it, a `127.0.0.0/8` address, `::1`, or the unspecified `0.0.0.0` and `::` a server prints for
//! where it listens. [`local_url`] reads one, and [`printed_local_urls`] finds them in a line a
//! terminal printed.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::path::{Path, PathBuf};

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

/// The local HTML page a program asked its opener to open (#586): a `file:` URL, or a path,
/// absolute or joined to `directory`, whose name ends in `.html` or `.htm`.
///
/// `cargo doc --open` hands the opener a plain path, Python's `webbrowser` a `file:` URL. Whether
/// the file is there is the caller's to check.
#[must_use]
pub fn local_page(text: &str, directory: &Path) -> Option<PathBuf> {
    let text = text.trim();
    let path = match url::Url::parse(text) {
        Ok(url) if url.scheme() == "file" => url.to_file_path().ok()?,
        Ok(_) => return None,
        Err(_) => directory.join(text),
    };
    let html = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
        });
    html.then_some(path)
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

/// A local URL a terminal printed or a user clicked (#503).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUrl {
    /// The URL to open, as parsed: an unspecified host becomes the loopback address of its
    /// family, since `0.0.0.0` says where a server listens and is no address to connect to.
    pub url: String,
    /// `host:port`, as the terminal's footer names it.
    pub label: String,
    /// The port, the scheme's own when the URL names none.
    pub port: u16,
}

/// `text` as a [`LocalUrl`], when it is an http or https URL on this machine.
#[must_use]
pub fn local_url(text: &str) -> Option<LocalUrl> {
    let mut url = url::Url::parse(text.trim()).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    let port = url.port_or_known_default()?;
    let connect_to: Option<IpAddr> = match url.host()? {
        url::Host::Domain(name) => {
            let name = name.to_ascii_lowercase();
            if name != "localhost" && !name.ends_with(".localhost") {
                return None;
            }
            None
        }
        url::Host::Ipv4(address) if address.is_unspecified() => Some(Ipv4Addr::LOCALHOST.into()),
        url::Host::Ipv6(address) if address.is_unspecified() => Some(Ipv6Addr::LOCALHOST.into()),
        url::Host::Ipv4(address) if address.is_loopback() => None,
        url::Host::Ipv6(address) if address.is_loopback() => None,
        url::Host::Ipv4(_) | url::Host::Ipv6(_) => return None,
    };
    if let Some(address) = connect_to {
        url.set_ip_host(address).ok()?;
    }
    let label = format!("{}:{port}", url.host_str()?);
    Some(LocalUrl {
        url: url.into(),
        label,
        port,
    })
}

/// The local URLs in `line`, in the order they appear. A URL ends at a space, a quote or an angle
/// bracket, and loses the punctuation a sentence puts after it and a closing bracket it did not
/// open.
#[must_use]
pub fn printed_local_urls(line: &str) -> Vec<LocalUrl> {
    let mut found = Vec::new();
    let mut rest = line;
    while let Some(start) = ["http://", "https://"]
        .iter()
        .filter_map(|scheme| rest.find(scheme))
        .min()
    {
        let candidate = rest.get(start..).unwrap_or_default();
        let end = candidate
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '<' | '>' | '"' | '\'' | '`')
            })
            .unwrap_or(candidate.len());
        let printed = candidate.get(..end).unwrap_or_default();
        found.extend(local_url(trim_printed(printed)));
        // Past the scheme at least, so a candidate that ends at once is not found again.
        rest = candidate.get(end.max(1)..).unwrap_or_default();
    }
    found
}

/// `url` without the sentence punctuation after it and the closing brackets it did not open.
fn trim_printed(mut url: &str) -> &str {
    loop {
        let unopened = |close: char, open: char| {
            url.ends_with(close) && url.matches(close).count() > url.matches(open).count()
        };
        if url.ends_with(['.', ',', ';', ':', '!', '?'])
            || unopened(')', '(')
            || unopened(']', '[')
            || unopened('}', '{')
        {
            url = url.get(..url.len() - 1).unwrap_or_default();
        } else {
            return url;
        }
    }
}
