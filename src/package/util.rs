use anyhow::Result;
use regex::Regex;
use std::process::Output;

pub fn string_from_std(cmd: &Output) -> String {
    String::from_utf8_lossy(&cmd.stdout).into_owned()
}

pub fn expand_if_env_var(input: &str) -> Result<String> {
    let re = Regex::new(r"^\$\{([A-Za-z_][A-Za-z0-9_]*)\}|\$([A-Za-z_][A-Za-z0-9_]*)$")?;

    if let Some(caps) = re.captures(input) {
        let var_name = caps
            .get(1)
            .or_else(|| caps.get(2))
            .map_or("", |m| m.as_str());

        Ok(std::env::var(var_name)?)
    } else {
        Ok(input.to_string())
    }
}

#[cfg(test)]
mod env_var_test {
    use super::expand_if_env_var;

    #[test]
    fn existing_env_var_is_expanded() {
        let home = "$HOME";
        // we're not sure what $HOME is on various systems, but it will exist
        // and shouldn't be the same as the literal "$HOME"
        assert_ne!(home, expand_if_env_var(home).unwrap())
    }

    #[test]
    fn existing_brace_y_env_var_is_expanded() {
        let brace_y_home = "${HOME}";
        assert_ne!(brace_y_home, expand_if_env_var(brace_y_home).unwrap());
    }

    #[test]
    fn missing_env_var_is_err() {
        let not_found = "$UN_FOUND";
        assert!(expand_if_env_var(not_found).is_err())
    }

    #[test]
    fn regular_string_is_passed_through() {
        let other = "/some/other/path";
        let maybe_expanded_other = expand_if_env_var(other);
        assert_eq!(other, maybe_expanded_other.unwrap())
    }
}
