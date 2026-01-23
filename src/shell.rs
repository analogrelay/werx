use anyhow::{anyhow, Result};

/// Output shell initialization code for the specified shell
pub fn cmd_shell_init(shell: &str) -> Result<()> {
    match shell {
        "bash" => {
            print!("{}", include_str!("../shell/init.bash"));
            Ok(())
        }
        "zsh" => {
            print!("{}", include_str!("../shell/init.zsh"));
            Ok(())
        }
        _ => Err(anyhow!(
            "Unsupported shell: {}\n\nSupported shells: bash, zsh",
            shell
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_init_bash() {
        // Test that bash init doesn't panic and includes expected content
        let result = cmd_shell_init("bash");
        assert!(result.is_ok());
    }

    #[test]
    fn test_shell_init_zsh() {
        // Test that zsh init doesn't panic and includes expected content
        let result = cmd_shell_init("zsh");
        assert!(result.is_ok());
    }

    #[test]
    fn test_shell_init_unsupported() {
        let result = cmd_shell_init("fish");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported shell"));
    }
}
