#![allow(dead_code)]

use bunner_cors_rs::Headers;
use bunner_cors_rs::constants::header;
use std::collections::HashSet;

pub fn header_value<'a>(headers: &'a Headers, name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(name))
        .map(|(_, v)| v.as_str())
}

pub fn has_header(headers: &Headers, name: &str) -> bool {
    header_value(headers, name).is_some()
}

pub fn vary_values(headers: &Headers) -> HashSet<String> {
    header_value(headers, header::VARY)
        .map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default()
}
