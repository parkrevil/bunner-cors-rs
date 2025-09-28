mod policy;

pub use policy::{
    CorsDecision, CorsOptions, CorsPolicy, Header, Origin, OriginPredicateFn, PreflightResult,
    RequestContext, SimpleResult,
};
