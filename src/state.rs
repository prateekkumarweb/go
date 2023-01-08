use crate::store::Config;
use tokio::sync::Mutex;

pub struct AppState {
    pub links: Mutex<Config>,
}
