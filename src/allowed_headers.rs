/// Configuration for the `Access-Control-Allow-Headers` response value.
#[derive(Clone, Default, PartialEq, Eq)]
pub enum AllowedHeaders {
    #[default]
    MirrorRequest,
    List(Vec<String>),
    Any,
}

impl AllowedHeaders {
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::List(
            values
                .into_iter()
                .map(|value| value.into().trim().to_string())
                .collect(),
        )
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub fn allows_headers(&self, request_headers: &str) -> bool {
        match self {
            Self::Any | Self::MirrorRequest => true,
            Self::List(allowed) => {
                let request_headers = request_headers.trim();
                if request_headers.is_empty() {
                    return true;
                }

                request_headers
                    .split(',')
                    .map(|value| value.trim())
                    .filter(|value| !value.is_empty())
                    .all(|header| {
                        allowed
                            .iter()
                            .any(|allowed_header| allowed_header.eq_ignore_ascii_case(header))
                    })
            }
        }
    }
}

#[cfg(test)]
#[path = "allowed_headers_test.rs"]
mod allowed_headers_test;
