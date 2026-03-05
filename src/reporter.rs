use std::collections::VecDeque;
use std::io::IsTerminal;
use std::sync::{Arc, Mutex};

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};

const SPINNER_CHARS: &str = "⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏";
const TICK_INTERVAL: std::time::Duration = std::time::Duration::from_millis(80);

/// Central reporter that abstracts over TTY/non-TTY output.
pub struct Reporter {
    pub verbose: bool,
    pub is_tty: bool,
    mp: MultiProgress,
}

impl Reporter {
    pub fn new(verbose: bool) -> Self {
        let is_tty = std::io::stdout().is_terminal();
        Self {
            verbose,
            is_tty,
            mp: MultiProgress::new(),
        }
    }

    /// Print a message above any active spinners (TTY) or to stdout (non-TTY).
    pub fn println(&self, msg: &str) {
        if self.is_tty {
            let _ = self.mp.println(msg);
        } else {
            println!("{}", msg);
        }
    }

    /// Print only in verbose mode.
    pub fn verbose_line(&self, msg: &str) {
        if self.verbose {
            self.println(msg);
        }
    }

    /// Start a new operation, returning a handle for progress updates.
    pub fn start_operation(&self, label: &str) -> OperationHandle {
        let pb = if self.is_tty {
            let pb = self.mp.add(ProgressBar::new_spinner());
            pb.set_style(
                ProgressStyle::default_spinner()
                    .tick_chars(SPINNER_CHARS)
                    .template("{spinner:.cyan} {prefix:.bold}: {wide_msg:.dim}")
                    .unwrap(),
            );
            pb.set_prefix(label.to_string());
            pb.enable_steady_tick(TICK_INTERVAL);
            Some(pb)
        } else {
            println!("  {}...", label);
            None
        };

        OperationHandle {
            pb,
            label: label.to_string(),
            verbose: self.verbose,
            lines: Arc::new(Mutex::new(VecDeque::new())),
        }
    }
}

/// Handle for a running operation, used to log progress and finish.
/// Must be Clone + Send + Sync for rayon compatibility.
#[derive(Clone)]
pub struct OperationHandle {
    pb: Option<ProgressBar>,
    pub label: String,
    verbose: bool,
    lines: Arc<Mutex<VecDeque<String>>>,
}

impl OperationHandle {
    /// Create a no-op handle (non-TTY, non-verbose) for use in tests or non-reporter contexts.
    pub fn noop() -> Self {
        Self {
            pb: None,
            label: String::new(),
            verbose: false,
            lines: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Log a line of output from the running operation.
    ///
    /// - TTY: updates spinner message with last 3 lines
    /// - Non-TTY + verbose: prints the line
    /// - Non-TTY + non-verbose: no-op
    pub fn log_line(&self, line: &str) {
        if let Some(pb) = &self.pb {
            let mut lines = self.lines.lock().unwrap();
            lines.push_back(line.to_string());
            while lines.len() > 3 {
                lines.pop_front();
            }
            let msg = lines
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" | ");
            pb.set_message(msg);
        } else if self.verbose {
            println!("    {}", line);
        }
    }

    /// Finish the operation with a success message (green checkmark).
    pub fn finish_ok(&self, msg: &str) {
        let decorated = format!("{} {}", style("✓").green(), msg);
        if let Some(pb) = &self.pb {
            pb.finish_with_message(decorated);
        } else {
            println!("  {}", decorated);
        }
    }

    /// Finish the operation with an error message (red X).
    pub fn finish_err(&self, msg: &str) {
        let decorated = format!("{} {}", style("✗").red(), msg);
        if let Some(pb) = &self.pb {
            pb.finish_with_message(decorated);
        } else {
            println!("  {}", decorated);
        }
    }
}
