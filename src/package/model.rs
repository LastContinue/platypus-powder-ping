use serde::{Deserialize, Serialize};
use tabled::Tabled;

#[derive(Serialize, Deserialize, Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Tabled)]
#[tabled(rename_all = "Upper Title Case")]
pub struct Package {
    pub name: String,
    pub current_version: String,
    #[serde(skip)]
    #[tabled(display = "display_update")]
    pub update: Option<String>,
}

fn display_update(update: &Option<String>) -> String {
    update.as_deref().unwrap_or("").to_string()
}
