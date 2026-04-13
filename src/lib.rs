pub mod discover;
pub mod generate;
pub mod ir;
pub mod log;
pub mod output;
pub mod parse;

use std::path::Path;
use std::process::ExitCode;

#[derive(Debug, Clone)]
pub enum Mode {
    Check,
    Fix { overwrite: bool },
    Pr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Cursor,
    Copilot,
}

pub fn all_targets() -> Vec<Target> {
    vec![Target::Cursor, Target::Copilot]
}

pub fn parse_targets(names: &[String]) -> Result<Vec<Target>, String> {
    names
        .iter()
        .map(|n| match n.as_str() {
            "cursor" => Ok(Target::Cursor),
            "copilot" => Ok(Target::Copilot),
            other => Err(format!("unknown target: {other}")),
        })
        .collect()
}

pub fn run(
    root: &Path,
    mode: Mode,
    targets: &[Target],
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let root = std::fs::canonicalize(root)
        .map_err(|e| format!("Invalid path '{}': {e}", root.display()))?;

    let sources = discover::discover(&root);

    if sources.claude_md.is_none()
        && sources.rules.is_empty()
        && sources.skills.is_empty()
        && sources.mcp_json.is_none()
    {
        log::info("No Claude config found, nothing to sync.");
        return Ok(ExitCode::SUCCESS);
    }

    let config = parse::parse_all(&sources).inspect_err(|e| {
        log::error(e);
    })?;

    let claude_md_content = sources
        .claude_md
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok());

    let mut generated = Vec::new();
    for target in targets {
        match target {
            Target::Cursor => generated.extend(generate::cursor::generate(&root, &config)),
            Target::Copilot => generated.extend(generate::copilot::generate(
                &root,
                claude_md_content.as_deref(),
            )),
        }
    }

    match mode {
        Mode::Check => {
            let result = output::check(&generated);
            for path in &result.stale {
                log::warn_file(path, "out of sync");
            }
            for msg in &result.warnings {
                log::warn(msg);
            }
            if result.is_in_sync() {
                log::info("All files in sync.");
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(1))
            }
        }
        Mode::Fix { overwrite } => {
            let result = output::fix(&generated, overwrite);
            let deleted = output::cleanup(&root, &generated);
            for path in &result.written {
                log::info(&format!("Wrote {}", path.display()));
            }
            for path in &result.skipped {
                log::warn_file(path, "skipped (no generated-by marker)");
            }
            for path in &deleted {
                log::info(&format!("Deleted {}", path.display()));
            }
            if result.skipped.is_empty() {
                Ok(ExitCode::SUCCESS)
            } else {
                Ok(ExitCode::from(1))
            }
        }
        Mode::Pr => {
            let result = output::check(&generated);
            if result.is_in_sync() {
                println!("All agentic-sync files are in sync.");
                return Ok(ExitCode::SUCCESS);
            }
            println!("## agentic-sync drift detected\n");
            println!("The following files are out of sync with Claude config:\n");
            for path in &result.stale {
                println!("- `{}`", path.display());
                if path.exists() {
                    if let Some(file) = generated.iter().find(|f| f.path == *path) {
                        let existing = std::fs::read_to_string(path).unwrap_or_default();
                        let diff = similar::TextDiff::from_lines(&existing, &file.content);
                        println!("\n```diff");
                        for change in diff.iter_all_changes() {
                            let sign = match change.tag() {
                                similar::ChangeTag::Delete => "-",
                                similar::ChangeTag::Insert => "+",
                                similar::ChangeTag::Equal => " ",
                            };
                            print!("{sign}{change}");
                        }
                        println!("```\n");
                    }
                } else {
                    println!("  (new file)\n");
                }
            }
            for msg in &result.warnings {
                println!("- Warning: {msg}");
            }
            println!("\nRun `agentic-sync --fix` to update.");
            Ok(ExitCode::from(1))
        }
    }
}
