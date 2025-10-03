use super::*;
use crate::constants::header;

mod new {
    use super::*;

    #[test]
    fn should_start_with_empty_headers_when_new_called_then_initialize_collection() {
        let collection = HeaderCollection::new();

        assert!(collection.into_headers().is_empty());
    }
}

mod default_impl {
    use super::*;

    #[test]
    fn should_delegate_to_with_estimate_when_default_called_then_use_pool_allocation() {
        let collection = HeaderCollection::default();

        assert!(collection.into_headers().is_empty());
    }
}

mod push {
    use super::*;

    #[test]
    fn should_store_header_once_when_header_regular_then_persist_value() {
        let mut collection = HeaderCollection::new();

        collection.push("Access-Control-Expose-Headers".into(), "X-Trace".into());

        let headers = collection.into_headers();
        assert_eq!(
            headers.get("Access-Control-Expose-Headers"),
            Some(&"X-Trace".to_string())
        );
    }

    #[test]
    fn should_use_deduplicated_value_when_header_vary_then_preserve_first_entry() {
        let mut collection = HeaderCollection::new();

        collection.push(header::VARY.to_string(), "Origin".into());
        collection.push(header::VARY.to_string(), "origin".into());

        let headers = collection.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }

    #[test]
    fn should_replace_existing_value_when_header_repeated_then_update_last_entry() {
        let mut collection = HeaderCollection::new();
        collection.push("Access-Control-Max-Age".into(), "600".into());

        collection.push("access-control-max-age".into(), "300".into());

        let headers = collection.into_headers();
        assert_eq!(headers.len(), 1);
        assert_eq!(
            headers.get("Access-Control-Max-Age"),
            Some(&"300".to_string())
        );
    }
}

mod add_vary {
    use super::*;

    #[test]
    fn should_store_unique_entries_when_values_have_mixed_case_then_deduplicate_case_insensitively()
    {
        let mut collection = HeaderCollection::new();

        collection.add_vary("Origin");
        collection.add_vary("Access-Control-Request-Headers");
        collection.add_vary("origin");

        let headers = collection.into_headers();
        assert_eq!(
            headers.get(header::VARY),
            Some(&"Origin, Access-Control-Request-Headers".to_string())
        );
    }

    #[test]
    fn should_remove_vary_header_when_value_whitespace_then_skip_entry() {
        let mut collection = HeaderCollection::new();

        collection.add_vary("   ");

        let headers = collection.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn should_preserve_existing_entries_when_value_whitespace_then_ignore_addition() {
        let mut collection = HeaderCollection::new();
        collection.add_vary("Origin");

        collection.add_vary("   ");

        let headers = collection.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }
}

mod extend {
    use super::*;

    #[test]
    fn should_combine_and_deduplicate_when_extending_collections_then_merge_headers() {
        let mut base = HeaderCollection::new();
        base.push("Access-Control-Allow-Credentials".into(), "true".into());
        base.add_vary("Origin");
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "origin".into());
        other.push("Access-Control-Expose-Headers".into(), "X-Trace".into());

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
        assert_eq!(
            headers.get("Access-Control-Allow-Credentials"),
            Some(&"true".to_string())
        );
        assert_eq!(
            headers.get("Access-Control-Expose-Headers"),
            Some(&"X-Trace".to_string())
        );
    }

    #[test]
    fn should_remove_vary_header_when_extending_with_whitespace_then_skip_entry() {
        let mut base = HeaderCollection::new();
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        base.extend(other);

        let headers = base.into_headers();
        assert!(!headers.contains_key(header::VARY));
    }

    #[test]
    fn should_preserve_vary_value_when_extending_with_whitespace_then_retain_existing_entry() {
        let mut base = HeaderCollection::new();
        base.add_vary("Origin");
        let mut other = HeaderCollection::new();
        other.push(header::VARY.to_string(), "   ".into());

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }

    #[test]
    fn should_merge_vary_header_when_extending_with_other_collection_then_combine_entries() {
        let mut base = HeaderCollection::new();
        base.add_vary("Access-Control-Request-Method");
        let mut other = HeaderCollection::new();
        other.add_vary("Origin");

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(
            headers.get(header::VARY),
            Some(&"Access-Control-Request-Method, Origin".to_string())
        );
    }

    #[test]
    fn should_handle_vary_entry_stored_in_headers_then_normalize_on_extend() {
        let mut base = HeaderCollection::new();
        let mut other = HeaderCollection::new();
        other.headers.push((header::VARY.to_string(), "Origin".into()));

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(headers.get(header::VARY), Some(&"Origin".to_string()));
    }

    #[test]
    fn should_update_existing_value_when_extending_with_duplicate_header_then_replace_entry() {
        let mut base = HeaderCollection::new();
        base.push("Access-Control-Max-Age".into(), "600".into());
        let mut other = HeaderCollection::new();
        other.push("access-control-max-age".into(), "300".into());

        base.extend(other);

        let headers = base.into_headers();
        assert_eq!(
            headers.get("Access-Control-Max-Age"),
            Some(&"300".to_string())
        );
    }
}

