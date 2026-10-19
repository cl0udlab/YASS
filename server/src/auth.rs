use crate::data::DATA_MANAGER;
use actix_web::dev::Payload;
use actix_web::error::ErrorUnauthorized;
use actix_web::{FromRequest, HttpRequest};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};

const TOKEN_TTL_SECS: i64 = 7 * 24 * 60 * 60;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
}

pub struct AdminAuth;

pub fn issue_token(username: &str) -> Result<(String, i64), jsonwebtoken::errors::Error> {
    let exp = chrono::Utc::now().timestamp() + TOKEN_TTL_SECS;
    let claims = Claims {
        sub: username.to_string(),
        exp,
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(DATA_MANAGER.jwt_secret().as_bytes()),
    )?;
    Ok((token, exp))
}

fn verify(token: &str) -> Option<Claims> {
    decode::<Claims>(
        token,
        &DecodingKey::from_secret(DATA_MANAGER.jwt_secret().as_bytes()),
        &Validation::new(Algorithm::HS256),
    )
    .ok()
    .map(|data| data.claims)
}

fn bearer(req: &HttpRequest) -> Option<String> {
    let raw = req.headers().get("Authorization")?.to_str().ok()?;
    let token = raw.strip_prefix("Bearer ")?.trim();
    (!token.is_empty()).then(|| token.to_string())
}

impl FromRequest for AdminAuth {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match bearer(req).and_then(|t| verify(&t)) {
            Some(_) => ready(Ok(AdminAuth)),
            None => ready(Err(ErrorUnauthorized(r#"{"error":"需要登入"}"#))),
        }
    }
}
