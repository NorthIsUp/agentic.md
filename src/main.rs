use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "agentic-sync",
    about = "Sync Claude Code config to other AI tools"
)]
struct Cli {
    /// Compare generated output to disk (default mode)
    #[arg(long)]
    check: bool,

    /// Write/overwrite target files
    #[arg(long)]
    fix: bool,

    /// Output markdown diff summary for PR comments
    #[arg(long)]
    pr: bool,

    /// Overwrite files even without generated-by header
    #[arg(long)]
    overwrite: bool,

    /// Target tools to generate for (comma-separated, repeatable)
    #[arg(long = "out", value_delimiter = ',')]
    targets: Vec<String>,

    /// Preferred instruction file: "claude" (default) or "agents"
    /// When "agents", AGENTS.md is the source of truth and CLAUDE.md
    /// is generated containing @AGENTS.md
    #[arg(long, default_value = "claude")]
    prefer: String,

    /// Project root (defaults to cwd)
    path: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mode = if cli.fix {
        agentic_sync::Mode::Fix {
            overwrite: cli.overwrite,
        }
    } else if cli.pr {
        agentic_sync::Mode::Pr
    } else {
        agentic_sync::Mode::Check
    };

    let targets = if cli.targets.is_empty() {
        agentic_sync::all_targets()
    } else {
        match agentic_sync::parse_targets(&cli.targets) {
            Ok(t) => t,
            Err(e) => {
                agentic_sync::log::error(&format!("Invalid target: {e}"));
                return ExitCode::from(2);
            }
        }
    };

    let prefer = match cli.prefer.as_str() {
        "claude" => agentic_sync::Prefer::Claude,
        "agents" => agentic_sync::Prefer::Agents,
        other => {
            agentic_sync::log::error(&format!("Invalid --prefer value: {other} (use 'claude' or 'agents')"));
            return ExitCode::from(2);
        }
    };

    let root = cli.path.unwrap_or_else(|| PathBuf::from("."));

    match agentic_sync::run(&root, mode, &targets, prefer) {
        Ok(status) => status,
        Err(e) => {
            agentic_sync::log::error(&format!("{e}"));
            ExitCode::from(2)
        }
    }
}
