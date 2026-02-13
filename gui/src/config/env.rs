//! Environment variables module.

use std::sync::OnceLock;

static ENV: OnceLock<Env> = OnceLock::new();

/// Cached environment variables.
pub struct Env {
    pub user: String,
    pub home: String,
}

impl Env {
    fn new() -> anyhow::Result<Self> {
        Ok(Self {
            user: std::env::var("USER")
                .map_err(|_| anyhow::anyhow!("USER environment variable is not set"))?,
            home: std::env::var("HOME")
                .map_err(|_| anyhow::anyhow!("HOME environment variable is not set"))?,
        })
    }
}

/// Initialize environment variables. Must be called at application startup.
pub fn init() -> anyhow::Result<()> {
    ENV.set(Env::new()?)
        .map_err(|_| anyhow::anyhow!("Environment variables already initialized"))?;
    Ok(())
}

/// Panics if not initialized (call `init()` at application startup).
pub fn get() -> &'static Env {
    ENV.get()
        .expect("Environment variables not initialized. Call config::env::init() at startup.")
}
