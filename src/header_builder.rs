use crate::allowed_headers::AllowedHeaders;
use crate::constants::header;
use crate::context::RequestContext;
use crate::headers::HeaderCollection;
use crate::options::CorsOptions;
use crate::origin::OriginDecision;
use crate::result::CorsError;

pub(crate) struct HeaderBuilder<'a> {
    options: &'a CorsOptions,
}

#[derive(Debug)]
pub(crate) enum OriginOutcome {
    Allow(HeaderCollection),
    Disallow(HeaderCollection),
    Skip,
}

impl<'a> HeaderBuilder<'a> {
    pub(crate) fn new(options: &'a CorsOptions) -> Self {
        Self { options }
    }

    pub(crate) fn build_origin_headers(
        &self,
        original: &RequestContext<'_>,
        normalized: &RequestContext<'_>,
    ) -> Result<OriginOutcome, CorsError> {
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
                if self.options.credentials {
                    return Err(CorsError::InvalidOriginAnyWithCredentials);
                }
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
                } else {
                    return Ok(OriginOutcome::Disallow(headers));
                }
            }
            OriginDecision::Disallow => {
                if self.options.origin.vary_on_disallow() {
                    headers.add_vary(header::ORIGIN);
                }
                return Ok(OriginOutcome::Disallow(headers));
            }
            OriginDecision::Skip => {
                return Ok(OriginOutcome::Skip);
            }
        }

        Ok(OriginOutcome::Allow(headers))
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

    pub(crate) fn build_allowed_headers(&self) -> HeaderCollection {
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
        let is_preflight = request.method.eq_ignore_ascii_case("OPTIONS");
        if self.options.allow_private_network
            && is_preflight
            && request.access_control_request_private_network
        {
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
            let value = values
                .iter()
                .map(|entry| entry.trim())
                .collect::<Vec<_>>()
                .join(",");

            if !value.is_empty() {
                headers.push(header::ACCESS_CONTROL_EXPOSE_HEADERS.to_string(), value);
            }
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

    pub(crate) fn build_timing_allow_origin_header(&self) -> HeaderCollection {
        let mut headers = HeaderCollection::new();
        if let Some(config) = &self.options.timing_allow_origin
            && let Some(value) = config.header_value()
        {
            headers.push(header::TIMING_ALLOW_ORIGIN.to_string(), value);
        }
        headers
    }
}

#[cfg(test)]
#[path = "header_builder_test.rs"]
mod header_builder_test;
