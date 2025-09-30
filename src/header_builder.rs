use crate::allowed_headers::AllowedHeaders;
use crate::constants::header;
use crate::context::RequestContext;
use crate::headers::HeaderCollection;
use crate::options::CorsOptions;
use crate::origin::OriginDecision;

pub(crate) struct HeaderBuilder<'a> {
    options: &'a CorsOptions,
}

impl<'a> HeaderBuilder<'a> {
    pub(crate) fn new(options: &'a CorsOptions) -> Self {
        Self { options }
    }

    pub(crate) fn build_origin_headers(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> (HeaderCollection, bool) {
        let mut headers = HeaderCollection::new();
        let decision = self.options.origin.resolve(
            if normalized.origin.is_empty() {
                None
            } else {
                Some(normalized.origin)
            },
            normalized,
        );

        match decision {
            OriginDecision::Any => {
                headers.push(
                    header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                    "*".to_string(),
                );
            }
            OriginDecision::Exact(value) => {
                headers.add_vary(header::ORIGIN);
                headers.push(header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(), value);
            }
            OriginDecision::Mirror => {
                headers.add_vary(header::ORIGIN);
                if !original.origin.is_empty() {
                    headers.push(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                        original.origin.to_string(),
                    );
                }
            }
            OriginDecision::Disallow => {
                if self.options.origin.vary_on_disallow() {
                    headers.add_vary(header::ORIGIN);
                }
            }
            OriginDecision::Skip => {
                return (HeaderCollection::new(), true);
            }
        }

        (headers, false)
    }

    pub(crate) fn build_methods_header(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if let Some(value) = self.options.methods.header_value() {
            headers.push(header::ACCESS_CONTROL_ALLOW_METHODS.to_string(), value);
        }
        headers
    }

    pub(crate) fn build_credentials_header(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if self.options.credentials {
            headers.push(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS.to_string(),
                "true".to_string(),
            );
        }
        headers
    }

    pub(crate) fn build_allowed_headers(&self, request: &RequestContext<'_>) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        match &self.options.allowed_headers {
            AllowedHeaders::List(values) => {
                if !values.is_empty() {
                    headers.push(
                        header::ACCESS_CONTROL_ALLOW_HEADERS.to_string(),
                        values.join(","),
                    );
                }
            }
            AllowedHeaders::MirrorRequest => {
                headers.add_vary(header::ACCESS_CONTROL_REQUEST_HEADERS);
                if !request.access_control_request_headers.is_empty() {
                    headers.push(
                        header::ACCESS_CONTROL_ALLOW_HEADERS.to_string(),
                        request.access_control_request_headers.to_string(),
                    );
                }
            }
            AllowedHeaders::Any => {
                headers.push(
                    header::ACCESS_CONTROL_ALLOW_HEADERS.to_string(),
                    "*".to_string(),
                );
            }
        }
        headers
    }

    pub(crate) fn build_private_network_header(
        &self,
        request: &RequestContext<'_>,
    ) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if request.access_control_request_private_network {
            headers.push(
                header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK.to_string(),
                "true".to_string(),
            );
        }
        headers
    }

    pub(crate) fn build_exposed_headers(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if let Some(values) = &self.options.exposed_headers
            && !values.is_empty()
        {
            headers.push(
                header::ACCESS_CONTROL_EXPOSE_HEADERS.to_string(),
                values.join(","),
            );
        }
        headers
    }

    pub(crate) fn build_max_age_header(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if let Some(value) = &self.options.max_age
            && !value.is_empty()
        {
            headers.push(header::ACCESS_CONTROL_MAX_AGE.to_string(), value.clone());
        }
        headers
    }
}

#[cfg(test)]
#[path = "header_builder_test.rs"]
mod header_builder_test;
