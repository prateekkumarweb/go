use crate::{
    auth::{authorize, AuthBody, AuthError, AuthPayload, Claims},
    error::AppError,
    state::AppState,
    store::Link,
    TEMPLATES,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    SignedCookieJar,
};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

pub async fn home(claims: Option<Claims>) -> Result<impl IntoResponse, AppError> {
    match claims {
        Some(claims) => {
            let mut context = Context::new();
            context.insert("username", &claims.username);
            let html = Tera::render(&TEMPLATES, "index.html", &context)?;
            Ok(Html::from(html).into_response())
        }
        None => Ok(Redirect::to("/login").into_response()),
    }
}

pub async fn login(claims: Option<Claims>) -> Result<impl IntoResponse, AppError> {
    if claims.is_some() {
        return Ok(Redirect::to("/").into_response());
    }
    let context = Context::new();
    let html = Tera::render(&TEMPLATES, "login.html", &context)?;
    Ok(Html::from(html).into_response())
}

#[derive(Debug, Serialize)]
pub struct LoginStatus {
    success: bool,
}

#[axum_macros::debug_handler]
pub async fn login_post(
    State(state): State<AppState>,
    jar: SignedCookieJar,
    Json(payload): Json<AuthPayload>,
) -> Result<(SignedCookieJar, Json<LoginStatus>), AuthError> {
    let token = authorize(state, payload).await;
    token.map(|token| {
        let mut cookie = Cookie::new("bearer_token", token.access_token);
        cookie.set_http_only(true);
        cookie.set_same_site(SameSite::Strict);
        cookie.set_secure(true);
        (jar.add(cookie), Json(LoginStatus { success: true }))
    })
}

#[axum_macros::debug_handler]
pub async fn login_token(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthBody>, AuthError> {
    authorize(state, payload).await.map(Json::from)
}

pub async fn logout(jar: SignedCookieJar) -> Result<impl IntoResponse, AppError> {
    Ok(jar.remove(Cookie::named("bearer_token")))
}

pub async fn goto(State(state): State<AppState>, Path(short): Path<String>) -> impl IntoResponse {
    let links = state.links().read().await;
    if let Some(url) = links.get_link(&short) {
        Redirect::temporary(url).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn get_links(State(state): State<AppState>) -> impl IntoResponse {
    let links = state.links().read().await;

    let all_links = links
        .links_iter()
        .map(|x| Link {
            short: x.0.into(),
            url: x.1.into(),
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(all_links)).into_response()
}

pub async fn create_link(
    State(state): State<AppState>,
    Json(payload): Json<Link>,
) -> impl IntoResponse {
    let mut links = state.links().write().await;

    if let Err(e) = links.add_link(payload.clone()).await {
        tracing::debug!("Failed to add link: {:?} {:?}", payload, e);
        return StatusCode::BAD_GATEWAY.into_response();
    }

    (StatusCode::CREATED, Json(payload)).into_response()
}

#[derive(Debug, Deserialize)]
pub struct DeletePayload {
    short: String,
}

pub async fn delete_link(
    State(state): State<AppState>,
    Json(payload): Json<DeletePayload>,
) -> impl IntoResponse {
    let mut links = state.links().write().await;

    if let Err(e) = links.remove_link(&payload.short).await {
        tracing::debug!("Failed to delete link: {:?} {:?}", payload, e);
        return (StatusCode::BAD_GATEWAY, format!("{:?}", e)).into_response();
    }

    StatusCode::OK.into_response()
}
