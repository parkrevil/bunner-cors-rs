use crate::util::normalize_lower;
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;

thread_local! {
    static REQUEST_HEADER_CACHE: RefCell<AllowedHeadersCache> = RefCell::new(AllowedHeadersCache::new());
}

#[derive(Default, Clone)]
pub struct AllowedHeadersCache {
    identity: (usize, usize),
    normalized_tokens: Vec<String>,
}

impl AllowedHeadersCache {
    pub fn new() -> Self {
        Self {
            identity: (0, 0),
            normalized_tokens: Vec::new(),
        }
    }

    pub fn prepare<'a>(&'a mut self, request_headers: &str) -> &'a [String] {
        let identity = (request_headers.as_ptr() as usize, request_headers.len());
        if self.identity != identity {
            self.identity = identity;
            self.normalized_tokens.clear();

            request_headers
                .split(',')
                .map(|value| value.trim())
                .filter(|value| !value.is_empty())
                .for_each(|header| {
                    self.normalized_tokens.push(normalize_lower(header));
                });
        }

        &self.normalized_tokens
    }

    pub fn reset(&mut self) {
        self.identity = (0, 0);
        self.normalized_tokens.clear();
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum AllowedHeaders {
    Any,
    List(AllowedHeaderList),
}

impl Default for AllowedHeaders {
    fn default() -> Self {
        AllowedHeaders::List(AllowedHeaderList::default())
    }
}

impl AllowedHeaders {
    pub fn list<I, S>(values: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut seen = HashSet::new();
        let mut deduped: Vec<String> = Vec::new();
        for value in values.into_iter() {
            let trimmed = value.into().trim().to_string();
            let key = normalize_lower(&trimmed);
            if seen.insert(key) {
                deduped.push(trimmed);
            }
        }

        Self::List(AllowedHeaderList::new(deduped, seen))
    }

    pub fn any() -> Self {
        Self::Any
    }

    pub fn allows_headers(&self, request_headers: &str) -> bool {
        match self {
            Self::Any => true,
            Self::List(allowed) => REQUEST_HEADER_CACHE.with(|cache| {
                let mut cache = cache.borrow_mut();
                allowed.allows_headers_with_cache(request_headers, &mut cache)
            }),
        }
    }

    pub fn allows_headers_with_cache(
        &self,
        request_headers: &str,
        cache: &mut AllowedHeadersCache,
    ) -> bool {
        match self {
            Self::Any => true,
            Self::List(allowed) => allowed.allows_headers_with_cache(request_headers, cache),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Default)]
pub struct AllowedHeaderList {
    values: Vec<String>,
    normalized: HashSet<String>,
}

impl AllowedHeaderList {
    fn new(values: Vec<String>, normalized: HashSet<String>) -> Self {
        Self { values, normalized }
    }

    pub fn values(&self) -> &[String] {
        &self.values
    }

    fn allows_headers_with_cache(
        &self,
        request_headers: &str,
        cache: &mut AllowedHeadersCache,
    ) -> bool {
        let request_headers = request_headers.trim();
        if request_headers.is_empty() {
            return true;
        }

        let normalized_tokens = cache.prepare(request_headers);
        if normalized_tokens.is_empty() {
            return true;
        }

        normalized_tokens
            .iter()
            .all(|normalized| self.normalized.contains(normalized.as_str()))
    }

    #[cfg(test)]
    fn allows_headers(&self, request_headers: &str) -> bool {
        let mut cache = AllowedHeadersCache::new();
        self.allows_headers_with_cache(request_headers, &mut cache)
    }
}

impl Deref for AllowedHeaderList {
    type Target = [String];

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

#[cfg(test)]
#[path = "allowed_headers_test.rs"]
mod allowed_headers_test;
