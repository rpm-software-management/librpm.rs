use std::{
    path::{Path, PathBuf},
    sync::Once,
};

use librpm::config;

static CONFIGURE: Once = Once::new();

// Read the default config
pub fn configure() {
    CONFIGURE.call_once(|| {
        config::read_file(None).unwrap();
    });
}

pub fn get_assets_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata")
}