mod into_headers {
    use super::*;

    #[test]
    fn should_consume_collection_and_return_map_when_into_headers_called_then_produce_map() {
        let mut collection = HeaderCollection::new();
        collection.push("Access-Control-Allow-Methods".into(), "GET".into());

        let headers = collection.into_headers();

        assert_eq!(
            headers.get("Access-Control-Allow-Methods"),
            Some(&"GET".to_string())
        );
    }

    #[test]
    fn should_emit_vary_header_first_when_into_headers_called_then_preserve_ordering() {
        let mut collection = HeaderCollection::new();
        collection.add_vary("Origin");
        collection.push("Access-Control-Allow-Methods".into(), "GET".into());

        let headers = collection.into_headers();
        let mut keys = headers.keys();

        assert_eq!(keys.next(), Some(&header::VARY.to_string()));
        assert_eq!(
            keys.next(),
            Some(&"Access-Control-Allow-Methods".to_string())
        );
    }
}

#[cfg(debug_assertions)]
mod pool_instrumentation {
    use super::*;

    #[test]
    fn should_return_entries_to_pool_when_collection_dropped_then_balance_counts() {
        super::header_pool_reset();

        {
            let mut collection = HeaderCollection::with_estimate(2);
            collection.push("X-Debug".into(), "true".into());
        }

        let stats = super::header_pool_stats();
        assert_eq!(stats.acquired, stats.released);
        assert_eq!(stats.current_in_use, 0);
        assert!(stats.max_in_use >= 1);
    }
}

mod capacity_management {
    use super::*;

    #[test]
    fn should_expand_reused_entries_when_estimate_increases_then_reserve_capacity() {
        super::HEADER_BUFFER_POOL.with(|pool| pool.borrow_mut().clear());

        {
            let _collection = HeaderCollection::with_estimate(4);
        }

        let mut pool_capacities = Vec::new();
        super::HEADER_BUFFER_POOL.with(|pool| {
            let pool = pool.borrow();
            assert_eq!(pool.len(), 1);
            pool_capacities.push(pool[0].capacity());
        });

        let large_estimate = 32;
        let collection = HeaderCollection::with_estimate(large_estimate);
        assert!(collection.headers.capacity() >= large_estimate);
        assert!(pool_capacities[0] < large_estimate);

        drop(collection);
    }

    #[test]
    fn should_drop_entries_when_pool_full_then_avoid_pushing_over_limit() {
        super::HEADER_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            pool.clear();
            for _ in 0..super::HEADER_BUFFER_POOL_LIMIT {
                pool.push(Vec::with_capacity(4));
            }
        });

        {
            let _collection = HeaderCollection::with_estimate(4);
        }

        super::HEADER_BUFFER_POOL.with(|pool| {
            let pool = pool.borrow();
            assert_eq!(pool.len(), super::HEADER_BUFFER_POOL_LIMIT);
        });
    }

    #[test]
    fn should_skip_insertion_when_release_entries_called_at_capacity() {
        super::HEADER_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            pool.clear();
            for _ in 0..super::HEADER_BUFFER_POOL_LIMIT {
                pool.push(Vec::with_capacity(4));
            }
        });

        super::release_entries(vec![("X-Test".to_string(), "1".to_string())]);

        super::HEADER_BUFFER_POOL.with(|pool| {
            let pool = pool.borrow();
            assert_eq!(pool.len(), super::HEADER_BUFFER_POOL_LIMIT);
        });
    }

    #[test]
    fn should_reserve_additional_capacity_when_acquiring_reused_buffer() {
        super::HEADER_BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            pool.clear();
            pool.push(Vec::with_capacity(4));
        });

        let entries = super::acquire_entries(32);

        assert!(entries.capacity() >= 32);

        super::release_entries(entries);
    }

    #[test]
    fn should_release_entries_on_drop_when_capacity_present() {
        super::HEADER_BUFFER_POOL.with(|pool| pool.borrow_mut().clear());

        {
            let mut collection = HeaderCollection::with_estimate(4);
            collection.push("X-Debug".into(), "1".into());

            assert_eq!(collection.headers.len(), 1);
        }

        super::HEADER_BUFFER_POOL.with(|pool| {
            let pool = pool.borrow();
            assert_eq!(pool.len(), 1);
            assert!(pool[0].capacity() >= 4);
        });
    }

    #[test]
    fn should_expand_reserved_capacity_when_reusing_smaller_buffer_then_requeue_on_drop() {
        super::HEADER_BUFFER_POOL.with(|pool| pool.borrow_mut().clear());

        super::release_entries(Vec::with_capacity(4));

        let mut collection = HeaderCollection::with_estimate(32);

        assert!(collection.headers.capacity() >= 32);
        assert!(collection.headers.is_empty());

        collection.push("X-Debug".into(), "1".into());

        drop(collection);

        super::HEADER_BUFFER_POOL.with(|pool| {
            let pool = pool.borrow();
            assert_eq!(pool.len(), 1);
            assert!(pool[0].capacity() >= 32);
        });
    }
}
