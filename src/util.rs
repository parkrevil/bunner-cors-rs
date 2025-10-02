#[doc(hidden)]
pub fn normalize_lower(value: &str) -> String {
    if value.is_ascii() {
        value.to_ascii_lowercase()
    } else {
        value.to_lowercase()
    }
}

#[doc(hidden)]
pub fn equals_ignore_case(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }

    if a.is_ascii() && b.is_ascii() {
        return a.eq_ignore_ascii_case(b);
    }

    let a_has_upper = a.chars().any(|ch| ch.is_uppercase());
    let b_has_upper = b.chars().any(|ch| ch.is_uppercase());

    match (a_has_upper, b_has_upper) {
        (false, false) => a == b,
        (true, false) => normalize_lower(a) == b,
        (false, true) => a == normalize_lower(b),
        (true, true) => normalize_lower(a) == normalize_lower(b),
    }
}

pub(crate) fn is_http_token(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            matches!(
                byte,
                b'0'..=b'9'
                    | b'A'..=b'Z'
                    | b'a'..=b'z'
                    | b'!'
                    | b'#'
                    | b'$'
                    | b'%'
                    | b'&'
                    | b'\''
                    | b'*'
                    | b'+'
                    | b'-'
                    | b'.'
                    | b'^'
                    | b'_'
                    | b'`'
                    | b'|'
                    | b'~'
            )
        })
}

#[cfg(test)]
#[path = "util_test.rs"]
mod util_test;
