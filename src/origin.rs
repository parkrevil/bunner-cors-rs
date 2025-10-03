use crate::context::RequestContext;
use crate::util::{equals_ignore_case, lowercase_unicode_into, normalize_lower};
use once_cell::sync::Lazy;
use regex_automata::meta::{BuildError, Regex};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub type OriginPredicateFn = dyn for<'a> Fn(&str, &RequestContext<'a>) -> bool + Send + Sync;
pub type OriginCallbackFn =
    dyn for<'a> Fn(Option<&'a str>, &RequestContext<'a>) -> OriginDecision + Send + Sync;

#[derive(Clone, Default)]
pub enum Origin {
    #[default]
    Any,
    Exact(String),
    List(OriginList),
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

static REGEX_CACHE: Lazy<RwLock<HashMap<String, Regex>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

thread_local! {
    static ORIGIN_UNICODE_BUFFER: RefCell<String> = const { RefCell::new(String::new()) };
}

#[derive(Clone, Debug)]
pub enum OriginMatcher {
    Exact(String),
    Pattern(Regex),
    Bool(bool),
}

#[derive(Clone, Debug)]
pub struct OriginList {
    matchers: Vec<OriginMatcher>,
    compiled: CompiledOriginList,
}

impl OriginList {
    fn new(matchers: Vec<OriginMatcher>) -> Self {
        let compiled = CompiledOriginList::compile(&matchers);
        Self { matchers, compiled }
    }

    pub fn len(&self) -> usize {
        self.matchers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.matchers.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &OriginMatcher> {
        self.matchers.iter()
    }

    pub(crate) fn matches(&self, candidate: &str) -> bool {
        self.compiled.matches(candidate, &self.matchers)
    }
}

const SMALL_LIST_LINEAR_SCAN_THRESHOLD: usize = 4;

#[derive(Clone, Debug, Default)]
struct CompiledOriginList {
    ascii_exact: HashSet<AsciiExact>,
    unicode_exact: HashSet<String>,
    regexes: Vec<Regex>,
    allow_all: bool,
    prefer_linear_scan: bool,
}

impl CompiledOriginList {
    fn compile(matchers: &[OriginMatcher]) -> Self {
        let prefer_linear_scan = matchers.len() <= SMALL_LIST_LINEAR_SCAN_THRESHOLD;
        let mut compiled = Self {
            prefer_linear_scan,
            ..Self::default()
        };

        for matcher in matchers {
            match matcher {
                OriginMatcher::Exact(value) => {
                    if value.is_ascii() {
                        compiled.ascii_exact.insert(AsciiExact::new(value.clone()));
                    } else {
                        compiled.unicode_exact.insert(normalize_lower(value));
                    }
                }
                OriginMatcher::Pattern(regex) => compiled.regexes.push(regex.clone()),
                OriginMatcher::Bool(value) => {
                    if *value {
                        compiled.allow_all = true;
                    }
                }
            }
        }

        compiled
    }

