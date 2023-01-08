use crate::{error::AppError, state::AppState, store::Link};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use tokio::fs::read_to_string;

#[axum_macros::debug_handler]
pub async fn home() -> Result<impl IntoResponse, AppError> {
    let html = read_to_string("client/home.html").await?;
    Ok(Html::from(html))
}

pub async fn goto(
    State(state): State<Arc<AppState>>,
    Path(short): Path<String>,
) -> impl IntoResponse {
    let links = state.links.lock().await;
    if let Some(url) = links.get_link(&short) {
        Redirect::temporary(url).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

pub async fn get_links(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let links = state.links.lock().await;

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
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Link>,
) -> impl IntoResponse {
    let mut links = state.links.lock().await;

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
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DeletePayload>,
) -> impl IntoResponse {
    let mut links = state.links.lock().await;

    if let Err(e) = links.remove_link(&payload.short).await {
        tracing::debug!("Failed to delete link: {:?} {:?}", payload, e);
        return (StatusCode::BAD_GATEWAY, format!("{:?}", e)).into_response();
    }

    StatusCode::OK.into_response()
}
