#![forbid(unsafe_code)]

use crate::{
    routes::{create_link, delete_link, get_links, goto, home},
    state::AppState,
    store::{Config, Credentials},
};
use axum::{
    extract::State,
    headers::{self, HeaderMapExt},
    http::{Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use secrecy::Secret;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::sync::Mutex;

mod cli;
mod error;
mod password;
mod routes;
mod state;
mod store;

#[derive(Debug, Parser)]
struct Args {
    #[arg(long, default_value = "data/config.yaml")]
    config: PathBuf,
    #[command(subcommand)]
    command: Option<Subcommand>,
}

#[derive(Debug, clap::Subcommand)]
enum Subcommand {
    /// Initialize config with username and password
    Init,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    tracing::debug!(?args);

    if let Some(command) = args.command {
        match command {
            Subcommand::Init => {
                cli::init().await?;
            }
        }
        return Ok(());
    }

    let links_config = Config::new(args.config).await?;
    let credentials = links_config.auth();

    let shared_state = Arc::new(AppState {
        links: Mutex::new(links_config),
    });

    let app = Router::new()
        .route("/", get(home))
        .nest("/api", api_router(credentials))
        .route("/:short", get(goto))
        .with_state(shared_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn api_router(credentials: Credentials) -> Router<Arc<AppState>> {
    tracing::debug!(?credentials);
    Router::new()
        .route("/link", get(get_links))
        .route("/link", post(create_link))
        .route("/link", delete(delete_link))
        .route_layer(middleware::from_fn_with_state(
            Arc::new(credentials),
            auth_middleware,
        ))
}

async fn auth_middleware<B>(
    State(credentials): State<Arc<Credentials>>,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .typed_get::<headers::Authorization<headers::authorization::Basic>>();

    let user = if let Some(basic) = auth_header {
        credentials.auth(basic.username(), Secret::new(basic.password().into()))
    } else {
        None
    };

    tracing::debug!(?user);

    match user {
        Some(_) => Ok(next.run(req).await),
        None => Err(StatusCode::UNAUTHORIZED),
    }
}
