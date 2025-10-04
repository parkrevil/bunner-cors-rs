use crate::allowed_headers::AllowedHeaders;
use crate::constants::header;
use crate::context::RequestContext;
use crate::exposed_headers::ExposedHeaders;
use crate::headers::HeaderCollection;
use crate::options::CorsOptions;
use crate::origin::OriginDecision;
use crate::result::CorsError;

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
    ) -> Result<(HeaderCollection, OriginDecision), CorsError> {
        let normalized_origin = normalized.origin;
        if normalized_origin.eq_ignore_ascii_case("null") && !self.options.allow_null_origin {
            let mut headers = HeaderCollection::with_estimate(1);
            headers.add_vary(header::ORIGIN);
            return Ok((headers, OriginDecision::Disallow));
        }

        let request_origin = if normalized_origin.is_empty() {
            None
        } else {
            Some(normalized_origin)
        };

        match self.options.origin.resolve(request_origin, normalized) {
            OriginDecision::Any => {
                if self.options.credentials {
                    return Err(CorsError::InvalidOriginAnyWithCredentials);
                }
                let mut headers = HeaderCollection::with_estimate(1);
                headers.push(
                    header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                    "*".to_string(),
                );
                Ok((headers, OriginDecision::Any))
            }
            OriginDecision::Exact(value) => {
                let mut headers = HeaderCollection::with_estimate(2);
                headers.add_vary(header::ORIGIN);
                headers.push(
                    header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                    value.clone(),
                );
                Ok((headers, OriginDecision::Exact(value)))
            }
            OriginDecision::Mirror => {
                let capacity = if original.origin.is_empty() { 1 } else { 2 };
                let mut headers = HeaderCollection::with_estimate(capacity);
                headers.add_vary(header::ORIGIN);
                if !original.origin.is_empty() {
                    headers.push(
                        header::ACCESS_CONTROL_ALLOW_ORIGIN.to_string(),
                        original.origin.to_string(),
                    );
                    Ok((headers, OriginDecision::Mirror))
                } else {
                    Ok((headers, OriginDecision::Disallow))
                }
            }
            OriginDecision::Disallow => {
                let mut headers = HeaderCollection::with_estimate(1);
                headers.add_vary(header::ORIGIN);
                Ok((headers, OriginDecision::Disallow))
            }
            OriginDecision::Skip => Ok((HeaderCollection::new(), OriginDecision::Skip)),
        }
    }

    pub(crate) fn build_methods_header(&self) -> HeaderCollection {
        if let Some(value) = self.options.methods.header_value() {
            let mut headers = HeaderCollection::with_estimate(1);
            headers.push(header::ACCESS_CONTROL_ALLOW_METHODS.to_string(), value);
            headers
        } else {
            HeaderCollection::new()
        }
    }

    pub(crate) fn build_credentials_header(&self) -> HeaderCollection {
        if self.options.credentials {
            let mut headers = HeaderCollection::with_estimate(1);
            headers.push(
                header::ACCESS_CONTROL_ALLOW_CREDENTIALS.to_string(),
                "true".to_string(),
            );
            headers
        } else {
            HeaderCollection::new()
        }
    }

    pub(crate) fn build_allowed_headers(&self) -> HeaderCollection {
        match &self.options.allowed_headers {
            AllowedHeaders::List(values) if values.is_empty() => HeaderCollection::new(),
            AllowedHeaders::List(values) => {
                let mut headers = HeaderCollection::with_estimate(1);
                headers.push(
                    header::ACCESS_CONTROL_ALLOW_HEADERS.to_string(),
                    values.join(","),
                );
                headers
            }

            AllowedHeaders::Any => {
                let mut headers = HeaderCollection::with_estimate(1);
                headers.push(
                    header::ACCESS_CONTROL_ALLOW_HEADERS.to_string(),
                    "*".to_string(),
                );
                headers
            }
        }
    }

    pub(crate) fn build_private_network_header(
        &self,
        request: &RequestContext<'_>,
    ) -> HeaderCollection {
        let is_preflight = request.method.eq_ignore_ascii_case("OPTIONS");
        if self.options.allow_private_network
            && is_preflight
            && request.access_control_request_private_network
        {
            let mut headers = HeaderCollection::with_estimate(1);
            headers.push(
                header::ACCESS_CONTROL_ALLOW_PRIVATE_NETWORK.to_string(),
                "true".to_string(),
            );
            return headers;
        }
        HeaderCollection::new()
    }

    pub(crate) fn build_exposed_headers(&self) -> HeaderCollection {
        match &self.options.exposed_headers {
            ExposedHeaders::None => HeaderCollection::new(),
            ExposedHeaders::Any => {
                let mut headers = HeaderCollection::with_estimate(1);
                headers.push(
                    header::ACCESS_CONTROL_EXPOSE_HEADERS.to_string(),
                    "*".to_string(),
                );
                headers
            }
            ExposedHeaders::List(values) if values.is_empty() => HeaderCollection::new(),
            ExposedHeaders::List(values) => {
                let entries = values
                    .iter()
                    .map(|entry| entry.trim())
                    .filter(|entry| !entry.is_empty())
                    .collect::<Vec<_>>();

                if entries.is_empty() {
                    HeaderCollection::new()
                } else {
                    let value = entries.join(",");
                    let mut headers = HeaderCollection::with_estimate(1);
                    headers.push(header::ACCESS_CONTROL_EXPOSE_HEADERS.to_string(), value);
                    headers
                }
            }
        }
    }

    pub(crate) fn build_max_age_header(&self) -> HeaderCollection {
        if let Some(value) = self.options.max_age {
            let mut headers = HeaderCollection::with_estimate(1);
            headers.push(
                header::ACCESS_CONTROL_MAX_AGE.to_string(),
                value.to_string(),
            );
            return headers;
        }
        HeaderCollection::new()
    }

    pub(crate) fn build_timing_allow_origin_header(&self) -> HeaderCollection {
        if let Some(config) = &self.options.timing_allow_origin
            && let Some(value) = config.header_value()
        {
            let mut headers = HeaderCollection::with_estimate(1);
            headers.push(header::TIMING_ALLOW_ORIGIN.to_string(), value);
            return headers;
        }
        HeaderCollection::new()
    }
}

#[cfg(test)]
#[path = "header_builder_test.rs"]
mod header_builder_test;
