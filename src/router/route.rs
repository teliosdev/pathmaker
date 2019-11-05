use lazy_static::lazy_static;
use phf::{phf_map, Map};
use regex::Regex;
use std::borrow::Cow;

lazy_static! {
    static ref SEGMENT_MATCH: Regex = Regex::new(r"^\{(?::(?P<kind>[a-zA-Z]\w*))?\}$").unwrap();
}

static MATCH_KINDS: Map<&'static str, &'static str> = phf_map! {
    "string" => r"([^/]+)",
    "int" => r"([-+]?\d+)",
    "uint" => r"(\d+)",
    "uuid" => r"([a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12})",
};

#[derive(Debug, Clone)]
pub struct Route<M, H> {
    pub(super) path: Cow<'static, str>,
    pub(super) method: M,
    pub(super) handler: H,
    pub(super) pattern: Regex,
}

impl<M, H> Route<M, H> {
    pub fn new<P>(path: P, method: M, handler: H) -> Route<M, H>
    where
        P: Into<Cow<'static, str>>,
    {
        let path = path.into();
        let compile = parse(path.as_ref());
        Route {
            path,
            method,
            handler,
            pattern: compile,
        }
    }
}

fn parse(path: &str) -> Regex {
    let normalized = crate::normalize_url(path);
    let split = normalized.split("/").skip(1);
    let mut pattern = split
        .map(|part| {
            if let Some(cap) = SEGMENT_MATCH.captures(part) {
                let name = cap.name("kind").map(|m| m.as_str()).unwrap_or("string");
                Cow::Borrowed(MATCH_KINDS.get(name).map(|v| *v).unwrap_or(r"([^/]*)"))
            } else {
                Cow::Owned(regex::escape(part))
            }
        })
        .fold(String::from("^"), |mut acc, el| {
            acc.push('/');
            acc.push_str(el.as_ref());
            acc
        });

    pattern.push('$');
    Regex::new(&pattern).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_parse() {
        fn assert_path(given: &str, expected: &str) {
            assert_eq!(parse(given).as_str(), expected)
        }
        assert_path("/some/path", r"^/some/path$");
        assert_path("/some/{:string}", r"^/some/([^/]+)$");
        assert_path("/some/{:int}", r"^/some/([-+]?\d+)$");
        assert_path("/some/{:uint}", r"^/some/(\d+)$");
        assert_path(
            "/some/{:uuid}",
            r"^/some/([a-fA-F0-9]{8}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{4}-[a-fA-F0-9]{12})$",
        );
    }
}
