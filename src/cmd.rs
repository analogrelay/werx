use std::io::{BufRead, BufReader};
use std::process::{Output, Stdio};
use std::thread;

/// Run a command with tracing. Captures stdout/stderr and emits each line as a
/// `trace!` event in real-time, while still returning the full `Output` to the
/// caller so existing code that parses stdout/stderr continues to work.
///
/// Usage: replace `.output()?` with `crate::cmd::run(&mut cmd)?`
pub fn run(cmd: &mut std::process::Command) -> anyhow::Result<Output> {
    let program = cmd.get_program().to_string_lossy().into_owned();
    let args: Vec<String> = cmd
        .get_args()
        .map(|a| a.to_string_lossy().into_owned())
        .collect();
    let cmd_str = format!("{} {}", program, args.join(" "));

    let span = tracing::debug_span!("cmd", cmd = %cmd_str);
    let _enter = span.enter();

    tracing::debug!("executing");

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout_pipe = child.stdout.take().unwrap();
    let stderr_pipe = child.stderr.take().unwrap();

    // Clone the current span so worker threads emit events inside it
    let stdout_span = tracing::Span::current();
    let stderr_span = tracing::Span::current();

    let stdout_thread = thread::spawn(move || {
        let _enter = stdout_span.enter();
        let reader = BufReader::new(stdout_pipe);
        let mut captured = Vec::new();
        for line in reader.lines().flatten() {
            tracing::trace!(stream = "stdout", "{}", line);
            captured.extend_from_slice(line.as_bytes());
            captured.push(b'\n');
        }
        captured
    });

    let stderr_thread = thread::spawn(move || {
        let _enter = stderr_span.enter();
        let reader = BufReader::new(stderr_pipe);
        let mut captured = Vec::new();
        for line in reader.lines().flatten() {
            tracing::trace!(stream = "stderr", "{}", line);
            captured.extend_from_slice(line.as_bytes());
            captured.push(b'\n');
        }
        captured
    });

    let status = child.wait()?;
    let stdout = stdout_thread.join().unwrap_or_default();
    let stderr = stderr_thread.join().unwrap_or_default();

    tracing::debug!(exit_code = %status.code().unwrap_or(-1), "done");

    Ok(Output {
        status,
        stdout,
        stderr,
    })
}
