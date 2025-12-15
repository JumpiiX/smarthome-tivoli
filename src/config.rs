use std::env;
use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub knx: KnxConfig,
    pub homekit: HomeKitConfig,
}

#[derive(Debug, Clone)]
pub struct KnxConfig {
    pub base_url: String,
    #[allow(dead_code)]
    pub pages: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct HomeKitConfig {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub pin: String,
    pub port: u16,
}

impl Config {
    pub fn load_from_env() -> Result<Self> {
        let base_url = env::var("SMARTHOME_BASE_URL")
            .context("SMARTHOME_BASE_URL not set in .env")?;

        let pages = Vec::new();

        Ok(Config {
            knx: KnxConfig {
                base_url,
                pages,
            },
            homekit: HomeKitConfig {
                name: "Rust KNX Bridge".to_string(),
                pin: "031-45-154".to_string(),
                port: 8080,
            },
        })
    }
}
