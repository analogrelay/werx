use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;

/// Emits a directive to change the shell's current directory.
///
/// Writes a `change_directory` directive to the file specified by
/// `WERX_DIRECTIVE_FILE`. Returns an error if the env var is not set
/// or the file cannot be written to.
///
/// Callers should treat a returned error as a non-fatal warning — the
/// command completed successfully, but the shell could not be notified.
pub fn emit_change_directory<P: AsRef<Path>>(path: P) -> Result<()> {
    let directive_file = std::env::var("WERX_DIRECTIVE_FILE").map_err(|_| {
        anyhow::anyhow!(
            "couldn't communicate with your shell to change the working directory; \
             have you installed the shell hook? (run 'werx shell init --help')"
        )
    })?;

    let path_str = path.as_ref().display().to_string();
    anyhow::ensure!(!path_str.contains('\n'), "Directory path cannot contain newlines");

    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(&directive_file)
        .with_context(|| format!("Failed to open directive file '{}'", directive_file))?;

    writeln!(file, "@werx:change_directory:{}", path_str)
        .with_context(|| format!("Failed to write to directive file '{}'", directive_file))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_change_directory_no_env_var() {
        // Temporarily remove the env var to ensure it is not set
        let saved = std::env::var("WERX_DIRECTIVE_FILE").ok();
        unsafe {
            std::env::remove_var("WERX_DIRECTIVE_FILE");
        }
        let result = emit_change_directory("/tmp/test");
        if let Some(val) = saved {
            unsafe {
                std::env::set_var("WERX_DIRECTIVE_FILE", val);
            }
        }
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("shell hook"), "error should mention shell hook: {}", msg);
    }

    #[test]
    fn test_emit_change_directory_writes_directive() {
        let tmp = tempfile::NamedTempFile::new().expect("create temp file");
        let path = tmp.path().to_str().unwrap().to_string();
        unsafe {
            std::env::set_var("WERX_DIRECTIVE_FILE", &path);
        }
        let result = emit_change_directory("/tmp/test-workspace");
        unsafe {
            std::env::remove_var("WERX_DIRECTIVE_FILE");
        }
        result.expect("emit_change_directory should succeed");
        let content = std::fs::read_to_string(&path).expect("read directive file");
        assert_eq!(content, "@werx:change_directory:/tmp/test-workspace\n");
    }
}
