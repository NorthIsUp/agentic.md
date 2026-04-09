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

pub fn run(root: &Path, mode: Mode, targets: &[Target]) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let _ = (root, mode, targets);
    Ok(ExitCode::SUCCESS)
}
