use std::cell::RefCell;

thread_local! {
    static CASEFOLD_BUFFERS: RefCell<(String, String)> = const { RefCell::new((String::new(), String::new())) };
}

#[doc(hidden)]
pub fn normalize_lower(value: &str) -> String {
    if value.is_ascii() {
        let mut owned = value.to_owned();
        owned.make_ascii_lowercase();
        owned
    } else {
        lowercase_unicode_if_needed(value).unwrap_or_else(|| value.to_owned())
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

    if !a_has_upper && !b_has_upper {
        return a == b;
    }

    CASEFOLD_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let (a_buf, b_buf) = &mut *buffers;

        let a_ref = if a_has_upper {
            if lowercase_unicode_into(a, a_buf) {
                a_buf.as_str()
            } else {
                a
            }
        } else {
            a
        };

        if !b_has_upper {
            a_ref == b
        } else {
            let b_ref = if lowercase_unicode_into(b, b_buf) {
                b_buf.as_str()
            } else {
                b
            };

            a_ref == b_ref
        }
    })
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

pub(crate) fn lowercase_unicode_if_needed(value: &str) -> Option<String> {
    for (idx, ch) in value.char_indices() {
        if ch.is_uppercase() {
            let mut lowered = String::with_capacity(value.len());
            lowered.push_str(&value[..idx]);
            lowered.extend(ch.to_lowercase());

            let tail_start = idx + ch.len_utf8();
            for tail_ch in value[tail_start..].chars() {
                if tail_ch.is_uppercase() {
                    lowered.extend(tail_ch.to_lowercase());
                } else {
                    lowered.push(tail_ch);
                }
            }

            return Some(lowered);
        }
    }

    None
}

pub(crate) fn lowercase_unicode_into(value: &str, buffer: &mut String) -> bool {
    buffer.clear();

    if buffer.capacity() < value.len() {
        buffer.reserve(value.len() - buffer.capacity());
    }

    for (idx, ch) in value.char_indices() {
        if ch.is_uppercase() {
            buffer.push_str(&value[..idx]);
            buffer.extend(ch.to_lowercase());

            let tail_start = idx + ch.len_utf8();
            for tail_ch in value[tail_start..].chars() {
                if tail_ch.is_uppercase() {
                    buffer.extend(tail_ch.to_lowercase());
                } else {
                    buffer.push(tail_ch);
                }
            }

            return true;
        }
    }

    false
}

#[cfg(test)]
#[path = "util_test.rs"]
mod util_test;
