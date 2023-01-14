use crate::store::Config;
use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use std::{ops::Deref, sync::Arc};
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState(Arc<InnerState>);

impl Deref for AppState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self(Arc::new(InnerState {
            // TODO: This should not be generated each time app starts
            key: Key::generate(),
            links: RwLock::new(config),
        }))
    }

    pub fn links(&self) -> &RwLock<Config> {
        &self.0.links
    }
}

pub struct InnerState {
    key: Key,
    links: RwLock<Config>,
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.0.key.clone()
    }
}
