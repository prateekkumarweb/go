use crate::state::AppState;
use axum::{
    async_trait,
    extract::FromRequestParts,
    headers::{authorization::Bearer, Authorization},
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json, RequestPartsExt, TypedHeader,
};
use axum_extra::extract::SignedCookieJar;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use std::time::UNIX_EPOCH;

pub struct JwtSecret {
    encoding: EncodingKey,
    decoding: DecodingKey,
}

lazy_static::lazy_static! {
    pub static ref JWT_SECRET: JwtSecret = {
        let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
        let secret = secret.as_bytes();
        JwtSecret { encoding: EncodingKey::from_secret(secret), decoding: DecodingKey::from_secret(secret) }
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    sub: String,
    pub username: String,
    exp: usize,
}

#[async_trait]
impl FromRequestParts<AppState> for Claims {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map(|h| h.0 .0.token().to_owned());
        let jar = parts
            .extract_with_state::<SignedCookieJar, AppState>(state)
            .await
            .map_err(|e| {
                tracing::debug!(?e);
                AuthError::InvalidToken
            })?;
        let token = match token {
            Ok(token) => Some(token),
            Err(_) => jar.get("bearer_token").map(|c| c.value().to_string()),
        };

        let token_data = decode::<Claims>(
            &token.ok_or(AuthError::InvalidToken)?,
            &JWT_SECRET.decoding,
            &Validation::default(),
        )
        .map_err(|_| AuthError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

#[derive(Debug, Serialize)]
pub struct AuthBody {
    pub access_token: String,
    token_type: String,
}

impl AuthBody {
    fn new(access_token: String) -> Self {
        Self {
            access_token,
            token_type: "Bearer".to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct AuthPayload {
    username: String,
    password: Secret<String>,
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(serde_json::json!({
            "error": error_message,
        }));
        (status, body).into_response()
    }
}

pub async fn authorize(state: AppState, payload: AuthPayload) -> Result<AuthBody, AuthError> {
    if payload.username.is_empty() || payload.password.expose_secret().is_empty() {
        return Err(AuthError::MissingCredentials);
    }
    let Some(_) = state.links().read().await.auth_user(&payload.username, payload.password) else {
        return Err(AuthError::WrongCredentials);
    };
    let claims = Claims {
        sub: payload.username.to_owned(),
        username: payload.username.to_owned(),
        exp: (std::time::SystemTime::now() + std::time::Duration::from_secs(24 * 60 * 60))
            .duration_since(UNIX_EPOCH)
            .expect("Wrong time! (did you go back in past?)")
            .as_millis() as usize,
    };
    let token = encode(&Header::default(), &claims, &JWT_SECRET.encoding)
        .map_err(|_| AuthError::TokenCreation)?;
    Ok(AuthBody::new(token))
}

pub async fn current_user(claims: Claims) -> impl IntoResponse {
    Json::from(serde_json::json!({
        "message": format!("Welcome {}!", claims.username),
        "username": claims.username
    }))
}
