use crate::context::RequestContext;
use std::borrow::Cow;

#[doc(hidden)]
pub struct NormalizedRequest<'a> {
    method: Cow<'a, str>,
    origin: Cow<'a, str>,
    access_control_request_method: Cow<'a, str>,
    access_control_request_headers: Cow<'a, str>,
    access_control_request_private_network: bool,
}

impl<'a> NormalizedRequest<'a> {
    #[doc(hidden)]
    pub fn new(request: &'a RequestContext<'a>) -> Self {
        Self {
            method: Self::normalize_component(request.method),
            origin: Self::normalize_component(request.origin),
            access_control_request_method: Self::normalize_component(
                request.access_control_request_method,
            ),
            access_control_request_headers: Self::normalize_component(
                request.access_control_request_headers,
            ),
            access_control_request_private_network: request.access_control_request_private_network,
        }
    }

    fn normalize_component(value: &'a str) -> Cow<'a, str> {
        if value.is_ascii() {
            if let Some(index) = value
                .as_bytes()
                .iter()
                .position(|byte| byte.is_ascii_uppercase())
            {
                let mut owned = value.to_owned();
                // SAFETY: `index` lies within the string bounds and `make_ascii_lowercase`
                // operates in-place without altering the slice length.
                unsafe {
                    owned.as_mut_vec()[index..].make_ascii_lowercase();
                }
                Cow::Owned(owned)
            } else {
                Cow::Borrowed(value)
            }
        } else if value.chars().any(|ch| ch.is_uppercase()) {
            Cow::Owned(value.to_lowercase())
        } else {
            Cow::Borrowed(value)
        }
    }

    #[doc(hidden)]
    pub fn as_context(&self) -> RequestContext<'_> {
        RequestContext {
            method: self.method.as_ref(),
            origin: self.origin.as_ref(),
            access_control_request_method: self.access_control_request_method.as_ref(),
            access_control_request_headers: self.access_control_request_headers.as_ref(),
            access_control_request_private_network: self.access_control_request_private_network,
        }
    }

    #[doc(hidden)]
    pub fn is_options(&self) -> bool {
        self.method.as_ref() == "options"
    }
}

#[cfg(test)]
#[path = "normalized_request_test.rs"]
mod normalized_request_test;
