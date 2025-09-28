use crate::constants::header;

/// Simple response header representation used by the CORS engine.
pub type Headers = std::collections::HashMap<String, String>;

#[derive(Default)]
pub(crate) struct HeaderCollection {
    headers: Headers,
}

impl HeaderCollection {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn push(&mut self, name: String, value: String) {
        if name.eq_ignore_ascii_case(header::VARY) {
            self.add_vary(value);
        } else {
            self.headers.insert(name, value);
        }
    }

    pub(crate) fn add_vary<S: Into<String>>(&mut self, value: S) {
        let value = value.into();
        if let Some(existing) = self.headers.get(header::VARY) {
            let new_value = format!("{}, {}", existing, value);
            self.headers.insert(header::VARY.to_string(), new_value);
        } else {
            self.headers.insert(header::VARY.to_string(), value);
        }
    }

    pub(crate) fn extend(&mut self, other: HeaderCollection) {
        for (name, value) in other.headers {
            if name.eq_ignore_ascii_case(header::VARY) {
                self.add_vary(value);
            } else {
                self.headers.insert(name, value);
            }
        }
    }

    pub(crate) fn into_headers(self) -> Headers {
        self.headers
    }
}
