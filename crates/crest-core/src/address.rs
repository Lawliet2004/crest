use serde::{Deserialize, Serialize};
use url::{Host, Url};

pub const HOME: &str = "crest://home/";
pub const BLANK: &str = "about:blank";

const MAX_ADDRESS_BYTES: usize = 32 * 1024;

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchEngine {
    #[default]
    Google,
    DuckDuckGo,
}

fn web_address(url: &Url) -> bool {
    matches!(url.scheme(), "https" | "http")
        && url.has_host()
        && url.username().is_empty()
        && url.password().is_none()
}

pub fn is_web_url(input: &str) -> bool {
    input.len() <= MAX_ADDRESS_BYTES && Url::parse(input).is_ok_and(|url| web_address(&url))
}

pub fn is_page_url(input: &str) -> bool {
    matches!(input, HOME | BLANK) || is_web_url(input)
}

pub fn resolve(input: &str, search_engine: SearchEngine) -> Result<String, &'static str> {
    let input = input.trim();
    if input.is_empty() {
        return Ok(HOME.into());
    }
    if input.len() > MAX_ADDRESS_BYTES || input.chars().any(char::is_control) {
        return Err("Enter an address or a shorter search query");
    }
    if matches!(input, HOME | BLANK) {
        return Ok(input.into());
    }

    let explicit = Url::parse(input);
    if let Ok(url) = &explicit
        && web_address(url)
    {
        return Ok(url.to_string());
    }
    if !input.contains("://")
        && !input.chars().any(char::is_whitespace)
        && let Ok(mut url) = Url::parse(&format!("https://{input}"))
        && web_address(&url)
        && let Some(host) = url.host()
    {
        let (hostname, local) = match host {
            Host::Domain(domain) => (
                domain.contains('.'),
                domain.eq_ignore_ascii_case("localhost"),
            ),
            Host::Ipv4(ip) => (true, ip.is_loopback()),
            Host::Ipv6(ip) => (true, ip.is_loopback()),
        };
        if local {
            let _ = url.set_scheme("http");
        }
        if hostname || local {
            return Ok(url.into());
        }
    }
    if explicit.is_ok() || input.contains("://") {
        return Err("Crest opens HTTP and HTTPS addresses");
    }

    let endpoint = match search_engine {
        SearchEngine::Google => "https://www.google.com/search",
        SearchEngine::DuckDuckGo => "https://duckduckgo.com/",
    };
    let search = Url::parse_with_params(endpoint, [("q", input)]).expect("valid search endpoint");
    Ok(search.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_web_hosts_and_local_development_addresses() {
        let engine = SearchEngine::Google;
        assert_eq!(
            resolve(" example.org/docs ", engine).unwrap(),
            "https://example.org/docs"
        );
        assert_eq!(
            resolve("localhost:8765", engine).unwrap(),
            "http://localhost:8765/"
        );
        assert_eq!(
            resolve("[::1]:4000/test", engine).unwrap(),
            "http://[::1]:4000/test"
        );
        assert_eq!(
            resolve("https://bücher.de", engine).unwrap(),
            "https://xn--bcher-kva.de/"
        );
        assert_eq!(resolve("", engine).unwrap(), HOME);
    }

    #[test]
    fn encodes_query_parameters_and_uses_selected_provider() {
        let query = "rust & q=browser + tabs";
        let result = resolve(query, SearchEngine::DuckDuckGo).unwrap();
        let url = Url::parse(&result).unwrap();
        assert_eq!(url.host_str(), Some("duckduckgo.com"));
        assert_eq!(
            url.query_pairs().collect::<Vec<_>>(),
            [("q".into(), query.into())]
        );
    }

    #[test]
    fn rejects_executable_and_credential_bearing_addresses() {
        for input in [
            "javascript:alert(1)",
            "file:///etc/passwd",
            "data:text/html,hello",
            "ftp://example.org/",
            "https://user:password@example.org/",
            "user:password@example.org",
        ] {
            assert!(resolve(input, SearchEngine::Google).is_err(), "{input}");
            assert!(!is_page_url(input), "{input}");
        }
    }
}
