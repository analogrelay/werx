use anyhow::{Context, Result, anyhow};
use std::fs;
use std::io::IsTerminal;
use std::process::Command;

use crate::{Werx, cmd, config::Config, github};

// ── Slug helpers ──────────────────────────────────────────────────────────────

/// Convert arbitrary text into a git-branch-friendly slug.
/// Rules: lowercase, runs of non-alphanumeric chars become a single hyphen,
/// leading/trailing hyphens stripped.
pub fn slugify(text: &str) -> String {
    let mut result = String::new();
    let mut in_sep = true; // treat leading non-alnum as separator (so they're trimmed)

    for ch in text.to_lowercase().chars() {
        if ch.is_alphanumeric() {
            result.push(ch);
            in_sep = false;
        } else if !in_sep {
            result.push('-');
            in_sep = true;
        }
    }

    // Trim trailing hyphen that was added for the last separator run
    result.trim_end_matches('-').to_string()
}

// ── Branch naming ─────────────────────────────────────────────────────────────

/// Build a branch name following the `username/[N-]topic` pattern.
pub fn make_branch_name(username: &str, issue_num: Option<u64>, topic: &str) -> String {
    match issue_num {
        Some(n) => format!("{}/{}-{}", username, n, topic),
        None => format!("{}/{}", username, topic),
    }
}

/// Resolve the GitHub username for branch naming.
///
/// Priority:
/// 1. Cached value in `config.github.username`
/// 2. Fetch via `gh api user` and cache
/// 3. Interactive prompt (or error in non-interactive mode)
pub fn resolve_username(werx: &Werx, config: &mut Config) -> Result<String> {
    if let Some(ref username) = config.github.username {
        return Ok(username.clone());
    }

    if github::is_gh_available() {
        match github::fetch_username() {
            Ok(username) if !username.is_empty() => {
                config.github.username = Some(username.clone());
                config
                    .save(&werx.config_file())
                    .context("Failed to save cached GitHub username to config")?;
                return Ok(username);
            }
            Ok(_) => {}
            Err(e) => {
                tracing::warn!("Failed to fetch GitHub username via gh: {}", e);
            }
        }
    }

    if !std::io::stdin().is_terminal() {
        return Err(anyhow!(
            "GitHub username not configured and cannot prompt in non-interactive mode.\n\
             Set `[github] username = \"your-username\"` in your werx config."
        ));
    }

    let username: String = dialoguer::Input::with_theme(&dialoguer::theme::ColorfulTheme::default())
        .with_prompt("GitHub username")
        .interact_text()
        .context("Failed to prompt for GitHub username")?;

    let username = username.trim().to_string();
    if username.is_empty() {
        return Err(anyhow!("GitHub username cannot be empty"));
    }

    config.github.username = Some(username.clone());
    config
        .save(&werx.config_file())
        .context("Failed to save GitHub username to config")?;
    Ok(username)
}

// ── Agent slug generation ─────────────────────────────────────────────────────

/// Build the prompt sent to the coding agent for slug generation.
pub fn build_slug_prompt(title: &str, body: &str) -> String {
    format!(
        "You are helping generate a short git branch name slug.\n\
         \n\
         Issue title: {title}\n\
         Issue body:\n\
         {body}\n\
         \n\
         Produce a slug of at most 4 words that describes the work in this issue.\n\
         Rules:\n\
         - lowercase, hyphen-separated words only\n\
         - no issue number prefix\n\
         - no username prefix\n\
         - 4 words maximum\n\
         \n\
         Respond with ONLY the tag and slug, nothing else:\n\
         <branch-slug>the-slug-here</branch-slug>\n\
         \n\
         Do not include any explanation, preamble, or other text."
    )
}

/// Invoke a configured coding agent, writing the prompt to a temp file and
/// capturing stdout. Returns an error on non-zero exit code.
pub fn invoke_agent(agent: &str, prompt: &str) -> Result<String> {
    let tmp = tempfile::NamedTempFile::new().context("Failed to create temp file for prompt")?;
    fs::write(tmp.path(), prompt.as_bytes()).context("Failed to write prompt to temp file")?;

    let prompt_path = tmp.path().to_string_lossy().into_owned();

    let output = match agent {
        "claude" => cmd::run(
            Command::new("sh")
                .arg("-c")
                .arg(format!("cat '{}' | claude --print", prompt_path)),
        ),
        "copilot" => cmd::run(
            Command::new("sh")
                .arg("-c")
                .arg(format!("cat '{}' | gh copilot suggest -t shell", prompt_path)),
        ),
        other => return Err(anyhow!("Unknown agent: '{}'. Supported: claude, copilot", other)),
    }
    .context(format!("Failed to invoke agent '{}'", agent))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Agent '{}' exited with non-zero status: {}",
            agent,
            stderr
        ));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Extract a branch slug from agent output by looking for `<branch-slug>…</branch-slug>`.
