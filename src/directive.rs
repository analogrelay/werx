use std::path::Path;

/// Emits a directive to change the shell's current directory.
///
/// This emits a `change_directory` directive to stderr that can be
/// intercepted by the shell wrapper function to execute `cd`.
pub fn emit_change_directory<P: AsRef<Path>>(path: P) {
    emit_directive("change_directory", &path.as_ref().display().to_string());
}

/// Emits a directive to stderr in the format: @werx:<name>:<arg>
///
/// Directives are used to communicate shell actions (like directory changes)
/// from the binary to the shell wrapper function.
///
/// # Panics
///
/// Panics if the directive name contains invalid characters (only lowercase
/// letters and underscores are allowed) or if the argument contains newlines.
pub fn emit_directive(name: &str, arg: &str) {
    // Validate directive name: only lowercase letters and underscores
    assert!(
        name.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
        "Directive name must contain only lowercase letters and underscores"
    );

    // Validate argument doesn't contain newlines
    assert!(
        !arg.contains('\n'),
        "Directive argument cannot contain newlines"
    );

    eprintln!("@werx:{}:{}", name, arg);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_emit_directive_format() {
        // Note: This test can't easily verify stderr output without mocking.
        // The actual verification happens in integration tests.
        // Here we just ensure the function doesn't panic with valid input.
        emit_directive("test_directive", "test_arg");
    }

    #[test]
    #[should_panic(expected = "Directive name must contain only lowercase letters and underscores")]
    fn test_emit_directive_validation_uppercase() {
        emit_directive("TestDirective", "arg");
    }

    #[test]
    #[should_panic(expected = "Directive name must contain only lowercase letters and underscores")]
    fn test_emit_directive_validation_hyphen() {
        emit_directive("test-directive", "arg");
    }

    #[test]
    #[should_panic(expected = "Directive argument cannot contain newlines")]
    fn test_emit_directive_validation_newline() {
        emit_directive("test_directive", "arg\nwith\nnewlines");
    }

    #[test]
    fn test_emit_change_directory() {
        let path = PathBuf::from("/tmp/test");
        emit_change_directory(&path);
    }

    #[test]
    fn test_emit_directive_with_underscores() {
        emit_directive("change_directory", "/path/to/dir");
    }
}
