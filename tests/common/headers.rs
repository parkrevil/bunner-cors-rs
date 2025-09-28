use bunner_cors_rs::constants::header;
use bunner_cors_rs::Header;
use std::collections::BTreeSet;

pub fn header_value<'a>(headers: &'a [Header], name: &str) -> Option<&'a str> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.as_str())
}

pub fn has_header(headers: &[Header], name: &str) -> bool {
    header_value(headers, name).is_some()
}

pub fn vary_values(headers: &[Header]) -> BTreeSet<String> {
    header_value(headers, header::VARY)
        .map(|value| {
            value
                .split(',')
                .map(|part| part.trim().to_string())
                .filter(|part| !part.is_empty())
                .collect::<BTreeSet<_>>()
        })
        .unwrap_or_default()
}
