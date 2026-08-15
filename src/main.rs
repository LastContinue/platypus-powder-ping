use anyhow::{Context, Result};
use clap::Parser;
use indicatif::ParallelProgressIterator;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{process::Command, time::Instant};
mod package;
use package::{Darwin, GetsPackages, NixEval, Package, ProvidesHostname, string_from_std};
mod config;
use config::{Config, ConfigPkgs, load_config, resolve_config_path};
mod output;
use output::{
    ProgressConfig, notify_updates, path_process_bar, query_flake_spinner, table, update_string,
};

#[derive(Parser, Debug)]
#[command(
    name = "Platypus Powder Ping",
    about = "Lists packages in Darwin Flake"
)]
struct Args {
    #[arg(short = 'c', long = "config", value_name = "CONFIG_PATH")]
    config: Option<String>,
    #[arg(short = 'n', long = "notify")]
    notify: bool,
    #[arg(short = 'b', long = "benchmark")]
    benchmark: bool,
}

struct RunSummary {
    pub packages: Vec<Package>,
    pub update_names: Vec<String>,
}

struct MacOsHostnameCmd;

impl ProvidesHostname for MacOsHostnameCmd {
    fn get_hostname(&self) -> Result<String> {
        let hostname = Command::new("hostname")
            .arg("-s")
            .output()
            .context("`hostname -s` encountered an issue")?;
        Ok(string_from_std(&hostname))
    }
}

fn main() -> Result<()> {
    let now = Instant::now();
    let args = Args::parse();
    let home = std::env::var("HOME")?;

    let config_path = resolve_config_path(&args, &home);
    let cfg: Config = load_config(&config_path)?;

    let flake = Darwin::new(
        cfg.flake_dir,
        cfg.flake_config_name,
        NixEval {},
        &MacOsHostnameCmd,
    )?;

    let progress_config = ProgressConfig {
        spinner: query_flake_spinner(),
        progress_bar: Box::new(|len| path_process_bar(len)),
    };

    let rs = run(&flake, cfg.pkgs, progress_config)?;

    println!("{}", table(&rs.packages));
    println!("{}", update_string(&rs.update_names));

    maybe_notify(args.notify, &rs.update_names, notify_updates)?;
    maybe_benchmark(args.benchmark, || {
        println!("Elapsed: {:.2?}", now.elapsed())
    });

    Ok(())
}

fn run<G>(flake: &G, pkg_details: ConfigPkgs, output_config: ProgressConfig) -> Result<RunSummary>
where
    G: GetsPackages + Sync,
{
    let spinner_msg = format!(
        "Querying Darwin Flake\n  • location:'{}'\n  • config:  '{}'",
        flake.path(),
        flake.config_name()
    );

    output_config.spinner.set_message(spinner_msg);

    let paths = flake.paths_from_flake_config()?;
    output_config.spinner.finish();

    let progress_bar = (output_config.progress_bar)(paths.len() as u64);

    progress_bar.set_message("Processing Paths".to_string());

    let options: Vec<Option<Package>> = paths
        .par_iter()
        .progress_with(progress_bar)
        .map(|path| flake.option_package_from_path(path, &pkg_details))
        .collect::<Result<Vec<Option<Package>>>>()?;

    let mut packages: Vec<Package> = options.into_iter().flatten().collect();
    packages.sort();

    let update_names: Vec<String> = packages
        .iter()
        .filter(|p| p.update.is_some())
        .map(|p| p.name.clone())
        .collect();

    Ok(RunSummary {
        packages,
        update_names,
    })
}

fn maybe_benchmark<F>(benchmark: bool, f: F)
where
    F: FnOnce(),
{
    if benchmark {
        f();
    }
}

fn maybe_notify<F>(notify: bool, updates: &[String], notify_fn: F) -> Result<()>
where
    F: FnOnce(&[String]) -> Result<()>,
{
    if notify && !updates.is_empty() {
        notify_fn(updates)?;
    }

    Ok(())
}

#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn parses_defaults_when_no_flags() {
        let args = Args::try_parse_from(["prog"]).unwrap();
        assert!(args.config.is_none());
        assert!(!args.notify);
        assert!(!args.benchmark);
    }

    #[test]
    fn parses_flags() {
        let args = Args::try_parse_from(["prog", "-n", "-b"]).unwrap();
        assert!(args.notify);
        assert!(args.benchmark);
    }

    #[test]
    fn runs_when_benchmark_true() {
        let mut count = 0;

        maybe_benchmark(true, || {
            count += 1;
        });

        assert_eq!(count, 1);

        let mut count2 = 0;
        maybe_benchmark(false, || {
            count2 += 1;
        });
        assert_eq!(count2, 0);
    }
}

