use anyhow::Result;

use super::Package;
use crate::config::ConfigPkgs;

pub trait GetsPackages {
    fn path(&self) -> &str;
    fn config_name(&self) -> &str;
    fn paths_from_flake_config(&self) -> Result<Vec<String>>;
    fn option_package_from_path(
        &self,
        path: &str,
        cfg_pkgs: &ConfigPkgs,
    ) -> Result<Option<Package>>;
}

pub trait GetsJSON {
    fn path_generator(&self, path: &str, config_name: &str) -> Result<Vec<String>>;
    fn package_generator(&self, store_path: &str) -> Result<String>;
    fn unstable_version_generator(
        &self,
        package_name: &str,
        current_version: &str,
    ) -> Result<Option<String>>;
}
