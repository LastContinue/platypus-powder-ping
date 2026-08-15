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

// This is for Serde parsing only
fn display_update(update: &Option<String>) -> String {
    update.as_deref().unwrap_or("").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwraps_some() {
        let text = "Optional Update".to_string();
        let update = Some(text.clone());

        assert_eq!(text, display_update(&update))
    }

    #[test]
    fn test_unwraps_none() {
        assert_eq!("".to_string(), display_update(&None))
    }
}
