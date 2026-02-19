use axum::http::{HeaderMap, HeaderValue, header::AUTHORIZATION};
use kernel::model::id::UserId;
use kernel::service::jwt::JwtIssuer;
#[cfg(test)]
use registry::MockAppRegistryExt;
use std::sync::Arc;

#[cfg(test)]
pub fn build_registry_with_jwt() -> (MockAppRegistryExt, Arc<JwtIssuer>) {
    let jwt_issuer = Arc::new(JwtIssuer::new("test-secret".to_string(), 60_u64 * 60 * 24));
    let mut registry = MockAppRegistryExt::new();
    registry
        .expect_jwt_issuer()
        .return_const(jwt_issuer.clone());
    (registry, jwt_issuer)
}

#[cfg(test)]
pub fn build_registry_with_valid_auth() -> (MockAppRegistryExt, HeaderMap) {
    let (registry, jwt_issuer) = build_registry_with_jwt();
    let headers = build_valid_auth_header(&jwt_issuer);
    (registry, headers)
}

#[cfg(test)]
pub fn build_registry_with_auth_for_user(
    user_id: UserId,
) -> (MockAppRegistryExt, HeaderMap) {
    let (registry, jwt_issuer) = build_registry_with_jwt();
    let headers = build_auth_header_for_user(&jwt_issuer, user_id);
    (registry, headers)
}

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
