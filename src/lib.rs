mod allowed_headers;
mod allowed_methods;
pub mod constants;
mod context;
mod cors;
mod exposed_headers;
mod header_builder;
mod headers;
mod normalized_request;
mod options;
mod origin;
mod result;
mod timing_allow_origin;
mod util;

pub use allowed_headers::AllowedHeaders;
pub use allowed_methods::AllowedMethods;
pub use context::RequestContext;
pub use cors::Cors;
pub use exposed_headers::ExposedHeaders;
pub use headers::Headers;
pub use options::{CorsOptions, ValidationError};
pub use origin::{
    Origin, OriginCallbackFn, OriginDecision, OriginMatcher, OriginPredicateFn, PatternError,
};
pub use result::{
    CorsDecision, CorsError, PreflightRejection, PreflightRejectionReason, SimpleRejection,
    SimpleRejectionReason,
};
pub use timing_allow_origin::TimingAllowOrigin;

#[doc(hidden)]
pub use normalized_request::NormalizedRequest;
#[doc(hidden)]
pub use util::{equals_ignore_case, normalize_lower};
