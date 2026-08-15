use std::process::Command;

use crate::config::ConfigPkgs;

use super::{GetsJSON, GetsPackages, Package, expand_if_env_var, string_from_std};
use anyhow::{Context, Result};

pub struct Darwin<G>
where
    G: GetsJSON + Send + Sync,
{
    pub path: String,
    pub config_name: String,
    pub json_gen: G,
}

impl<G> Darwin<G>
where
    G: GetsJSON + Send + Sync,
{
    pub fn new(flake_dir: String, flake_config_name: Option<String>, json_gen: G) -> Result<Self> {
        let path = expand_if_env_var(flake_dir.as_str())?;

        let config_name = match flake_config_name {
            Some(n) => expand_if_env_var(n.as_str())?,
            // User maybe using "config as hostname -s" trick to keep from having to specify the config name
            None => {
                let hostname = Command::new("hostname")
                    .arg("-s")
                    .output()
                    .context("`hostname -s` encountered an issue")?;
                string_from_std(&hostname).trim().to_string()
            }
        };

        Ok(Darwin {
            path,
            config_name,
            json_gen,
        })
    }
}

impl<G> GetsPackages for Darwin<G>
where
    G: GetsJSON + Send + Sync,
{
    fn path(&self) -> &str {
        &self.path
    }
    fn config_name(&self) -> &str {
        &self.config_name
    }

    fn paths_from_flake_config(&self) -> Result<Vec<String>> {
        self.json_gen.path_generator(&self.path, &self.config_name)
    }

    fn option_package_from_path(
        &self,
        path: &str,
        cfg_pkgs: &ConfigPkgs,
    ) -> Result<Option<Package>> {
        let package_json = self.json_gen.package_generator(path)?;
        let mut package: Package = serde_json::from_str(&package_json).with_context(|| {
            format!("package details JSON was not well-formatted \n JSON: {package_json}",)
        })?;

        let is_ignored = cfg_pkgs.ignore.contains(&package.name);

        // I noticed that there are a few packages without versions
        // that seem to be installed with Darwin. I think we can just ignore them
        // if the main use-case is "show me what has updates"
        let is_unversioned = package.current_version.is_empty();

        // If it's ignored, or unversioned, go no further
        if is_ignored || is_unversioned {
            Ok(None)
        } else {
            let lookup_name: String = cfg_pkgs
                .overrides
                .get(&package.name)
                .cloned()
                .unwrap_or_else(|| package.name.clone());

            // I believe getting this info in the same Nix expression as package_details() is
            // possible, but it's beyond my Nix-Understanding at this point in time
            package.update = self
                .json_gen
                .unstable_version_generator(&lookup_name, &package.current_version)?;

            //Add lookup name if there is any difference so output will make more sense
            if package.name != lookup_name {
                package.name.push_str(&format!(" ({lookup_name})"));
            }

            Ok(Some(package))
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use indicatif::ProgressBar;

    use crate::{ProgressConfig, config::ConfigPkgs, run};

    use super::*;

    struct MockEval<F>
    where
        F: Fn(&str) -> String + Send + Sync,
    {
        //this is the only thing that changes between tests.
        package_logic: F,
    }

    impl<F: Fn(&str) -> String + Send + Sync> GetsJSON for MockEval<F> {
        fn path_generator(&self, _path: &str, _config_name: &str) -> Result<Vec<String>> {
            let paths: Vec<String> = vec!["path_a".into(), "path_b".into()];
            Ok(paths)
        }
        fn package_generator(&self, store_path: &str) -> Result<String> {
            let logic = &self.package_logic;
            Ok(logic(store_path))
        }

        fn unstable_version_generator(
            &self,
            _package_name: &str,
            _current_version: &str,
        ) -> Result<Option<String>> {
            Ok(None)
        }
    }

    fn get_fake_output_config() -> ProgressConfig {
        ProgressConfig {
            spinner: ProgressBar::hidden(),
            progress_bar: Box::new(|_len| ProgressBar::hidden()),
        }
    }

    fn get_test_darwin<G>(json_gen: G) -> Result<Darwin<G>>
    where
        G: GetsJSON + Send + Sync,
    {
        let darwin = Darwin::new(
            "/some/dir".to_string(),
            Some("config name".to_string()),
            json_gen,
        )?;

        Ok(darwin)
    }

    #[test]
    fn option_package_from_path_filters_ignored() -> Result<()> {
        let eval = MockEval {
            package_logic: |store_path| {
                let pkg;
                if store_path == "path_a" {
                    pkg = "{\"current_version\":\"0.3.42\",\"name\":\"codebook\"}\n".to_string();
                } else {
                    pkg = "{\"current_version\":\"15.2.0\",\"name\":\"ripgrep\"}\n".to_string();
                }
                pkg
            },
        };

        let flake = get_test_darwin(eval)?;

        let pkgs = ConfigPkgs {
            ignore: vec!["codebook".into()],
            overrides: HashMap::new(),
        };

        let rs = run(&flake, pkgs, get_fake_output_config())?;

        let codebook = rs.packages.iter().find(|pkg| pkg.name == "codebook");
        assert!(codebook.is_none());

        Ok(())
    }

    #[test]
    fn option_package_from_path_overrides() -> Result<()> {
        let eval = MockEval {
            package_logic: |store_path| {
                let pkg;
                if store_path == "path_a" {
                    pkg = "{\"current_version\":\"0.3.42\",\"name\":\"codebook\"}\n".to_string();
                } else {
                    pkg = "{\"current_version\":\"15.2.0\",\"name\":\"ripgrep\"}\n".to_string();
                }
                pkg
            },
        };

        let pkgs = ConfigPkgs {
            ignore: vec![],
            overrides: HashMap::from([("ripgrep".into(), "rg".into())]),
        };

        let flake = get_test_darwin(eval)?;

        let rs = run(&flake, pkgs, get_fake_output_config())?;

        let ripgrep = rs.packages.iter().find(|pkg| pkg.name == "ripgrep (rg)");

        assert!(ripgrep.is_some());

        Ok(())
    }

    #[test]
    fn option_package_from_path_ignore_unversioned() -> Result<()> {
        let eval = MockEval {
            package_logic: |store_path| {
                let pkg;
                if store_path == "path_a" {
                    pkg = "{\"current_version\":\"\",\"name\":\"codebook\"}\n".to_string();
                } else {
                    pkg = "{\"current_version\":\"15.2.0\",\"name\":\"ripgrep\"}\n".to_string();
                }
                pkg
            },
        };

        let pkgs = ConfigPkgs {
            ignore: vec![],
            overrides: HashMap::new(),
        };

        let flake = get_test_darwin(eval)?;

        let rs = run(&flake, pkgs, get_fake_output_config())?;

        let codebook = rs.packages.iter().find(|pkg| pkg.name == "codebook");
        let ripgrep = rs.packages.iter().find(|pkg| pkg.name == "ripgrep");

        assert!(ripgrep.is_some());
        assert!(codebook.is_none());

        Ok(())
    }
}
