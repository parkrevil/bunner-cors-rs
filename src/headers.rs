use crate::constants::header;
use indexmap::IndexMap;
use std::cell::RefCell;
use std::mem;

#[cfg(debug_assertions)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PoolStats {
    pub acquired: usize,
    pub released: usize,
    pub current_in_use: usize,
    pub max_in_use: usize,
}

#[cfg(debug_assertions)]
thread_local! {
    static HEADER_POOL_STATS: RefCell<PoolStats> = RefCell::new(PoolStats::default());
}

#[cfg(debug_assertions)]
fn header_stats_record_acquire() {
    HEADER_POOL_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.acquired += 1;
        stats.current_in_use += 1;
        if stats.current_in_use > stats.max_in_use {
            stats.max_in_use = stats.current_in_use;
        }
    });
}

#[cfg(debug_assertions)]
fn header_stats_record_release() {
    HEADER_POOL_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.released += 1;
        stats.current_in_use = stats.current_in_use.saturating_sub(1);
    });
}

#[cfg(all(test, debug_assertions))]
pub(crate) fn header_pool_stats() -> PoolStats {
    HEADER_POOL_STATS.with(|stats| *stats.borrow())
}

#[cfg(all(test, debug_assertions))]
pub(crate) fn header_pool_reset() {
    HEADER_POOL_STATS.with(|stats| *stats.borrow_mut() = PoolStats::default());
}

/// Canonical map type used for returning header modifications to callers.
///
/// The map preserves insertion order to keep the resulting response headers
/// stable, which simplifies snapshot testing and debugging.
pub type Headers = IndexMap<String, String>;

const HEADER_BUFFER_POOL_LIMIT: usize = 64;

thread_local! {
    static HEADER_BUFFER_POOL: RefCell<Vec<Vec<(String, String)>>> = const { RefCell::new(Vec::new()) };
}

fn acquire_entries(estimate: usize) -> Vec<(String, String)> {
    let capacity = estimate.max(4);

    let entries = HEADER_BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        match pool.pop() {
            Some(mut entries) => {
                let required = capacity.saturating_sub(entries.len());
                if required > 0 {
                    entries.reserve(required);
                }
                entries
            }
            None => Vec::with_capacity(capacity),
        }
    });

    header_stats_record_acquire();

    entries
}

fn release_entries(mut entries: Vec<(String, String)>) {
    if entries.capacity() == 0 {
        return;
    }

    entries.clear();

    header_stats_record_release();

    HEADER_BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < HEADER_BUFFER_POOL_LIMIT {
            pool.push(entries);
        }
    });
}

#[derive(Debug, Clone)]
pub(crate) struct HeaderCollection {
    vary: Option<String>,
    headers: Vec<(String, String)>,
}

impl HeaderCollection {
    pub(crate) fn new() -> Self {
        Self::with_estimate(4)
    }

    pub(crate) fn with_estimate(estimate: usize) -> Self {
        Self {
            vary: None,
            headers: acquire_entries(estimate),
        }
    }

    pub(crate) fn push(&mut self, name: String, value: String) {
        if name.eq_ignore_ascii_case(header::VARY) {
            self.add_vary(value);
        } else if let Some((_, existing)) = self
            .headers
            .iter_mut()
            .rev()
            .find(|(existing_name, _)| existing_name.eq_ignore_ascii_case(&name))
        {
            *existing = value;
        } else {
            self.headers.push((name, value));
        }
    }

    pub(crate) fn add_vary<S: Into<String>>(&mut self, value: S) {
        let mut entries: Vec<String> = self
            .vary
            .as_ref()
            .map(|existing| {
                existing
                    .split(',')
                    .map(|part| part.trim().to_string())
                    .filter(|part| !part.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let incoming = value.into().trim().to_string();
        if !incoming.is_empty() {
            entries.push(incoming);
        }

        if entries.is_empty() {
            self.vary = None;
            return;
        }

        let mut deduped: Vec<String> = Vec::with_capacity(entries.len());
        for entry in entries {
            if deduped
                .iter()
                .any(|existing| existing.eq_ignore_ascii_case(&entry))
            {
                continue;
            }
            deduped.push(entry);
        }

        let value = deduped.join(", ");
        self.vary = Some(value);
    }

    pub(crate) fn extend(&mut self, mut other: HeaderCollection) {
        if let Some(vary) = other.vary.take() {
            self.add_vary(vary);
        }

        for (name, value) in other.headers.drain(..) {
            if name.eq_ignore_ascii_case(header::VARY) {
                self.add_vary(value);
            } else if let Some((_, existing)) = self
                .headers
                .iter_mut()
                .rev()
                .find(|(existing_name, _)| existing_name.eq_ignore_ascii_case(&name))
            {
                *existing = value;
            } else {
                self.headers.push((name, value));
            }
        }
    }

    pub(crate) fn into_headers(mut self) -> Headers {
        let mut headers =
            Headers::with_capacity(self.headers.len() + usize::from(self.vary.is_some()));

        if let Some(vary) = self.vary.take() {
            headers.insert(header::VARY.to_string(), vary);
        }

        for (name, value) in self.headers.drain(..) {
            headers.insert(name, value);
        }

        headers
    }
}

impl Default for HeaderCollection {
    fn default() -> Self {
        Self::with_estimate(4)
    }
}

impl Drop for HeaderCollection {
    fn drop(&mut self) {
        let entries = mem::take(&mut self.headers);
        release_entries(entries);
    }
}

#[cfg(test)]
#[path = "headers_test.rs"]
mod headers_test;
