pub(crate) fn normalize_lower(value: &str) -> String {
    if value.is_ascii() {
        value.to_ascii_lowercase()
    } else {
        value.to_lowercase()
    }
}

pub(crate) fn equals_ignore_case(a: &str, b: &str) -> bool {
    if a.is_ascii() && b.is_ascii() {
        a.eq_ignore_ascii_case(b)
    } else {
        normalize_lower(a) == normalize_lower(b)
    }
}
