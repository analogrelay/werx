use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

/// Environment variable for custom Werx location
const WERX_DIR_ENV: &str = "WERX_DIR";

/// Default Werx location relative to home directory
const DEFAULT_WERX_DIR: &str = "werx";

/// Resolve the Werx path based on priority: CLI arg > env var > default
///
/// Priority order:
/// 1. Command-line argument (if provided)
/// 2. WERX_DIR environment variable
/// 3. Default location: ~/werx
pub fn resolve_werx_path(cli_path: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cli_path {
        return expand_path(&path);
    }

    if let Ok(env_path) = env::var(WERX_DIR_ENV) {
        return expand_path(&PathBuf::from(env_path));
    }

    // Default: ~/werx
    let home = dirs::home_dir().context("Could not determine home directory")?;

    Ok(home.join(DEFAULT_WERX_DIR))
}

/// Expand tilde (~) in paths to home directory
fn expand_path(path: &std::path::Path) -> Result<PathBuf> {
    let path_str = path.to_str().context("Path contains invalid UTF-8")?;

    if path_str.starts_with("~/") || path_str == "~" {
        let home = dirs::home_dir().context("Could not determine home directory")?;

        if path_str == "~" {
            Ok(home)
        } else {
            Ok(home.join(&path_str[2..]))
        }
    } else {
        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::sync::Mutex;

    // Mutex to ensure env var tests run serially
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_cli_path_overrides_all() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::set_var(WERX_DIR_ENV, "/env/path");
        }
        let result =
            resolve_werx_path(Some(PathBuf::from("/cli/path"))).expect("Should resolve CLI path");
        assert_eq!(result, PathBuf::from("/cli/path"));
        unsafe {
            env::remove_var(WERX_DIR_ENV);
        }
    }

    #[test]
    fn test_env_var_overrides_default() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::set_var(WERX_DIR_ENV, "/env/path");
        }
        let result = resolve_werx_path(None).expect("Should resolve env var path");
        assert_eq!(result, PathBuf::from("/env/path"));
        unsafe {
            env::remove_var(WERX_DIR_ENV);
        }
    }

    #[test]
    fn test_default_path() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            env::remove_var(WERX_DIR_ENV);
        }
        let result = resolve_werx_path(None).expect("Should resolve default path");
        let expected = dirs::home_dir()
            .expect("Should have home dir")
            .join(DEFAULT_WERX_DIR);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tilde_expansion() {
        let path = PathBuf::from("~/test");
        let result = expand_path(&path).expect("Should expand tilde");
        let expected = dirs::home_dir().expect("Should have home dir").join("test");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_tilde_only_expansion() {
        let path = PathBuf::from("~");
        let result = expand_path(&path).expect("Should expand tilde");
        let expected = dirs::home_dir().expect("Should have home dir");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_no_tilde_expansion() {
        let path = PathBuf::from("/absolute/path");
        let result = expand_path(&path).expect("Should not expand");
        assert_eq!(result, PathBuf::from("/absolute/path"));
    }
}
