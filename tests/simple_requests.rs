mod common;

use bunner_cors_rs::constants::header;
use common::asserts::assert_simple;
use common::builders::{cors, simple_request};
use common::headers::{has_header, header_value};

#[test]
fn default_simple_request_allows_any_origin() {
    let cors = cors().build();
    let headers = assert_simple(
        simple_request()
            .origin("https://example.com")
            .evaluate(&cors),
    );

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
    assert!(!has_header(&headers, header::VARY));
}

#[test]
fn default_simple_request_without_origin_still_allows_any() {
    let cors = cors().build();
    let headers = assert_simple(simple_request().evaluate(&cors));

    assert_eq!(
        header_value(&headers, header::ACCESS_CONTROL_ALLOW_ORIGIN),
        Some("*")
    );
}