/// Normalizes the captured text with `slugify`.
pub fn extract_branch_slug(output: &str) -> Option<String> {
    const START: &str = "<branch-slug>";
    const END: &str = "</branch-slug>";

    let start_idx = output.find(START)? + START.len();
    let end_idx = start_idx + output[start_idx..].find(END)?;
    let raw = output[start_idx..end_idx].trim();
    let slug = slugify(raw);
    if slug.is_empty() {
        None
    } else {
        Some(slug)
    }
}

/// Generate a branch slug for an issue, using the configured agent if available,
/// falling back to a slugified version of the issue title.
pub fn generate_slug(_werx: &Werx, config: &Config, title: &str, body: &str) -> String {
    if let Some(ref agent) = config.agent.agent {
        let prompt = build_slug_prompt(title, body);
        match invoke_agent(agent, &prompt) {
            Ok(output) => {
                if let Some(slug) = extract_branch_slug(&output) {
                    return slug;
                }
                tracing::warn!(
                    "Agent did not return a valid <branch-slug> tag; falling back to title slug"
                );
            }
            Err(e) => {
                tracing::warn!("Agent invocation failed ({}); falling back to title slug", e);
            }
        }
    }

    slugify(title)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── slugify ───────────────────────────────────────────────────────────────

    #[test]
    fn test_slugify_lowercase() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_spaces_become_hyphens() {
        assert_eq!(slugify("fix the bug"), "fix-the-bug");
    }

    #[test]
    fn test_slugify_special_chars_become_hyphen() {
        assert_eq!(slugify("feat: add foo/bar"), "feat-add-foo-bar");
    }

    #[test]
    fn test_slugify_consecutive_separators_collapse() {
        assert_eq!(slugify("foo  --  bar"), "foo-bar");
    }

    #[test]
    fn test_slugify_trims_leading_separators() {
        assert_eq!(slugify("  --hello"), "hello");
    }

    #[test]
    fn test_slugify_trims_trailing_separators() {
        assert_eq!(slugify("hello--  "), "hello");
    }

    #[test]
    fn test_slugify_all_non_alnum() {
        assert_eq!(slugify("---"), "");
    }

    #[test]
    fn test_slugify_empty_string() {
        assert_eq!(slugify(""), "");
    }

    // ── make_branch_name ─────────────────────────────────────────────────────

    #[test]
    fn test_make_branch_name_with_issue_number() {
        assert_eq!(
            make_branch_name("alice", Some(42), "fix-the-bug"),
            "alice/42-fix-the-bug"
        );
    }

    #[test]
    fn test_make_branch_name_without_issue_number() {
        assert_eq!(
            make_branch_name("alice", None, "add-feature"),
            "alice/add-feature"
        );
    }

    #[test]
    fn test_make_branch_name_large_issue_number() {
        assert_eq!(
            make_branch_name("bob", Some(1234), "some-work"),
            "bob/1234-some-work"
        );
    }

    // ── extract_branch_slug ──────────────────────────────────────────────────

    #[test]
    fn test_extract_branch_slug_clean_tag() {
        assert_eq!(
            extract_branch_slug("<branch-slug>my-slug</branch-slug>"),
            Some("my-slug".to_string())
        );
    }

    #[test]
    fn test_extract_branch_slug_tag_in_prose() {
        let output = "Sure! Here is your slug:\n<branch-slug>fix-login-bug</branch-slug>\nHope that helps!";
        assert_eq!(
            extract_branch_slug(output),
            Some("fix-login-bug".to_string())
        );
    }

    #[test]
    fn test_extract_branch_slug_normalizes_via_slugify() {
        assert_eq!(
            extract_branch_slug("<branch-slug>Fix Login Bug</branch-slug>"),
            Some("fix-login-bug".to_string())
        );
    }

    #[test]
    fn test_extract_branch_slug_no_tag() {
        assert!(extract_branch_slug("no tags here").is_none());
    }

    #[test]
    fn test_extract_branch_slug_empty_string() {
        assert!(extract_branch_slug("").is_none());
    }

    // ── build_slug_prompt ────────────────────────────────────────────────────

    #[test]
    fn test_build_slug_prompt_contains_title() {
        let prompt = build_slug_prompt("Fix login bug", "Details here");
        assert!(prompt.contains("Fix login bug"));
    }

    #[test]
    fn test_build_slug_prompt_contains_body() {
        let prompt = build_slug_prompt("My Title", "Some body text");
        assert!(prompt.contains("Some body text"));
    }

    #[test]
    fn test_build_slug_prompt_contains_tag_instruction() {
        let prompt = build_slug_prompt("Title", "Body");
        assert!(prompt.contains("<branch-slug>"));
        assert!(prompt.contains("</branch-slug>"));
    }

    #[test]
    fn test_build_slug_prompt_contains_rules() {
        let prompt = build_slug_prompt("Title", "Body");
        assert!(prompt.contains("4 words maximum"));
        assert!(prompt.contains("lowercase"));
    }
}
