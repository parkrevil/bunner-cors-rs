use crate::context::RequestContext;
use crate::util::lowercase_unicode_into;
use std::borrow::Cow;
use std::cell::RefCell;
use std::mem;

#[cfg(debug_assertions)]
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct PoolStats {
    pub acquired: usize,
    pub released: usize,
    pub current_in_use: usize,
    pub max_in_use: usize,
}

#[cfg(debug_assertions)]
thread_local! {
    static NORMALIZATION_POOL_STATS: RefCell<PoolStats> = RefCell::new(PoolStats::default());
}

#[cfg(debug_assertions)]
fn normalization_stats_record_acquire() {
    NORMALIZATION_POOL_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.acquired += 1;
        stats.current_in_use += 1;
        if stats.current_in_use > stats.max_in_use {
            stats.max_in_use = stats.current_in_use;
        }
    });
}

#[cfg(not(debug_assertions))]
fn normalization_stats_record_acquire() {}

#[cfg(debug_assertions)]
fn normalization_stats_record_release() {
    NORMALIZATION_POOL_STATS.with(|stats| {
        let mut stats = stats.borrow_mut();
        stats.released += 1;
        stats.current_in_use = stats.current_in_use.saturating_sub(1);
    });
}

#[cfg(not(debug_assertions))]
fn normalization_stats_record_release() {}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub(crate) fn normalization_pool_stats() -> PoolStats {
    NORMALIZATION_POOL_STATS.with(|stats| *stats.borrow())
}

#[cfg(debug_assertions)]
#[allow(dead_code)]
pub(crate) fn normalization_pool_reset() {
    NORMALIZATION_POOL_STATS.with(|stats| *stats.borrow_mut() = PoolStats::default());
}

const NORMALIZATION_BUFFER_POOL_LIMIT: usize = 16;

thread_local! {
    static NORMALIZATION_BUFFER_POOL: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) };
}

fn acquire_buffer(min_capacity: usize) -> String {
    let buffer = NORMALIZATION_BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if let Some(mut buffer) = pool.pop() {
            if buffer.capacity() < min_capacity {
                buffer.reserve(min_capacity - buffer.capacity());
            }
            buffer
        } else {
            String::with_capacity(min_capacity)
        }
    });

    normalization_stats_record_acquire();

    buffer
}

fn release_buffer(mut buffer: String) {
    normalization_stats_record_release();

    NORMALIZATION_BUFFER_POOL.with(|pool| {
        let mut pool = pool.borrow_mut();
        if pool.len() < NORMALIZATION_BUFFER_POOL_LIMIT {
            buffer.clear();
            pool.push(buffer);
        }
    });
}

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
                let mut owned = acquire_buffer(value.len());
                owned.clear();
                owned.push_str(value);
                // SAFETY: `index` lies within the string bounds and `make_ascii_lowercase`
                // operates in-place without altering the slice length.
                unsafe {
                    owned.as_mut_vec()[index..].make_ascii_lowercase();
                }
                Cow::Owned(owned)
            } else {
                Cow::Borrowed(value)
            }
        } else {
            let mut buffer = acquire_buffer(value.len());

            if lowercase_unicode_into(value, &mut buffer) {
                Cow::Owned(buffer)
            } else {
                release_buffer(buffer);
                Cow::Borrowed(value)
            }
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

impl<'a> Drop for NormalizedRequest<'a> {
    fn drop(&mut self) {
        fn release<'a>(target: &mut Cow<'a, str>) {
            if let Cow::Owned(buffer) = mem::replace(target, Cow::Borrowed("")) {
                release_buffer(buffer);
            }
        }

        release(&mut self.method);
        release(&mut self.origin);
        release(&mut self.access_control_request_method);
        release(&mut self.access_control_request_headers);
    }
}

#[cfg(test)]
#[path = "normalized_request_test.rs"]
mod normalized_request_test;
