use crate::context::RequestContext;
use regex::Regex;
use std::sync::Arc;

pub type OriginPredicateFn = dyn for<'a> Fn(&str, &RequestContext<'a>) -> bool + Send + Sync;
pub type OriginCallbackFn =
    dyn for<'a> Fn(Option<&'a str>, &RequestContext<'a>) -> OriginDecision + Send + Sync;

/// Strategy used to decide whether a request origin is accepted.
#[derive(Clone, Default)]
pub enum Origin {
    #[default]
    Any,
    Exact(String),
    List(Vec<OriginMatcher>),
    Predicate(Arc<OriginPredicateFn>),
    Custom(Arc<OriginCallbackFn>),
}

/// Decision outcome when resolving an origin.
#[derive(Debug, Clone)]
pub enum OriginDecision {
    Any,
    Exact(String),
    Mirror,
    Disallow,
    Skip,
}

impl OriginDecision {
    pub fn any() -> Self {
        Self::Any
    }

    pub fn exact<S: Into<String>>(value: S) -> Self {
        Self::Exact(value.into())
    }

    pub fn mirror() -> Self {
        Self::Mirror
    }

    pub fn disallow() -> Self {
        Self::Disallow
    }

    pub fn skip() -> Self {
        Self::Skip
    }
}

impl From<bool> for OriginDecision {
    fn from(value: bool) -> Self {
        if value {
            OriginDecision::Mirror
        } else {
            OriginDecision::Skip
        }
    }
}

impl<T> From<Option<T>> for OriginDecision
where
    T: Into<String>,
{
    fn from(value: Option<T>) -> Self {
        match value {
            Some(inner) => OriginDecision::Exact(inner.into()),
            None => OriginDecision::Skip,
        }
    }
}

/// Matcher that determines if an origin is allowed.
#[derive(Clone)]
pub enum OriginMatcher {
    Exact(String),
    Pattern(Regex),
    Bool(bool),
}

impl OriginMatcher {
    pub fn exact<S: Into<String>>(value: S) -> Self {
        Self::Exact(value.into())
    }

    pub fn pattern(regex: Regex) -> Self {
        Self::Pattern(regex)
    }

    pub fn matches(&self, candidate: &str) -> bool {
        match self {
            OriginMatcher::Exact(value) => value == candidate,
            OriginMatcher::Pattern(regex) => regex.is_match(candidate),
            OriginMatcher::Bool(value) => *value,
        }
    }
}

impl From<String> for OriginMatcher {
    fn from(value: String) -> Self {
        OriginMatcher::Exact(value)
    }
}

impl From<&str> for OriginMatcher {
    fn from(value: &str) -> Self {
        OriginMatcher::Exact(value.to_owned())
    }
}

impl From<bool> for OriginMatcher {
    fn from(value: bool) -> Self {
        OriginMatcher::Bool(value)
    }
}

impl Origin {
    pub fn any() -> Self {
        Self::Any
    }

    pub fn exact<S: Into<String>>(value: S) -> Self {
        Self::Exact(value.into())
    }

    pub fn list<I, T>(values: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: Into<OriginMatcher>,
    {
        Self::List(values.into_iter().map(Into::into).collect())
    }

    pub fn predicate<F>(predicate: F) -> Self
    where
        F: for<'a> Fn(&str, &RequestContext<'a>) -> bool + Send + Sync + 'static,
    {
        Self::Predicate(Arc::new(predicate))
    }

    pub fn custom<F>(callback: F) -> Self
    where
        F: for<'a> Fn(Option<&'a str>, &RequestContext<'a>) -> OriginDecision
            + Send
            + Sync
            + 'static,
    {
        Self::Custom(Arc::new(callback))
    }

    pub fn disabled() -> Self {
        Self::custom(|_, _| OriginDecision::Skip)
    }

    pub fn resolve(
        &self,
        request_origin: Option<&str>,
        ctx: &RequestContext<'_>,
    ) -> OriginDecision {
        match self {
            Origin::Any => OriginDecision::Any,
            Origin::Exact(value) => OriginDecision::Exact(value.clone()),
            Origin::List(matchers) => {
                if let Some(origin) = request_origin {
                    if matchers.iter().any(|matcher| matcher.matches(origin)) {
                        OriginDecision::Mirror
                    } else {
                        OriginDecision::Disallow
                    }
                } else {
                    OriginDecision::Disallow
                }
            }
            Origin::Predicate(predicate) => {
                if let Some(origin) = request_origin {
                    if predicate(origin, ctx) {
                        OriginDecision::Mirror
                    } else {
                        OriginDecision::Disallow
                    }
                } else {
                    OriginDecision::Disallow
                }
            }
            Origin::Custom(callback) => callback(request_origin, ctx),
        }
    }

    pub fn vary_on_disallow(&self) -> bool {
        !matches!(self, Origin::Any)
    }
}
