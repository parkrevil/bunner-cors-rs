mod common;

use bunner_cors_rs::constants::{header, method};
use bunner_cors_rs::{AllowedHeaders, Origin};
use common::asserts::{assert_preflight, assert_simple};
use common::builders::{cors, preflight_request, simple_request};
use common::headers::header_value;
use std::sync::Arc;
use std::thread;

#[test]
fn cors_can_be_shared_across_threads() {
    let cors = Arc::new(
        cors()
            .origin(Origin::any())
            .credentials(true)
            .allowed_headers(AllowedHeaders::list(["X-Thread"]))
            .build(),
    );

    let mut handles = Vec::new();
    for i in 0..8 {
        let cors = Arc::clone(&cors);
        handles.push(thread::spawn(move || {
            let origin = format!("https://thread{}.example", i);
            let headers = assert_preflight(
                preflight_request()
                    .origin(origin.as_str())
                    .request_method(method::POST)
                    .request_headers("X-Thread")
                    .check(&cors),
            );

            assert_eq!(
                header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
                Some(origin.as_str()),
            );
            assert_eq!(
                header_value(&headers, header::ACCESS_CONTROL_ALLOW_HEADERS),
                Some("X-Thread"),
            );

            let simple_headers =
                assert_simple(simple_request().origin(origin.as_str()).check(&cors));
            assert_eq!(
                header_value(&simple_headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
                Some(origin.as_str()),
            );
        }));
    }

    for handle in handles {
        handle.join().expect("thread panic");
    }
}
