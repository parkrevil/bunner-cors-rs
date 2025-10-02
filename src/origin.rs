use crate::util::equals_ignore_case;
use crate::context::RequestContext;
use regex_automata::meta::{BuildError, Regex};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};

pub type OriginPredicateFn = dyn for<'a> Fn(&str, &RequestContext<'a>) -> bool + Send + Sync;
pub type OriginCallbackFn =
    dyn for<'a> Fn(Option<&'a str>, &RequestContext<'a>) -> OriginDecision + Send + Sync;

#[derive(Clone, Default)]
pub enum Origin {
    #[default]
    Any,
    Exact(String),
    List(Vec<OriginMatcher>),
    Predicate(Arc<OriginPredicateFn>),
    Custom(Arc<OriginCallbackFn>),
}

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

#[derive(Debug)]
pub enum PatternError {
    Build(Box<BuildError>),
    Timeout { elapsed: Duration, budget: Duration },
    TooLong { length: usize, max: usize },
}

impl fmt::Display for PatternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatternError::Build(_) => write!(f, "failed to compile origin pattern"),
            PatternError::Timeout { .. } => {
                write!(f, "compiling origin pattern exceeded the configured budget")
            }
            PatternError::TooLong { length, max } => write!(
                f,
                "origin pattern length {} exceeds maximum allowed {}",
                length, max
            ),
        }
    }
}

impl std::error::Error for PatternError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            PatternError::Build(err) => Some(err.as_ref()),
            _ => None,
        }
    }
}

const PATTERN_COMPILE_BUDGET: Duration = Duration::from_millis(100);
const MAX_PATTERN_LENGTH: usize = 50_000;
const MAX_ORIGIN_LENGTH: usize = 4_096;

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

    pub fn pattern_str(pattern: &str) -> Result<Self, PatternError> {
        Self::compile_pattern(pattern, PATTERN_COMPILE_BUDGET).map(Self::Pattern)
    }

    fn compile_pattern(pattern: &str, budget: Duration) -> Result<Regex, PatternError> {
        if pattern.len() > MAX_PATTERN_LENGTH {
            return Err(PatternError::TooLong {
                length: pattern.len(),
                max: MAX_PATTERN_LENGTH,
            });
        }

        let started = Instant::now();
        let regex = Regex::new(&format!("(?i:{pattern})"))
            .map_err(|err| PatternError::Build(Box::new(err)))?;
        let elapsed = started.elapsed();
        if elapsed > budget {
            return Err(PatternError::Timeout { elapsed, budget });
        }

        Ok(regex)
    }

    #[cfg(test)]
    pub(crate) fn pattern_str_with_budget(
        pattern: &str,
        budget: Duration,
    ) -> Result<Self, PatternError> {
        Self::compile_pattern(pattern, budget).map(Self::Pattern)
    }

    pub fn matches(&self, candidate: &str) -> bool {
        match self {
            OriginMatcher::Exact(value) => equals_ignore_case(value, candidate),
            OriginMatcher::Pattern(regex) => regex.is_match(candidate.as_bytes()),
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
        if let Some(origin) = request_origin
            && origin.len() > MAX_ORIGIN_LENGTH
        {
            return OriginDecision::Disallow;
        }

        match self {
            Origin::Any => match request_origin {
                Some(_) => OriginDecision::Any,
                None => OriginDecision::Skip,
            },
            Origin::Exact(value) => match request_origin {
                Some(origin) if equals_ignore_case(value, origin) => {
                    OriginDecision::Exact(value.clone())
                }
                Some(_) => OriginDecision::Disallow,
                None => OriginDecision::Skip,
            },
            Origin::List(matchers) => {
                if let Some(origin) = request_origin {
                    if matchers.iter().any(|matcher| matcher.matches(origin)) {
                        OriginDecision::Mirror
                    } else {
                        OriginDecision::Disallow
                    }
                } else {
                    OriginDecision::Skip
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
                    OriginDecision::Skip
                }
            }
            Origin::Custom(callback) => callback(request_origin, ctx),
        }
    }

    pub fn vary_on_disallow(&self) -> bool {
        !matches!(self, Origin::Any)
    }
}

#[cfg(test)]
#[path = "origin_test.rs"]
mod origin_test;