#[cfg(test)]
mod run_tests {
    use indicatif::ProgressBar;

    use super::*;
    use std::collections::HashMap;

    struct FakeFlake {
        paths: Vec<String>,
        results: HashMap<String, Result<Option<Package>>>,
    }

    impl GetsPackages for FakeFlake {
        fn path(&self) -> &str {
            "test_path"
        }
        fn config_name(&self) -> &str {
            "test_config_name"
        }

        fn paths_from_flake_config(&self) -> Result<Vec<String>> {
            Ok(self.paths.clone())
        }

        fn option_package_from_path(
            &self,
            path: &str,
            _cfg_pkgs: &ConfigPkgs,
        ) -> Result<Option<Package>> {
            match self.results.get(path) {
                Some(Ok(pkg_opt)) => Ok(pkg_opt.clone()),
                Some(Err(_e)) => Err(anyhow::anyhow!("fake flake error for path: {path}")),
                _ => Ok(None),
            }
        }
    }

    fn setup() -> (FakeFlake, ConfigPkgs, Vec<Package>) {
        let path_a = "/a/good/path".to_string();
        let path_b = "/another/good/path".to_string();
        let path_c = "/third/path/why/not".to_string();

        let paths = vec![path_a.clone(), path_b.clone(), path_c.clone()];

        let pkg_a = Package {
            name: "GoodPkg".to_string(),
            current_version: "1.0.0".to_string(),
            update: None,
        };

        let pkg_b = Package {
            name: "FunPkg".to_string(),
            current_version: "1.1.1".to_string(),
            update: None,
        };

        let pkg_c = Package {
            name: "WeirdPkg".to_string(),
            current_version: "some_day_string".to_string(),
            update: Some("animalName".to_string()),
        };

        let pkgs = vec![pkg_a, pkg_b, pkg_c];

        let results: HashMap<String, Result<Option<Package>>> = HashMap::from([
            (paths[0].clone(), Ok(Some(pkgs[0].clone()))),
            (paths[1].clone(), Ok(Some(pkgs[1].clone()))),
            (paths[2].clone(), Ok(Some(pkgs[2].clone()))),
        ]);

        let flake = FakeFlake { paths, results };

        // Testing "ignore" and "overrides" will be done with to other flake tests
        // because it requires mocking/testing deeper functionality beyond what
        // a FlakeLike provides
        let fake_details = ConfigPkgs {
            ignore: vec![],
            overrides: HashMap::new(),
        };

        (flake, fake_details, pkgs)
    }

    #[test]
    fn run_returns_packages() -> Result<()> {
        let (flake, fake_details, mut pkgs) = setup();

        //If you ever mess with Indicatif, ::hidden will save you a bunch of generic code for mocking
        let output_config = ProgressConfig {
            spinner: ProgressBar::hidden(),
            progress_bar: Box::new(|_len| ProgressBar::hidden()),
        };

        let rs = run(&flake, fake_details, output_config)?;

        // "run" sorts the packages by name, so b,a,c is the order for
        // this instance
        pkgs.sort();

        assert_eq!(rs.packages, pkgs);
        assert_eq!(rs.update_names, vec!["WeirdPkg".to_string()]);

        Ok(())
    }
}

#[cfg(test)]
mod notification_tests {
    use super::*;

    #[test]
    fn calls_notifications_when_notify_true() -> Result<()> {
        let mut counter = 0;

        let update_names: [String; 3] = ["PkgA".into(), "PkgB".into(), "PkgC".into()];

        let x = |_updates: &[String]| -> Result<()> {
            counter += 1;
            Ok(())
        };

        let notify_result = maybe_notify(true, &update_names, x)?;

        assert_eq!(counter, 1);

        Ok(notify_result)
    }

    #[test]
    fn run_skips_notifications_when_notify_false() -> Result<()> {
        let mut counter = 0;

        let update_names: [String; 3] = ["PkgA".into(), "PkgB".into(), "PkgC".into()];

        let x = |_updates: &[String]| -> Result<()> {
            counter += 1;
            Ok(())
        };

        let notify_result = maybe_notify(false, &update_names, x)?;

        assert_eq!(counter, 0);

        Ok(notify_result)
    }

    #[test]
    fn run_does_not_call_notifications_when_no_updates() -> Result<()> {
        let mut counter = 0;

        let x = |_updates: &[String]| -> Result<()> {
            counter += 1;
            Ok(())
        };

        let notify_result = maybe_notify(true, &[], x)?;

        assert_eq!(counter, 0);

        Ok(notify_result)
    }
}
