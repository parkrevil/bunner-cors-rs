mod common;

use bunner_cors_rs::constants::method;
use bunner_cors_rs::{AllowedHeaders, Cors, Origin, OriginMatcher};
use common::asserts::assert_preflight;
use common::builders::{PreflightRequestBuilder, cors, preflight_request};
use insta::assert_yaml_snapshot;
use serde::Serialize;

#[derive(Serialize)]
struct HeaderSnapshot {
    name: String,
    value: String,
}

#[derive(Serialize)]
struct PreflightSnapshot {
    headers: Vec<HeaderSnapshot>,
}

fn capture_preflight(cors: &Cors, request: PreflightRequestBuilder) -> PreflightSnapshot {
    let headers = assert_preflight(request.check(cors));
    let mut header_vec: Vec<_> = headers
        .into_iter()
        .map(|(name, value)| HeaderSnapshot { name, value })
        .collect();
    header_vec.sort_by(|a, b| a.name.cmp(&b.name));
    PreflightSnapshot {
        headers: header_vec,
    }
}

mod capture_preflight {
    use super::*;

    #[test]
    fn should_capture_default_preflight_when_snapshot_requested_then_match_snapshot() {
        let snapshot = super::capture_preflight(
            &cors().build(),
            preflight_request()
                .origin("https://snapshot.dev")
                .request_method(method::GET),
        );

        insta::with_settings!({prepend_module_to_snapshot => false}, {
            assert_yaml_snapshot!("default_preflight_snapshot", snapshot);
        });
    }

    #[test]
    fn should_capture_preflight_when_mirror_origin_configured_then_match_snapshot() {
        let cors = cors()
            .origin(Origin::list([OriginMatcher::exact("https://mirror.dev")]))
            .credentials(true)
            .allowed_headers(AllowedHeaders::list(["X-Trace-Id"]))
            .max_age("3600")
            .build();

        let snapshot = super::capture_preflight(
            &cors,
            preflight_request()
                .origin("https://mirror.dev")
                .request_method(method::POST)
                .request_headers("X-Trace-Id"),
        );

        insta::with_settings!({prepend_module_to_snapshot => false}, {
            assert_yaml_snapshot!("mirror_origin_preflight_snapshot", snapshot);
        });
    }

    #[test]
    fn should_capture_preflight_when_strict_origin_configured_then_match_snapshot() {
        let cors = cors()
            .origin(Origin::list([OriginMatcher::pattern_str(
                r"^https://.*\.strict\.dev$",
            )
            .unwrap()]))
            .methods([method::GET, method::POST])
            .credentials(true)
            .allowed_headers(AllowedHeaders::list(["X-Strict", "X-Trace"]))
            .exposed_headers(["X-Result"])
            .max_age("0")
            .build();

        let snapshot = super::capture_preflight(
            &cors,
            preflight_request()
                .origin("https://api.strict.dev")
                .request_method(method::POST)
                .request_headers("X-Strict, X-Trace"),
        );

        insta::with_settings!({prepend_module_to_snapshot => false}, {
            assert_yaml_snapshot!("strict_origin_preflight_snapshot", snapshot);
        });
    }
}
