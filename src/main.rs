#![forbid(unsafe_code)]

use crate::{
    routes::{
        create_link, delete_link, get_links, goto, home, login, login_post, login_token, logout,
    },
    state::AppState,
    store::Config,
};
use auth::{current_user, Claims};
use axum::{
    middleware::{self},
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use std::{net::SocketAddr, path::PathBuf};
use tera::Tera;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod cli;
mod error;
mod password;
mod routes;
mod state;
mod store;

lazy_static::lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                std::process::exit(1);
            }
        };
        tera.autoescape_on(vec![".html"]);
        tera
    };
}

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
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

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

    let state = AppState::new(links_config);

    let app = Router::new()
        .route("/", get(home))
        .route("/login", get(login))
        .route("/login", post(login_post))
        .route("/login/token", post(login_token))
        .route("/logout", post(logout))
        .nest("/api", api_router(state.clone()))
        .route("/:short", get(goto))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn api_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/link", get(get_links))
        .route("/link", post(create_link))
        .route("/link", delete(delete_link))
        .route("/user", get(current_user))
        .route_layer(middleware::from_extractor_with_state::<Claims, AppState>(
            state,
        ))
}
