use starberry_oauth::oauth_core::types::OAuthError;
use starberry_core::http::response::HttpResponse;
use starberry_core::http::body::HttpBody;
use starberry_core::http::start_line::HttpStartLine;
use starberry_core::http::start_line::ResponseStartLine;
use starberry_core::http::http_value::StatusCode;
use starberry_core::http::http_value::HttpVersion;
use starberry_core::http::http_value::HttpContentType;
use serde_json::Value;
use std::str;

#[tokio::test]
async fn test_oauth_error_into_response() {
    let cases = vec![
        (OAuthError::InvalidClient, StatusCode::UNAUTHORIZED, "invalid_client", "Client authentication failed"),
        (OAuthError::InvalidGrant, StatusCode::BAD_REQUEST, "invalid_grant", "Invalid grant provided"),
        (OAuthError::InvalidToken, StatusCode::UNAUTHORIZED, "invalid_token", "The token is invalid"),
        (OAuthError::TokenExpired, StatusCode::UNAUTHORIZED, "invalid_token", "The token has expired"),
        (OAuthError::InsufficientScopes, StatusCode::FORBIDDEN, "insufficient_scope", "Insufficient scopes for this request"),
        (OAuthError::CsrfMismatch, StatusCode::BAD_REQUEST, "invalid_request", "CSRF token mismatch"),
        (OAuthError::RateLimited, StatusCode::TOO_MANY_REQUESTS, "rate_limited", "Too many requests"),
        (OAuthError::HttpError("oops".into()), StatusCode::BAD_GATEWAY, "http_error", "oops"),
        (OAuthError::Unauthorized, StatusCode::FORBIDDEN, "unauthorized_client", "Client not authorized"),
        (OAuthError::ServerError, StatusCode::INTERNAL_SERVER_ERROR, "server_error", "Internal server error"),
    ];

    for (err, expected_status, expected_code, expected_desc) in cases {
        let mut resp: HttpResponse = err.into_response();
        // Check status line
        let start_line = resp.meta.start_line.clone();
        match start_line {
            HttpStartLine::Response(ResponseStartLine { http_version, status_code }) => {
                assert_eq!(http_version.to_string(), "HTTP/1.1");
                assert_eq!(status_code, expected_status, "Status for {:?}", err);
            }
            _ => panic!("Expected response start line"),
        }
        // Check content type header
        assert_eq!(resp.meta.get_content_type().unwrap(), HttpContentType::ApplicationJson());
        // Check JSON body
        match resp.body {
            HttpBody::Binary(ref bytes) => {
                let s = str::from_utf8(bytes).unwrap();
                let v: Value = serde_json::from_str(s).unwrap();
                assert_eq!(v["error"], expected_code, "Error code for {:?}", err);
                assert_eq!(v["error_description"], expected_desc, "Description for {:?}", err);
            }
            _ => panic!("Expected binary JSON body"),
        }
    }
} 