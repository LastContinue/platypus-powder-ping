use super::{GetsJSON, string_from_std};
use anyhow::{Context, Result};
use std::process::{Command, Output};

pub struct NixEval {}

impl NixEval {
    fn eval(&self, extra_args: &[&str]) -> Result<Output> {
        let output = Command::new("nix")
            .arg("eval")
            .args(extra_args)
            .output()
            .context("failed to run `nix eval`")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            anyhow::bail!(
                "`nix eval` failed: status={}\nextra_args={:?}\nstderr:\n{}\nstdout:\n{}",
                output.status,
                extra_args,
                stderr,
                stdout
            );
        }

        Ok(output)
    }
}

impl GetsJSON for NixEval {
    fn path_generator(&self, path: &str, config_name: &str) -> Result<Vec<String>> {
        let nix_expr = format!(
            "{}#darwinConfigurations.\"{}\".config.environment.systemPackages",
            path, config_name
        );

        let flake_query = self.eval(&["--json", nix_expr.as_str()])?;
        let input = string_from_std(&flake_query);
        let paths: Vec<String> = serde_json::from_str(&input).context(format!(
            "flake query JSON was not well-formatted.\nJSON input={input}\nnix eval={nix_expr}"
        ))?;

        Ok(paths)
    }

    fn package_generator(&self, store_path: &str) -> Result<String> {
        let nix_expr = format!(
            r#"
            let
                sp = {store_path};
                base = builtins.baseNameOf sp;

                rest = (let m = builtins.match "^[^-]+-(.*)$" base;
                        in if m == null then base else builtins.elemAt m 0);

                parsed = builtins.parseDrvName rest;

                name =
                if parsed ? name
                then parsed.name
                else rest;

                current_version =
                if parsed ? version && parsed.version != null
                then parsed.version
                else "unknown";
            in
            {{
                name = name;
                current_version = current_version;
            }}
            "#
        );

        let pkg_query = self.eval(&["--json", "--expr", nix_expr.as_str()])?;

        Ok(string_from_std(&pkg_query))
    }

    /**
     * Checks nixpkgs-unstable for package version
     */
    fn unstable_version_generator(
        &self,
        package_name: &str,
        current_version: &str,
    ) -> Result<Option<String>> {
        let nix_expr = format!("github:NixOS/nixpkgs/nixpkgs-unstable#{package_name}.version");
        let pkg_query = self.eval(&["--raw", nix_expr.as_str()])?;
        let unstable_version = string_from_std(&pkg_query);
        let unstable_upgrade = (unstable_version != current_version).then_some(unstable_version);

        Ok(unstable_upgrade)
    }
}