    fn matches(&self, candidate: &str, matchers: &[OriginMatcher]) -> bool {
        if self.allow_all {
            return true;
        }

        if self.prefer_linear_scan {
            return matchers.iter().any(|matcher| matcher.matches(candidate));
        }

        if !self.ascii_exact.is_empty() && candidate.is_ascii() {
            let borrowed = AsciiCaseInsensitive::new(candidate);
            if self.ascii_exact.contains(borrowed) {
                return true;
            }
        }

        if !self.unicode_exact.is_empty() && !candidate.is_ascii() {
            let matched = ORIGIN_UNICODE_BUFFER.with(|buffer| {
                let mut buffer = buffer.borrow_mut();
                if lowercase_unicode_into(candidate, &mut buffer) {
                    self.unicode_exact.contains(buffer.as_str())
                } else {
                    self.unicode_exact.contains(candidate)
                }
            });

            if matched {
                return true;
            }
        }

        let haystack = candidate.as_bytes();
        for regex in &self.regexes {
            if regex.is_match(haystack) {
                return true;
            }
        }

        false
    }
}

#[derive(Clone, Debug, Eq)]
struct AsciiExact {
    value: String,
}

impl AsciiExact {
    fn new(value: String) -> Self {
        Self { value }
    }
}

impl PartialEq for AsciiExact {
    fn eq(&self, other: &Self) -> bool {
        self.value.eq_ignore_ascii_case(&other.value)
    }
}

impl Hash for AsciiExact {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.value.as_bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}

impl std::borrow::Borrow<AsciiCaseInsensitive> for AsciiExact {
    fn borrow(&self) -> &AsciiCaseInsensitive {
        AsciiCaseInsensitive::new(&self.value)
    }
}

#[repr(transparent)]
struct AsciiCaseInsensitive(str);

impl AsciiCaseInsensitive {
    fn new(value: &str) -> &Self {
        // SAFETY: AsciiCaseInsensitive is a transparent wrapper around str.
        unsafe { &*(value as *const str as *const AsciiCaseInsensitive) }
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl Hash for AsciiCaseInsensitive {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for byte in self.as_str().as_bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
    }
}

impl PartialEq for AsciiCaseInsensitive {
    fn eq(&self, other: &Self) -> bool {
        self.as_str().eq_ignore_ascii_case(other.as_str())
    }
}

impl Eq for AsciiCaseInsensitive {}

impl PartialEq<AsciiCaseInsensitive> for AsciiExact {
    fn eq(&self, other: &AsciiCaseInsensitive) -> bool {
        self.value.eq_ignore_ascii_case(other.as_str())
    }
}

impl PartialEq<AsciiExact> for AsciiCaseInsensitive {
    fn eq(&self, other: &AsciiExact) -> bool {
        other.value.eq_ignore_ascii_case(self.as_str())
    }
}

impl OriginMatcher {
    pub fn exact<S: Into<String>>(value: S) -> Self {
        Self::Exact(value.into())
    }

    pub fn pattern(regex: Regex) -> Self {
        Self::Pattern(regex)
    }

    pub fn pattern_str(pattern: &str) -> Result<Self, PatternError> {
        if let Some(regex) = Self::cached_pattern(pattern) {
            return Ok(Self::Pattern(regex));
        }
        let regex = Self::compile_pattern(pattern, PATTERN_COMPILE_BUDGET)?;
        Self::cache_pattern(pattern, &regex);
        Ok(Self::Pattern(regex))
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

    fn cached_pattern(pattern: &str) -> Option<Regex> {
        let cache = REGEX_CACHE.read().unwrap_or_else(|err| err.into_inner());
        cache.get(pattern).cloned()
    }

    fn cache_pattern(pattern: &str, regex: &Regex) {
        let mut cache = REGEX_CACHE.write().unwrap_or_else(|err| err.into_inner());
        cache.insert(pattern.to_owned(), regex.clone());
    }

    #[cfg(test)]
    pub(crate) fn pattern_str_with_budget(
        pattern: &str,
        budget: Duration,
    ) -> Result<Self, PatternError> {
        if let Some(regex) = Self::cached_pattern(pattern) {
            return Ok(Self::Pattern(regex));
        }
        let regex = Self::compile_pattern(pattern, budget)?;
        Self::cache_pattern(pattern, &regex);
        Ok(Self::Pattern(regex))
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
        let matchers = values.into_iter().map(Into::into).collect();
        Self::List(OriginList::new(matchers))
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
            Origin::List(list) => {
                if let Some(origin) = request_origin {
                    if list.matches(origin) {
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

#[cfg(test)]
pub(crate) fn clear_regex_cache() {
    REGEX_CACHE
        .write()
        .unwrap_or_else(|err| err.into_inner())
        .clear();
}

#[cfg(test)]
pub(crate) fn regex_cache_size() -> usize {
    REGEX_CACHE
        .read()
        .unwrap_or_else(|err| err.into_inner())
        .len()
}

#[cfg(test)]
pub(crate) fn regex_cache_contains(pattern: &str) -> bool {
    REGEX_CACHE
        .read()
        .unwrap_or_else(|err| err.into_inner())
        .contains_key(pattern)
}
