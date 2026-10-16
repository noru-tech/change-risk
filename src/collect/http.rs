//! A GET-only JSON client for forge APIs: no redirects, a size cap, bounded pages, and fixed error
//! messages so that response bodies and headers never reach the terminal.

use crate::{Exit, failure, failure_with_hint};
use anyhow::{Result, ensure};
use serde_json::Value;
use std::io::Read;
use std::time::Duration;

const MAX_BYTES: u64 = 16 * 1024 * 1024;
const USER_AGENT: &str = concat!("change-risk/", env!("CARGO_PKG_VERSION"));

pub enum Response {
    /// The value, and whether another page follows.
    Found(Value, bool),
    NotFound,
    Forbidden,
}

pub struct Http {
    agent: ureq::Agent,
    base: String,
    auth: Option<(&'static str, String)>,
    extra: Vec<(&'static str, &'static str)>,
    forge: &'static str,
    pub max_pages: usize,
}

/// `http://127.0.0.1:PORT` or `http://localhost:PORT`.
pub fn loopback(base: &str) -> bool {
    base.strip_prefix("http://127.0.0.1:")
        .or_else(|| base.strip_prefix("http://localhost:"))
        .is_some_and(|port| !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()))
}

impl Http {
    /// A client for `base`. A token is only ever sent to an `https` origin; loopback `http` is for
    /// tests and never carries one.
    pub fn new(
        forge: &'static str,
        base: &str,
        auth: Option<(&'static str, String)>,
        extra: Vec<(&'static str, &'static str)>,
        max_pages: usize,
    ) -> Result<Self> {
        if !(1..=1000).contains(&max_pages) {
            return Err(failure(Exit::Usage, "max-pages must be between 1 and 1000"));
        }
        let base = base.trim_end_matches('/');
        let https = base.starts_with("https://") && !base[8..].is_empty();
        if !(https || (loopback(base) && auth.is_none())) {
            return Err(failure_with_hint(
                Exit::Usage,
                format!("unsupported {forge} API origin"),
                "use an https:// API URL",
            ));
        }
        let agent = ureq::Agent::config_builder()
            .max_redirects(0)
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(30)))
            .user_agent(USER_AGENT)
            .build()
            .new_agent();
        Ok(Self {
            agent,
            base: base.into(),
            auth,
            extra,
            forge,
            max_pages,
        })
    }

    fn api(&self, message: &'static str) -> anyhow::Error {
        failure(Exit::Api, format!("{} {message}", self.forge))
    }

    /// GET `path`. With `tolerate`, a permission 403 is [`Response::Forbidden`] rather than an
    /// error, for data whose absence is recorded rather than fatal.
    pub fn fetch(&self, path: &str, tolerate: bool) -> Result<Response> {
        ensure!(
            path.starts_with('/') && !path.starts_with("//"),
            "invalid API path"
        );
        let mut request = self.agent.get(format!("{}{}", self.base, path));
        for (k, v) in &self.extra {
            request = request.header(*k, *v);
        }
        if let Some((name, value)) = &self.auth {
            request = request.header(*name, value);
        }
        let mut response = request.call().map_err(|_| self.api("request failed"))?;
        let status = response.status().as_u16();
        match status {
            200 => {}
            404 => return Ok(Response::NotFound),
            401 => {
                return Err(failure_with_hint(
                    Exit::Auth,
                    format!("{} authentication failed", self.forge),
                    "check the token: it is missing, expired, or not valid for this forge",
                ));
            }
            403 | 429 => {
                let h = response.headers();
                let limited = status == 429
                    || h.contains_key("retry-after")
                    || h.get("x-ratelimit-remaining")
                        .is_some_and(|v| v.as_bytes() == b"0")
                    || h.get("ratelimit-remaining")
                        .is_some_and(|v| v.as_bytes() == b"0");
                if tolerate && !limited {
                    return Ok(Response::Forbidden);
                }
                return Err(failure_with_hint(
                    Exit::Api,
                    format!("{} request forbidden or rate limited", self.forge),
                    if limited {
                        "wait for the rate limit to reset, or use a token with a higher limit"
                    } else {
                        "the token lacks a permission this read needs; see docs/forges.md"
                    },
                ));
            }
            _ => return Err(self.api("API returned an unexpected status")),
        }
        let h = response.headers();
        let next = h
            .get("link")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|s| s.contains("rel=\"next\""))
            || h.get("x-next-page")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|s| !s.trim().is_empty());
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_BYTES + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| self.api("response read failed"))?;
        if bytes.len() as u64 > MAX_BYTES {
            return Err(self.api("payload exceeds size limit"));
        }
        let value =
            serde_json::from_slice(&bytes).map_err(|_| self.api("returned invalid JSON"))?;
        Ok(Response::Found(value, next))
    }

    /// An object that must exist.
    pub fn object(&self, path: &str) -> Result<Value> {
        match self.fetch(path, false)? {
            Response::Found(value, _) => Ok(value),
            Response::NotFound | Response::Forbidden => Err(failure_with_hint(
                Exit::Api,
                format!("{} resource not found", self.forge),
                "check the repository and number, and that the token can read them",
            )),
        }
    }

    /// Walk a list endpoint. Returns the items and whether the listing is complete: with
    /// `tolerate`, an unreadable listing is incomplete rather than an error, and a listing longer
    /// than `max_pages` is always incomplete, never silently short.
    pub fn pages(&self, path: &str, tolerate: bool) -> Result<(Vec<Value>, bool)> {
        let mut all = Vec::new();
        for page in 1..=self.max_pages {
            let sep = if path.contains('?') { '&' } else { '?' };
            let (value, next) =
                match self.fetch(&format!("{path}{sep}per_page=100&page={page}"), tolerate)? {
                    Response::Found(value, next) => (value, next),
                    Response::NotFound | Response::Forbidden if tolerate => {
                        return Ok((all, false));
                    }
                    Response::NotFound | Response::Forbidden => {
                        return Err(failure_with_hint(
                            Exit::Api,
                            format!("{} resource not found", self.forge),
                            "check the repository and number, and that the token can read them",
                        ));
                    }
                };
            let items = value
                .as_array()
                .ok_or_else(|| self.api("list response is not an array"))?;
            all.extend(items.iter().cloned());
            if !next {
                return Ok((all, true));
            }
        }
        Ok((all, false))
    }
}

/// Percent-encode one path segment (RFC 3986 unreserved characters pass through).
pub fn segment(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        if b.is_ascii_alphanumeric() || b"-._~".contains(&b) {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_only_go_to_https() {
        assert!(
            Http::new(
                "GitHub",
                "https://api.github.com",
                Some(("A", "t".into())),
                vec![],
                10
            )
            .is_ok()
        );
        assert!(Http::new("GitHub", "http://api.github.com", None, vec![], 10).is_err());
        assert!(Http::new("GitHub", "http://127.0.0.1:8080", None, vec![], 10).is_ok());
        assert!(
            Http::new(
                "GitHub",
                "http://127.0.0.1:8080",
                Some(("A", "t".into())),
                vec![],
                10
            )
            .is_err()
        );
        assert!(Http::new("GitHub", "https://api.github.com", None, vec![], 0).is_err());
    }

    #[test]
    fn segments_are_encoded() {
        assert_eq!(segment("group/sub project"), "group%2Fsub%20project");
        assert_eq!(segment("a-b_c.d~e"), "a-b_c.d~e");
    }
}
