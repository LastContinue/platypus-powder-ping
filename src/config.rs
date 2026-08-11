use crate::Args;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub flake_dir: String,                 // Must be set
    pub flake_config_name: Option<String>, // Option flow works well here
    #[serde(default)]
    pub pkgs: ConfigPkgs,
}

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default)] //Setting serde defaults here works better for because it turns into an Option chain for everything
pub struct ConfigPkgs {
    pub ignore: Vec<String>,
    pub overrides: HashMap<String, String>,
}

const FALLBACK_CONFIG_PATH: &str = "~/.config/platypus-powder-ping/config.toml";

pub fn load_config(config_path: &str) -> Result<Config> {
    let read_string = fs::read_to_string(config_path)
        .with_context(|| format!("failed to read config file: {}", config_path))?;

    let cfg = toml::from_str(&read_string)
        .with_context(|| format!("failed to parse TOML in {}", config_path))?;

    Ok(cfg)
}

pub fn resolve_config_path(args: &Args, home: &str) -> String {
    let raw = match args.config.as_deref() {
        Some(path) => path,
        None => FALLBACK_CONFIG_PATH,
    };
    expand_tilde_with_home(raw, home)
}

fn expand_tilde_with_home(input: &str, home: &str) -> String {
    if let Some(rest) = input.strip_prefix("~/") {
        format!("{}/{}", home, rest)
    } else if input == "~" {
        home.to_string()
    } else {
        input.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Args;
    use clap::Parser;

    #[test]
    fn resolves_fallback_config_when_config_absent() {
        // expand_tilde depends on HOME; set it for a deterministic test
        let args = Args::try_parse_from(["prog"]).unwrap();
        let resolved = resolve_config_path(&args, "/home/testuser");

        assert_eq!(
            resolved,
            "/home/testuser/.config/platypus-powder-ping/config.toml"
        );
    }

    #[test]
    fn resolves_custom_config_when_config_provided() {
        let args = Args::try_parse_from(["prog", "--config", "~/.config/x.toml"]).unwrap();
        let resolved = resolve_config_path(&args, "/home/testuser");

        assert_eq!(resolved, "/home/testuser/.config/x.toml");
    }
}
