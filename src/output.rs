use anyhow::{Context, Result};
use std::process::Command;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};
use tabled::Table;
use tabled::settings::object::{Columns, Rows};
use tabled::settings::{Color, Style};

use crate::package::Package;

const NOTIFIER_APP: &str = "macos-notifier";
const NOTIFIER_FALLBACK: &str = "osascript";
const NOTIFICATION_TITLE: &str = "Nix Package Update";

pub struct ProgressConfig<TP, PF>
where
    TP: TracksProgress + Sync,
    PF: Fn(u64) -> TP + Send,
    // Send + Sync needed because this will get passed into Rayon
{
    pub spinner: TP,
    pub progress_bar: PF,
}

// This was done so we can mock ProgressBar
pub trait TracksProgress {
    fn set_message(&self, msg: String);
    fn inc(&self, delta: u64);
    fn finish(&self);
}

// Since we want our mocks to work just like ProgressBar,
// these are all pass-through
impl TracksProgress for ProgressBar {
    fn set_message(&self, msg: String) {
        self.set_message(msg)
    }

    fn inc(&self, delta: u64) {
        self.inc(delta)
    }

    fn finish(&self) {
        self.finish()
    }
}

pub fn notify_updates(updates: &[String]) -> Result<()> {
    let message = format!(
        "Following Packages have updates\n• {}",
        updates.join("\n• ")
    );

    let (output, cmd_name) = if Command::new(NOTIFIER_APP).spawn().is_ok() {
        (
            Command::new(NOTIFIER_APP)
                .arg("--title")
                .arg(NOTIFICATION_TITLE)
                .arg("--content")
                .arg(&message)
                .output()
                .context("failed to run `{NOTIFIER_APP}`")?,
            NOTIFIER_APP,
        )
    } else {
        (
            Command::new(NOTIFIER_FALLBACK)
                .arg("-e")
                .arg(format!(
                    "display notification \"{}\" with title \"{}\"",
                    &message, NOTIFICATION_TITLE
                ))
                .output()
                .context("failed to run `{NOTIFIER_FALLBACK}`")?,
            NOTIFIER_FALLBACK,
        )
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        anyhow::bail!(
            "`notification failed` failed: cmd={} status={}\nmessage={:?}\nstderr:\n{}\nstdout:\n{}",
            cmd_name,
            output.status,
            message,
            stderr,
            stdout
        );
    }

    Ok(())
}

pub fn table(packages: &Vec<Package>) -> Table {
    let mut table = Table::new(packages);

    /* Modern style
     * Make all data in 3rd column (columns are 0 indexed), including header, green
     * Then Re-make all data in 1st row (the header) default color (based on term colors)
     * This makes all of the headers the same color, but allows the
     * data in the 3rd column to be green.
     */
    table
        .with(Style::modern())
        .modify(Columns::new(2..), Color::FG_GREEN)
        .modify(Rows::first(), Color::default());

    table
}

pub fn update_string(update_names: &[String]) -> String {
    if update_names.is_empty() {
        "0 Updated Packages".to_string()
    } else {
        format!(
            "{} Updated Packages - {}",
            update_names.len(),
            update_names.join(", ")
        )
    }
}

pub fn path_process_bar(length: u64) -> ProgressBar {
    let bar = indicatif::ProgressBar::new(length);

    bar.set_style(
        ProgressStyle::with_template("{msg} {bar:40.cyan/blue} ({pos:>2}/{len:>2})")
            .unwrap()
            .progress_chars("##-"),
    );

    bar
}

pub fn query_flake_spinner() -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(120)); //adjust this by feel
    spinner.set_style(ProgressStyle::with_template("{spinner:.green} {msg}").unwrap());
    spinner
}

#[cfg(test)]
mod test {
    use crate::output::update_string;

    #[test]
    fn update_string_formats_correctly_for_updates() {
        let update_names = vec![
            "a-pkg".to_string(),
            "b-pkg".to_string(),
            "c-pkg".to_string(),
        ];

        let update_string = update_string(&update_names);

        assert_eq!(update_string, "3 Updated Packages - a-pkg, b-pkg, c-pkg")
    }

    #[test]
    fn update_string_formats_correctly_for_none() {
        let update_names = vec![];

        let update_string = update_string(&update_names);

        assert_eq!(update_string, "0 Updated Packages")
    }
}
