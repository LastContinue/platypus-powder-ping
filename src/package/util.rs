use std::process::Output;

pub fn string_from_std(cmd: &Output) -> String {
    String::from_utf8_lossy(&cmd.stdout).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Output;

    #[test]
    fn test_valid_utf8() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"testing".to_vec(),
            stderr: vec![],
        };
        assert_eq!(string_from_std(&output), "testing");
    }

    #[test]
    fn test_invalid_utf8_replacement() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![0xFF, 0xFE],
            stderr: vec![],
        };
        let result = string_from_std(&output);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_empty_stdout() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: vec![],
            stderr: vec![],
        };
        assert_eq!(string_from_std(&output), "");
    }

    #[test]
    fn test_multiline_output() {
        let output = Output {
            status: std::process::ExitStatus::default(),
            stdout: b"Line 1\nLine 2\nLine 3".to_vec(),
            stderr: vec![],
        };
        assert_eq!(string_from_std(&output), "Line 1\nLine 2\nLine 3");
    }
}
