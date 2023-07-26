use std::{path::Path, sync::Arc};

use lazy_static::lazy_static;

use super::Config;

lazy_static! {
    pub static ref CONFIG: Arc<Config> =
        Arc::new(Config::new(Path::new("/sdcard/Android/fas-rs/games.toml")));
}
