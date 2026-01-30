use std::time::{SystemTime, UNIX_EPOCH};

use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use shared::error::{AppError, AppResult};
use tracing::warn;

use crate::model::{auth::AccessToken, id::UserId};

#[derive(Serialize, Deserialize, Debug)]
struct Claims {
    sub: String,
    iat: i64, // 発行日時（UNIXタイムスタンプ）
    exp: i64, // 有効期限（UNIXタイムスタンプ）
}

#[derive(Debug, PartialEq)]
pub struct VerifiedToken {
    pub sub: String,
}

impl From<Claims> for VerifiedToken {
    fn from(value: Claims) -> Self {
        Self { sub: value.sub }
    }
}

pub struct JwtIssuer {
    secret: String,
    ttl: u64,
}

impl JwtIssuer {
    pub fn new(secret: String, ttl: u64) -> Self {
        Self { secret, ttl }
    }

    pub fn ttl(&self) -> u64 {
        self.ttl
    }

    pub fn issue_token(&self, user_id: UserId) -> AppResult<AccessToken> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("timestamp")
            .as_secs() as i64;
        let claims = Claims {
            sub: user_id.to_string(),
            iat: now,
            exp: now + self.ttl as i64,
        };
        let key = EncodingKey::from_secret(self.secret.as_bytes());
        encode(&Header::default(), &claims, &key)
            .map(AccessToken)
            .map_err(AppError::from)
    }

    pub fn verify_token(&self, token: &AccessToken) -> AppResult<VerifiedToken> {
        let key = DecodingKey::from_secret(self.secret.as_bytes());
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.set_required_spec_claims(&["exp", "sub"]);

        let claims = decode::<Claims>(token.0.as_str(), &key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                warn!(error = %e, "Invalid token");
                AppError::Unauthorized("Invalid token".into())
            })?;

        if claims.sub.is_empty() {
            warn!("Invalid token: sub is empty");
            return Err(AppError::Unauthorized("Invalid token".into()));
        }

        Ok(VerifiedToken::from(claims))
    }
}

#[cfg(test)]
mod tests {
    use super::{JwtIssuer, VerifiedToken};
    use crate::model::{auth::AccessToken, id::UserId};

    #[test]
    fn jwt生成と検証でsubとexpが正しい() {
        let user_id = UserId::new();
        let secret = "test-secret";
        let ttl = 60_u64 * 60;
        let expected = VerifiedToken {
            sub: user_id.to_string(),
        };

        let issuer = JwtIssuer::new(secret.to_string(), ttl);
        let token: AccessToken = issuer.issue_token(user_id).expect("jwt生成");
        let claims: VerifiedToken = issuer.verify_token(&token).expect("jwt検証");
        assert_eq!(claims, expected);
    }

    #[test]
    fn jwtは不正な署名で検証に失敗する() {
        let user_id = UserId::new();
        let ttl = 60_u64 * 60;
        let issuer = JwtIssuer::new("correct-secret".to_string(), ttl);
        let token: AccessToken = issuer.issue_token(user_id).expect("jwt生成");

        let wrong_issuer = JwtIssuer::new("wrong-secret".to_string(), ttl);
        let err = wrong_issuer
            .verify_token(&token)
            .expect_err("不正署名は失敗する");
        let _ = err;
    }
}
