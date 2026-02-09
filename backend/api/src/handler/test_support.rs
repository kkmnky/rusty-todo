use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
use kernel::model::id::UserId;
use kernel::service::jwt::JwtIssuer;

pub fn build_auth_header(token: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let value = HeaderValue::from_str(&format!("Bearer {}", token)).expect("header生成");
    headers.insert(AUTHORIZATION, value);
    headers
}

pub fn build_valid_auth_header(issuer: &JwtIssuer) -> HeaderMap {
    let token = issuer.issue_token(UserId::new()).expect("jwt生成");
    build_auth_header(&token.0)
}

pub fn build_auth_header_for_user(issuer: &JwtIssuer, user_id: UserId) -> HeaderMap {
    let token = issuer.issue_token(user_id).expect("jwt生成");
    build_auth_header(&token.0)
}
