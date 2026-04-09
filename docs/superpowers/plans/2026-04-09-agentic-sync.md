# agentic-sync Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI that reads Claude Code project config (CLAUDE.md, rules, skills, MCP) and generates equivalent Cursor and Copilot config files.

**Architecture:** A pipeline of parse → intermediate representation → generate. Sources are parsed into a common `ProjectConfig` struct (sections, skills, mcp), then each target generator transforms that IR into its output format. The CLI orchestrates discovery, parsing, generation, and diffing.

**Tech Stack:** Rust, clap (CLI parsing), serde + serde_json + serde_yaml (serialization), similar (diffing for --pr mode)

---

## File Structure

```
Cargo.toml
src/
  main.rs              # CLI entry point, arg parsing, orchestration
  lib.rs               # Re-exports for testing
  discover.rs          # Find source files at project root
  parse/
    mod.rs             # Parse module re-exports
    claude_md.rs       # CLAUDE.md section splitter + frontmatter parser
    rules.rs           # .claude/rules/*.md parser
    skills.rs          # .claude/skills/*/SKILL.md parser
    mcp.rs             # .mcp.json parser
  ir.rs                # Intermediate representation types (ProjectConfig, Section, Skill, McpConfig)
  generate/
    mod.rs             # Generator trait + dispatch
    cursor.rs          # Cursor .mdc + mcp.json generator
    copilot.rs         # Copilot instructions.md generator
  output.rs            # File writing, ownership checking, cleanup, diffing
  log.rs               # Logging with GitHub Actions annotation support
tests/
  integration/
    mod.rs
    check_test.rs      # --check mode e2e tests
    fix_test.rs        # --fix mode e2e tests
    cleanup_test.rs    # Stale file cleanup tests
  fixtures/
    basic/             # Simple CLAUDE.md + expected outputs
    with_overrides/    # Sections with cursor: frontmatter
    with_skills/       # .claude/skills/ examples
    with_mcp/          # .mcp.json examples
    conflict/          # Pre-existing files without generated-by header
```

---

### Task 1: Project Scaffold & CLI Parsing

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: Initialize Cargo project**

```bash
cd /Users/adam/src/agentic.md
cargo init --name agentic-sync
```

- [ ] **Step 2: Add dependencies to Cargo.toml**

Replace the generated `Cargo.toml` with:

```toml
[package]
name = "agentic-sync"
version = "0.1.0"
edition = "2024"

[dependencies]
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
serde_yaml = "0.9"
similar = "2"
```

- [ ] **Step 3: Write CLI arg parsing in main.rs**

```rust
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(name = "agentic-sync", about = "Sync Claude Code config to other AI tools")]
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

    /// Project root (defaults to cwd)
    path: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let mode = if cli.fix {
        agentic_sync::Mode::Fix { overwrite: cli.overwrite }
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

    let root = cli.path.unwrap_or_else(|| PathBuf::from("."));

    match agentic_sync::run(&root, mode, &targets) {
        Ok(status) => status,
        Err(e) => {
            agentic_sync::log::error(&format!("{e}"));
            ExitCode::from(2)
        }
    }
}
```

- [ ] **Step 4: Write lib.rs with top-level types and stub run function**

```rust
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
    // Will be filled in as modules are implemented
    let _ = (root, mode, targets);
    Ok(ExitCode::SUCCESS)
}
```

- [ ] **Step 5: Create stub modules so it compiles**

Create empty module files:

`src/discover.rs`:
```rust
```

`src/parse/mod.rs`:
```rust
pub mod claude_md;
pub mod mcp;
pub mod rules;
pub mod skills;
```

`src/parse/claude_md.rs`, `src/parse/rules.rs`, `src/parse/skills.rs`, `src/parse/mcp.rs`:
```rust
```

`src/ir.rs`:
```rust
```

`src/generate/mod.rs`:
```rust
pub mod copilot;
pub mod cursor;
```

`src/generate/cursor.rs`, `src/generate/copilot.rs`:
```rust
```

`src/output.rs`:
```rust
```

`src/log.rs`:
```rust
pub fn error(msg: &str) {
    if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        eprintln!("::error::{msg}");
    } else {
        eprintln!("error: {msg}");
    }
}
```

- [ ] **Step 6: Verify it compiles and runs**

```bash
cargo build
cargo run -- --help
cargo run -- --check
```

Expected: Compiles. `--help` prints usage. `--check` exits 0 (stub).

- [ ] **Step 7: Commit**

```bash
git init
git add Cargo.toml src/ docs/
git commit -m "feat: scaffold agentic-sync Rust CLI with arg parsing"
```

---

### Task 2: Intermediate Representation

**Files:**
- Create: `src/ir.rs`

- [ ] **Step 1: Write failing test for IR types**

Add to bottom of `src/ir.rs`:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectConfig {
    pub sections: Vec<Section>,
    pub skills: Vec<Skill>,
    pub mcp: Option<McpConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    /// Heading text, e.g. "Commit Conventions". None for preamble.
    pub title: Option<String>,
    /// Slug for filename, e.g. "commit-conventions". "_project" for preamble.
    pub slug: String,
    /// Markdown body content (frontmatter stripped)
    pub body: String,
    /// Source: "claude-md" or "rules"
    pub source: SectionSource,
    /// Per-target overrides parsed from frontmatter, keyed by target name
    pub target_overrides: std::collections::HashMap<String, Vec<(String, String)>>,
    /// Description from Claude frontmatter (if rules file)
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionSource {
    ClaudeMd,
    Rules,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub name: String,
    pub description: Option<String>,
    pub paths: Vec<String>,
    pub disable_model_invocation: bool,
    /// Per-target overrides parsed from frontmatter
    pub target_overrides: std::collections::HashMap<String, Vec<(String, String)>>,
    /// Markdown body content (frontmatter stripped)
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct McpConfig {
    /// Raw JSON value — the tool normalizes on output, not on parse
    pub servers: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_config_default() {
        let config = ProjectConfig {
            sections: vec![],
            skills: vec![],
            mcp: None,
        };
        assert!(config.sections.is_empty());
        assert!(config.skills.is_empty());
        assert!(config.mcp.is_none());
    }

    #[test]
    fn section_slug_for_preamble() {
        let section = Section {
            title: None,
            slug: "_project".to_string(),
            body: "preamble content".to_string(),
            source: SectionSource::ClaudeMd,
            target_overrides: Default::default(),
            description: None,
        };
        assert_eq!(section.slug, "_project");
        assert!(section.title.is_none());
    }
}
```

- [ ] **Step 2: Run tests**

```bash
cargo test ir::tests
```

Expected: PASS (2 tests).

- [ ] **Step 3: Commit**

```bash
git add src/ir.rs
git commit -m "feat: define intermediate representation types"
```

---

### Task 3: Logging Module

**Files:**
- Create: `src/log.rs`

- [ ] **Step 1: Write the logging module**

Replace `src/log.rs`:

```rust
use std::path::Path;

fn is_github_actions() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
}

pub fn error(msg: &str) {
    if is_github_actions() {
        eprintln!("::error::{msg}");
    } else {
        eprintln!("error: {msg}");
    }
}

pub fn error_file(path: &Path, msg: &str) {
    if is_github_actions() {
        eprintln!("::error file={path}::{msg}", path = path.display());
    } else {
        eprintln!("error: {}: {msg}", path.display());
    }
}

pub fn warn(msg: &str) {
    if is_github_actions() {
        eprintln!("::warning::{msg}");
    } else {
        eprintln!("warning: {msg}");
    }
}

pub fn warn_file(path: &Path, msg: &str) {
    if is_github_actions() {
        eprintln!("::warning file={path}::{msg}", path = path.display());
    } else {
        eprintln!("warning: {}: {msg}", path.display());
    }
}

pub fn info(msg: &str) {
    if is_github_actions() {
        eprintln!("::notice::{msg}");
    } else {
        eprintln!("{msg}");
    }
}

pub fn group(name: &str) {
    if is_github_actions() {
        eprintln!("::group::{name}");
    }
}

pub fn endgroup() {
    if is_github_actions() {
        eprintln!("::endgroup::");
    }
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/log.rs
git commit -m "feat: add logging with GitHub Actions annotation support"
```

---

### Task 4: Source Discovery

**Files:**
- Create: `src/discover.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing test**

```rust
use std::path::{Path, PathBuf};

/// All source files found at the project root.
#[derive(Debug, Default)]
pub struct Sources {
    pub claude_md: Option<PathBuf>,
    pub rules: Vec<PathBuf>,
    pub skills: Vec<PathBuf>,
    pub mcp_json: Option<PathBuf>,
}

pub fn discover(root: &Path) -> Sources {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup_dir(base: &Path) {
        fs::create_dir_all(base.join(".claude/rules")).unwrap();
        fs::create_dir_all(base.join(".claude/skills/explain-code")).unwrap();
        fs::write(base.join("CLAUDE.md"), "# Project\n## Stack\nRust").unwrap();
        fs::write(base.join(".claude/rules/testing.md"), "# Testing rules").unwrap();
        fs::write(
            base.join(".claude/skills/explain-code/SKILL.md"),
            "---\nname: explain-code\n---\nExplain code",
        )
        .unwrap();
        fs::write(base.join(".mcp.json"), r#"{"server": {}}"#).unwrap();
    }

    #[test]
    fn discovers_all_sources() {
        let dir = tempfile::tempdir().unwrap();
        setup_dir(dir.path());
        let sources = discover(dir.path());
        assert!(sources.claude_md.is_some());
        assert_eq!(sources.rules.len(), 1);
        assert_eq!(sources.skills.len(), 1);
        assert!(sources.mcp_json.is_some());
    }

    #[test]
    fn empty_dir_returns_empty_sources() {
        let dir = tempfile::tempdir().unwrap();
        let sources = discover(dir.path());
        assert!(sources.claude_md.is_none());
        assert!(sources.rules.is_empty());
        assert!(sources.skills.is_empty());
        assert!(sources.mcp_json.is_none());
    }
}
```

- [ ] **Step 2: Add tempfile dev-dependency to Cargo.toml**

```toml
[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test discover::tests
```

Expected: FAIL (todo! panics).

- [ ] **Step 4: Implement discover()**

Replace the `todo!()` in `discover`:

```rust
pub fn discover(root: &Path) -> Sources {
    let mut sources = Sources::default();

    let claude_md = root.join("CLAUDE.md");
    if claude_md.is_file() {
        sources.claude_md = Some(claude_md);
    }

    let rules_dir = root.join(".claude/rules");
    if rules_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&rules_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") && path.is_file() {
                    sources.rules.push(path);
                }
            }
            sources.rules.sort();
        }
    }

    let skills_dir = root.join(".claude/skills");
    if skills_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&skills_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let skill_md = path.join("SKILL.md");
                if path.is_dir() && skill_md.is_file() {
                    sources.skills.push(skill_md);
                }
            }
            sources.skills.sort();
        }
    }

    let mcp = root.join(".mcp.json");
    if mcp.is_file() {
        sources.mcp_json = Some(mcp);
    }

    sources
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test discover::tests
```

Expected: PASS (2 tests).

- [ ] **Step 6: Commit**

```bash
git add src/discover.rs Cargo.toml
git commit -m "feat: source file discovery"
```

---

### Task 5: CLAUDE.md Parser

**Files:**
- Create: `src/parse/claude_md.rs`
- Test: inline `#[cfg(test)]`

- [ ] **Step 1: Write failing tests**

```rust
use crate::ir::{Section, SectionSource};

/// Parse CLAUDE.md into sections split on ## headings.
pub fn parse_claude_md(content: &str) -> Vec<Section> {
    todo!()
}

/// Convert heading text to a URL-safe slug.
fn slugify(text: &str) -> String {
    todo!()
}

/// Parse a section-level frontmatter block (--- delimited) and extract
/// target-namespaced overrides. Returns (overrides_map, remaining_body).
fn parse_section_frontmatter(
    body: &str,
) -> (std::collections::HashMap<String, Vec<(String, String)>>, String) {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_h2_headings() {
        let input = "Preamble text\n\n## Stack\nRust + Clap\n\n## Testing\nUse cargo test\n";
        let sections = parse_claude_md(input);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].slug, "_project");
        assert!(sections[0].title.is_none());
        assert!(sections[0].body.contains("Preamble text"));
        assert_eq!(sections[1].slug, "stack");
        assert_eq!(sections[1].title.as_deref(), Some("Stack"));
        assert!(sections[1].body.contains("Rust + Clap"));
        assert_eq!(sections[2].slug, "testing");
    }

    #[test]
    fn skips_empty_sections() {
        let input = "## Empty\n\n## HasContent\nSome content\n";
        let sections = parse_claude_md(input);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slug, "has-content");
    }

    #[test]
    fn no_preamble_if_starts_with_heading() {
        let input = "## Stack\nRust\n";
        let sections = parse_claude_md(input);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slug, "stack");
    }

    #[test]
    fn deduplicates_slugs() {
        let input = "## Testing\nUnit tests\n\n## Testing\nIntegration tests\n";
        let sections = parse_claude_md(input);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].slug, "testing");
        assert_eq!(sections[1].slug, "testing-2");
    }

    #[test]
    fn parses_cursor_frontmatter_overrides() {
        let input = "## Testing\n---\ncursor: alwaysApply=false\ncursor: globs=**/*.test.ts\n---\nRun tests with cargo\n";
        let sections = parse_claude_md(input);
        assert_eq!(sections.len(), 1);
        let overrides = sections[0].target_overrides.get("cursor").unwrap();
        assert_eq!(overrides.len(), 2);
        assert_eq!(overrides[0], ("alwaysApply".to_string(), "false".to_string()));
        assert_eq!(overrides[1], ("globs".to_string(), "**/*.test.ts".to_string()));
        assert!(!sections[0].body.contains("---"));
        assert!(sections[0].body.contains("Run tests with cargo"));
    }

    #[test]
    fn slugify_works() {
        assert_eq!(slugify("Commit Conventions"), "commit-conventions");
        assert_eq!(slugify("API Design & Patterns"), "api-design--patterns");
        assert_eq!(slugify("  Spaces  "), "spaces");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test parse::claude_md::tests
```

Expected: FAIL (todo! panics).

- [ ] **Step 3: Implement slugify**

```rust
fn slugify(text: &str) -> String {
    text.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
```

- [ ] **Step 4: Implement parse_section_frontmatter**

```rust
fn parse_section_frontmatter(
    body: &str,
) -> (std::collections::HashMap<String, Vec<(String, String)>>, String) {
    let trimmed = body.trim_start();
    if !trimmed.starts_with("---") {
        return (Default::default(), body.to_string());
    }

    // Find closing ---
    let after_open = &trimmed[3..];
    let after_open = after_open.strip_prefix('\n').unwrap_or(after_open);
    let Some(close_pos) = after_open.find("\n---") else {
        return (Default::default(), body.to_string());
    };

    let frontmatter_block = &after_open[..close_pos];
    let rest = &after_open[close_pos + 4..]; // skip \n---
    let rest = rest.strip_prefix('\n').unwrap_or(rest);

    let mut overrides: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for line in frontmatter_block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Parse "target: key=value"
        if let Some((target, kv)) = line.split_once(": ") {
            let target = target.trim();
            let kv = kv.trim();
            if let Some((key, value)) = kv.split_once('=') {
                overrides
                    .entry(target.to_string())
                    .or_default()
                    .push((key.to_string(), value.to_string()));
            }
        }
    }

    (overrides, rest.to_string())
}
```

- [ ] **Step 5: Implement parse_claude_md**

```rust
pub fn parse_claude_md(content: &str) -> Vec<Section> {
    let mut raw_sections: Vec<(Option<String>, String)> = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_body = String::new();
    let mut in_preamble = true;

    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // Flush previous section
            if in_preamble {
                let trimmed = current_body.trim().to_string();
                if !trimmed.is_empty() {
                    raw_sections.push((None, trimmed));
                }
                in_preamble = false;
            } else {
                raw_sections.push((current_title.take(), current_body.clone()));
            }
            current_title = Some(heading.trim().to_string());
            current_body.clear();
        } else {
            if !current_body.is_empty() || !line.trim().is_empty() {
                current_body.push_str(line);
                current_body.push('\n');
            }
        }
    }

    // Flush last section
    if in_preamble {
        let trimmed = current_body.trim().to_string();
        if !trimmed.is_empty() {
            raw_sections.push((None, trimmed));
        }
    } else {
        raw_sections.push((current_title.take(), current_body.clone()));
    }

    // Build sections, dedup slugs, skip empty
    let mut slug_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut sections = Vec::new();

    for (title, body) in raw_sections {
        let (overrides, clean_body) = parse_section_frontmatter(&body);
        let clean_body_trimmed = clean_body.trim().to_string();

        if title.is_some() && clean_body_trimmed.is_empty() {
            continue; // skip empty sections
        }

        let base_slug = match &title {
            Some(t) => slugify(t),
            None => "_project".to_string(),
        };

        let count = slug_counts.entry(base_slug.clone()).or_insert(0);
        *count += 1;
        let slug = if *count > 1 {
            crate::log::warn(&format!(
                "Duplicate heading '{}', using slug '{}-{}'",
                title.as_deref().unwrap_or("_project"),
                base_slug,
                count
            ));
            format!("{base_slug}-{count}")
        } else {
            base_slug
        };

        sections.push(Section {
            title: title.clone(),
            slug,
            body: clean_body_trimmed,
            source: SectionSource::ClaudeMd,
            target_overrides: overrides,
            description: title,
        });
    }

    sections
}
```

- [ ] **Step 6: Run tests to verify they pass**

```bash
cargo test parse::claude_md::tests
```

Expected: PASS (6 tests).

- [ ] **Step 7: Commit**

```bash
git add src/parse/claude_md.rs
git commit -m "feat: CLAUDE.md section parser with frontmatter overrides"
```

---

### Task 6: Rules Parser

**Files:**
- Create: `src/parse/rules.rs`

- [ ] **Step 1: Write failing test**

```rust
use crate::ir::{Section, SectionSource};
use std::path::Path;

/// Parse a .claude/rules/*.md file into a Section.
pub fn parse_rule(path: &Path) -> Result<Section, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_rule_with_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("testing.md");
        fs::write(
            &path,
            "---\ndescription: Testing guidelines\ncursor: alwaysApply=false\n---\nRun cargo test\n",
        )
        .unwrap();
        let section = parse_rule(&path).unwrap();
        assert_eq!(section.slug, "testing");
        assert_eq!(section.description.as_deref(), Some("Testing guidelines"));
        assert!(section.body.contains("Run cargo test"));
        assert!(!section.body.contains("---"));
        let overrides = section.target_overrides.get("cursor").unwrap();
        assert_eq!(overrides[0].0, "alwaysApply");
    }

    #[test]
    fn parses_rule_without_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("style.md");
        fs::write(&path, "Use 4-space indentation\n").unwrap();
        let section = parse_rule(&path).unwrap();
        assert_eq!(section.slug, "style");
        assert!(section.body.contains("4-space"));
        assert!(section.description.is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test parse::rules::tests
```

- [ ] **Step 3: Implement parse_rule**

```rust
pub fn parse_rule(path: &Path) -> Result<Section, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let (description, target_overrides, body) = parse_rule_frontmatter(&content);

    Ok(Section {
        title: Some(name.to_string()),
        slug: name.to_string(),
        body: body.trim().to_string(),
        source: SectionSource::Rules,
        target_overrides,
        description,
    })
}

fn parse_rule_frontmatter(
    content: &str,
) -> (
    Option<String>,
    std::collections::HashMap<String, Vec<(String, String)>>,
    String,
) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, Default::default(), content.to_string());
    }

    let after_open = &trimmed[3..];
    let after_open = after_open.strip_prefix('\n').unwrap_or(after_open);
    let Some(close_pos) = after_open.find("\n---") else {
        return (None, Default::default(), content.to_string());
    };

    let frontmatter_block = &after_open[..close_pos];
    let rest = &after_open[close_pos + 4..];
    let rest = rest.strip_prefix('\n').unwrap_or(rest);

    let mut description = None;
    let mut overrides: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    for line in frontmatter_block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(": ") {
            let key = key.trim();
            let value = value.trim();
            if key == "description" {
                description = Some(value.to_string());
            } else if let Some((k, v)) = value.split_once('=') {
                // target-namespaced: "cursor: key=value"
                overrides
                    .entry(key.to_string())
                    .or_default()
                    .push((k.to_string(), v.to_string()));
            }
        }
    }

    (description, overrides, rest.to_string())
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test parse::rules::tests
```

Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/parse/rules.rs
git commit -m "feat: .claude/rules/*.md parser"
```

---

### Task 7: Skills Parser

**Files:**
- Create: `src/parse/skills.rs`

- [ ] **Step 1: Write failing test**

```rust
use crate::ir::Skill;
use std::path::Path;

/// Parse a .claude/skills/*/SKILL.md file into a Skill.
pub fn parse_skill(path: &Path) -> Result<Skill, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_skill_with_full_frontmatter() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("explain-code");
        fs::create_dir_all(&skill_dir).unwrap();
        let path = skill_dir.join("SKILL.md");
        fs::write(
            &path,
            "---\nname: explain-code\ndescription: Explains code with diagrams\npaths: src/**/*.rs, tests/**\ndisable-model-invocation: true\ncursor: alwaysApply=false\n---\nWhen explaining code, include diagrams.\n",
        )
        .unwrap();
        let skill = parse_skill(&path).unwrap();
        assert_eq!(skill.name, "explain-code");
        assert_eq!(skill.description.as_deref(), Some("Explains code with diagrams"));
        assert_eq!(skill.paths, vec!["src/**/*.rs", "tests/**"]);
        assert!(skill.disable_model_invocation);
        assert!(skill.target_overrides.contains_key("cursor"));
        assert!(skill.body.contains("When explaining code"));
    }

    #[test]
    fn falls_back_to_directory_name() {
        let dir = tempfile::tempdir().unwrap();
        let skill_dir = dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let path = skill_dir.join("SKILL.md");
        fs::write(&path, "---\ndescription: does stuff\n---\nDo the thing\n").unwrap();
        let skill = parse_skill(&path).unwrap();
        assert_eq!(skill.name, "my-skill");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test parse::skills::tests
```

- [ ] **Step 3: Implement parse_skill**

```rust
pub fn parse_skill(path: &Path) -> Result<Skill, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let dir_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let (frontmatter, body) = split_frontmatter(&content);

    let mut name = dir_name.to_string();
    let mut description = None;
    let mut paths = Vec::new();
    let mut disable_model_invocation = false;
    let mut target_overrides: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();

    if let Some(fm) = frontmatter {
        for line in fm.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Some((key, value)) = line.split_once(": ") {
                let key = key.trim();
                let value = value.trim();
                match key {
                    "name" => name = value.to_string(),
                    "description" => description = Some(value.to_string()),
                    "paths" => {
                        paths = value.split(',').map(|s| s.trim().to_string()).collect();
                    }
                    "disable-model-invocation" => {
                        disable_model_invocation = value == "true";
                    }
                    _ => {
                        // Check for target-namespaced override
                        if let Some((k, v)) = value.split_once('=') {
                            target_overrides
                                .entry(key.to_string())
                                .or_default()
                                .push((k.to_string(), v.to_string()));
                        }
                    }
                }
            }
        }
    }

    Ok(Skill {
        name,
        description,
        paths,
        disable_model_invocation,
        target_overrides,
        body: body.trim().to_string(),
    })
}

fn split_frontmatter(content: &str) -> (Option<&str>, &str) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, content);
    }

    let after_open = &trimmed[3..];
    let after_open = after_open.strip_prefix('\n').unwrap_or(after_open);
    match after_open.find("\n---") {
        Some(pos) => {
            let fm = &after_open[..pos];
            let rest = &after_open[pos + 4..];
            let rest = rest.strip_prefix('\n').unwrap_or(rest);
            (Some(fm), rest)
        }
        None => (None, content),
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test parse::skills::tests
```

Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/parse/skills.rs
git commit -m "feat: SKILL.md parser with frontmatter mapping"
```

---

### Task 8: MCP Parser

**Files:**
- Create: `src/parse/mcp.rs`

- [ ] **Step 1: Write failing test**

```rust
use crate::ir::McpConfig;
use std::path::Path;

/// Parse .mcp.json into McpConfig.
pub fn parse_mcp(path: &Path) -> Result<McpConfig, String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_flat_mcp_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".mcp.json");
        fs::write(&path, r#"{"context7": {"command": "npx", "args": ["-y", "@upstash/context7-mcp"]}}"#).unwrap();
        let config = parse_mcp(&path).unwrap();
        assert!(config.servers.get("context7").is_some());
    }

    #[test]
    fn parses_wrapped_mcp_json() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".mcp.json");
        fs::write(&path, r#"{"mcpServers": {"linear": {"type": "http", "url": "https://mcp.linear.app/mcp"}}}"#).unwrap();
        let config = parse_mcp(&path).unwrap();
        assert!(config.servers.get("linear").is_some());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test parse::mcp::tests
```

- [ ] **Step 3: Implement parse_mcp**

```rust
pub fn parse_mcp(path: &Path) -> Result<McpConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {e}", path.display()))?;

    // If the top-level has "mcpServers", unwrap it
    let servers = if let Some(inner) = value.get("mcpServers") {
        inner.clone()
    } else {
        value
    };

    Ok(McpConfig { servers })
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test parse::mcp::tests
```

Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/parse/mcp.rs
git commit -m "feat: .mcp.json parser with mcpServers unwrapping"
```

---

### Task 9: Parse Module Orchestrator

**Files:**
- Modify: `src/parse/mod.rs`

- [ ] **Step 1: Write parse_all function**

```rust
pub mod claude_md;
pub mod mcp;
pub mod rules;
pub mod skills;

use crate::discover::Sources;
use crate::ir::ProjectConfig;

pub fn parse_all(sources: &Sources) -> Result<ProjectConfig, String> {
    let mut sections = Vec::new();

    if let Some(ref path) = sources.claude_md {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        sections.extend(claude_md::parse_claude_md(&content));
    }

    for path in &sources.rules {
        match rules::parse_rule(path) {
            Ok(section) => sections.push(section),
            Err(e) => crate::log::warn(&e),
        }
    }

    let mut skills = Vec::new();
    for path in &sources.skills {
        match skills::parse_skill(path) {
            Ok(skill) => skills.push(skill),
            Err(e) => crate::log::warn(&e),
        }
    }

    let mcp = match &sources.mcp_json {
        Some(path) => match mcp::parse_mcp(path) {
            Ok(config) => Some(config),
            Err(e) => {
                crate::log::warn(&e);
                None
            }
        },
        None => None,
    };

    Ok(ProjectConfig {
        sections,
        skills,
        mcp,
    })
}
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/parse/mod.rs
git commit -m "feat: parse orchestrator combining all source parsers"
```

---

### Task 10: Cursor Generator

**Files:**
- Create: `src/generate/cursor.rs`

- [ ] **Step 1: Write failing tests**

```rust
use crate::ir::{McpConfig, ProjectConfig, Section, SectionSource, Skill};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// A file to be written by the generator.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}

/// Generate Cursor config files from the project config.
pub fn generate(root: &Path, config: &ProjectConfig) -> Vec<GeneratedFile> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn section(title: Option<&str>, slug: &str, body: &str) -> Section {
        Section {
            title: title.map(String::from),
            slug: slug.to_string(),
            body: body.to_string(),
            source: SectionSource::ClaudeMd,
            target_overrides: HashMap::new(),
            description: title.map(String::from),
        }
    }

    #[test]
    fn generates_cursor_rules_from_sections() {
        let config = ProjectConfig {
            sections: vec![
                section(None, "_project", "Project preamble"),
                section(Some("Stack"), "stack", "Rust + Clap"),
            ],
            skills: vec![],
            mcp: None,
        };
        let files = generate(Path::new("/proj"), &config);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, PathBuf::from("/proj/.cursor/rules/_project.mdc"));
        assert!(files[0].content.contains("generated-by: agentic-sync"));
        assert!(files[0].content.contains("alwaysApply: true"));
        assert!(files[0].content.contains("Project preamble"));
        assert_eq!(files[1].path, PathBuf::from("/proj/.cursor/rules/stack.mdc"));
        assert!(files[1].content.contains("description: \"Stack\""));
    }

    #[test]
    fn applies_cursor_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "cursor".to_string(),
            vec![("alwaysApply".to_string(), "false".to_string())],
        );
        let config = ProjectConfig {
            sections: vec![Section {
                title: Some("Testing".to_string()),
                slug: "testing".to_string(),
                body: "Run tests".to_string(),
                source: SectionSource::ClaudeMd,
                target_overrides: overrides,
                description: Some("Testing".to_string()),
            }],
            skills: vec![],
            mcp: None,
        };
        let files = generate(Path::new("/proj"), &config);
        assert!(files[0].content.contains("alwaysApply: false"));
    }

    #[test]
    fn generates_cursor_skills() {
        let config = ProjectConfig {
            sections: vec![],
            skills: vec![Skill {
                name: "explain-code".to_string(),
                description: Some("Explains code".to_string()),
                paths: vec![],
                disable_model_invocation: false,
                target_overrides: HashMap::new(),
                body: "When explaining code, use diagrams.".to_string(),
            }],
            mcp: None,
        };
        let files = generate(Path::new("/proj"), &config);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].path,
            PathBuf::from("/proj/.cursor/skills/explain-code.mdc")
        );
        assert!(files[0].content.contains("alwaysApply: true"));
        assert!(files[0].content.contains("description: \"Explains code\""));
    }

    #[test]
    fn generates_mcp_json() {
        let config = ProjectConfig {
            sections: vec![],
            skills: vec![],
            mcp: Some(McpConfig {
                servers: serde_json::json!({
                    "context7": {"command": "npx", "args": ["-y", "@upstash/context7-mcp"]}
                }),
            }),
        };
        let files = generate(Path::new("/proj"), &config);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, PathBuf::from("/proj/.cursor/mcp.json"));
        assert!(files[0].content.contains("mcpServers"));
        assert!(files[0].content.contains("context7"));
    }

    #[test]
    fn disable_model_invocation_maps_to_always_apply_false() {
        let config = ProjectConfig {
            sections: vec![],
            skills: vec![Skill {
                name: "deploy".to_string(),
                description: Some("Deploy".to_string()),
                paths: vec![],
                disable_model_invocation: true,
                target_overrides: HashMap::new(),
                body: "Deploy steps".to_string(),
            }],
            mcp: None,
        };
        let files = generate(Path::new("/proj"), &config);
        assert!(files[0].content.contains("alwaysApply: false"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test generate::cursor::tests
```

- [ ] **Step 3: Implement generate**

```rust
pub fn generate(root: &Path, config: &ProjectConfig) -> Vec<GeneratedFile> {
    let mut files = Vec::new();

    // Sections → .cursor/rules/*.mdc
    for section in &config.sections {
        let path = root.join(format!(".cursor/rules/{}.mdc", section.slug));
        let content = render_mdc_rule(section);
        files.push(GeneratedFile { path, content });
    }

    // Skills → .cursor/skills/*.mdc
    for skill in &config.skills {
        let path = root.join(format!(".cursor/skills/{}.mdc", skill.name));
        let content = render_mdc_skill(skill);
        files.push(GeneratedFile { path, content });
    }

    // MCP → .cursor/mcp.json
    if let Some(ref mcp) = config.mcp {
        let path = root.join(".cursor/mcp.json");
        let wrapped = serde_json::json!({ "mcpServers": mcp.servers });
        let content = serde_json::to_string_pretty(&wrapped).unwrap_or_default();
        files.push(GeneratedFile { path, content });
    }

    files
}

fn render_mdc_rule(section: &Section) -> String {
    let mut always_apply = "true".to_string();
    let mut description = section
        .description
        .clone()
        .unwrap_or_default();
    let mut extra_fields = Vec::new();

    if let Some(overrides) = section.target_overrides.get("cursor") {
        for (key, value) in overrides {
            match key.as_str() {
                "alwaysApply" => always_apply = value.clone(),
                "description" => description = value.clone(),
                _ => extra_fields.push((key.clone(), value.clone())),
            }
        }
    }

    let mut frontmatter = format!(
        "generated-by: agentic-sync\nalwaysApply: {always_apply}"
    );
    if !description.is_empty() {
        frontmatter.push_str(&format!("\ndescription: \"{description}\""));
    }
    for (k, v) in &extra_fields {
        frontmatter.push_str(&format!("\n{k}: {v}"));
    }

    format!("---\n{frontmatter}\n---\n\n{}\n", section.body)
}

fn render_mdc_skill(skill: &Skill) -> String {
    let always_apply = if skill.disable_model_invocation {
        "false"
    } else {
        "true"
    };

    let mut description = skill.description.clone().unwrap_or_default();
    let mut extra_fields = Vec::new();
    let mut always_apply_override = None;

    if let Some(overrides) = skill.target_overrides.get("cursor") {
        for (key, value) in overrides {
            match key.as_str() {
                "alwaysApply" => always_apply_override = Some(value.clone()),
                "description" => description = value.clone(),
                _ => extra_fields.push((key.clone(), value.clone())),
            }
        }
    }

    let always_apply = always_apply_override
        .as_deref()
        .unwrap_or(always_apply);

    let mut frontmatter = format!(
        "generated-by: agentic-sync\nalwaysApply: {always_apply}"
    );
    if !description.is_empty() {
        frontmatter.push_str(&format!("\ndescription: \"{description}\""));
    }
    if !skill.paths.is_empty() {
        let globs = skill.paths.join(", ");
        frontmatter.push_str(&format!("\nglobs: {globs}"));
    }
    for (k, v) in &extra_fields {
        frontmatter.push_str(&format!("\n{k}: {v}"));
    }

    format!("---\n{frontmatter}\n---\n\n{}\n", skill.body)
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test generate::cursor::tests
```

Expected: PASS (5 tests).

- [ ] **Step 5: Commit**

```bash
git add src/generate/cursor.rs
git commit -m "feat: Cursor .mdc and mcp.json generator"
```

---

### Task 11: Copilot Generator

**Files:**
- Create: `src/generate/copilot.rs`

- [ ] **Step 1: Write failing test**

```rust
use std::path::{Path, PathBuf};

/// A file to be written by the generator.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}

/// Generate Copilot instructions from the raw CLAUDE.md content.
pub fn generate(root: &Path, claude_md_content: Option<&str>) -> Vec<GeneratedFile> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_copilot_instructions() {
        let files = generate(Path::new("/proj"), Some("## Stack\nRust\n\n## Testing\nCargo test\n"));
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].path,
            PathBuf::from("/proj/.github/copilot-instructions.md")
        );
        assert!(files[0].content.starts_with("<!-- Generated by agentic-sync"));
        assert!(files[0].content.contains("## Stack"));
        assert!(files[0].content.contains("Cargo test"));
    }

    #[test]
    fn no_claude_md_no_output() {
        let files = generate(Path::new("/proj"), None);
        assert!(files.is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test generate::copilot::tests
```

- [ ] **Step 3: Implement generate**

```rust
pub fn generate(root: &Path, claude_md_content: Option<&str>) -> Vec<GeneratedFile> {
    let Some(content) = claude_md_content else {
        return vec![];
    };

    let path = root.join(".github/copilot-instructions.md");
    let output = format!(
        "<!-- Generated by agentic-sync — do not edit manually -->\n\n{content}"
    );

    vec![GeneratedFile {
        path,
        content: output,
    }]
}
```

- [ ] **Step 4: Run tests to verify they pass**

```bash
cargo test generate::copilot::tests
```

Expected: PASS (2 tests).

- [ ] **Step 5: Commit**

```bash
git add src/generate/copilot.rs
git commit -m "feat: Copilot instructions.md generator"
```

---

### Task 12: Shared GeneratedFile Type & Generate Module

**Files:**
- Modify: `src/generate/mod.rs`
- Modify: `src/generate/cursor.rs`
- Modify: `src/generate/copilot.rs`

- [ ] **Step 1: Move GeneratedFile to generate/mod.rs and re-export**

`src/generate/mod.rs`:

```rust
pub mod copilot;
pub mod cursor;

use std::path::PathBuf;

/// A file to be written by a generator.
#[derive(Debug, Clone, PartialEq)]
pub struct GeneratedFile {
    pub path: PathBuf,
    pub content: String,
}
```

- [ ] **Step 2: Update cursor.rs and copilot.rs to use shared type**

In `src/generate/cursor.rs`, remove the local `GeneratedFile` definition and import:

```rust
use super::GeneratedFile;
```

In `src/generate/copilot.rs`, same change:

```rust
use super::GeneratedFile;
```

Update all references in both files to use the imported type.

- [ ] **Step 3: Run all tests to verify nothing broke**

```bash
cargo test
```

Expected: All tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/generate/
git commit -m "refactor: shared GeneratedFile type in generate module"
```

---

### Task 13: Output Module — Write, Check, Cleanup

**Files:**
- Create: `src/output.rs`

- [ ] **Step 1: Write failing tests**

```rust
use crate::generate::GeneratedFile;
use std::path::{Path, PathBuf};

const GENERATED_BY_MARKER: &str = "generated-by: agentic-sync";
const GENERATED_BY_HTML_MARKER: &str = "<!-- Generated by agentic-sync";

/// Result of a check or fix operation.
#[derive(Debug, Default)]
pub struct OutputResult {
    pub stale: Vec<PathBuf>,
    pub written: Vec<PathBuf>,
    pub skipped: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub warnings: Vec<String>,
}

impl OutputResult {
    pub fn is_in_sync(&self) -> bool {
        self.stale.is_empty() && self.skipped.is_empty()
    }
}

/// Check if generated files match what's on disk.
pub fn check(files: &[GeneratedFile]) -> OutputResult {
    todo!()
}

/// Write generated files to disk. Respects generated-by ownership.
pub fn fix(files: &[GeneratedFile], overwrite: bool) -> OutputResult {
    todo!()
}

/// Delete stale generated files that no longer have a source.
pub fn cleanup(root: &Path, current_files: &[GeneratedFile]) -> Vec<PathBuf> {
    todo!()
}

/// Check if a file on disk has the generated-by marker.
fn has_generated_by_marker(path: &Path) -> bool {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn check_detects_stale_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".cursor/rules/stack.mdc");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "---\ngenerated-by: agentic-sync\nalwaysApply: true\n---\nOld content\n").unwrap();

        let files = vec![GeneratedFile {
            path: path.clone(),
            content: "---\ngenerated-by: agentic-sync\nalwaysApply: true\n---\nNew content\n".to_string(),
        }];
        let result = check(&files);
        assert_eq!(result.stale.len(), 1);
    }

    #[test]
    fn check_reports_in_sync() {
        let dir = tempfile::tempdir().unwrap();
        let content = "---\ngenerated-by: agentic-sync\nalwaysApply: true\n---\nContent\n";
        let path = dir.path().join(".cursor/rules/stack.mdc");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();

        let files = vec![GeneratedFile {
            path: path.clone(),
            content: content.to_string(),
        }];
        let result = check(&files);
        assert!(result.is_in_sync());
    }

    #[test]
    fn check_new_file_is_stale() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".cursor/rules/new.mdc");
        let files = vec![GeneratedFile {
            path,
            content: "---\ngenerated-by: agentic-sync\n---\nNew\n".to_string(),
        }];
        let result = check(&files);
        assert_eq!(result.stale.len(), 1);
    }

    #[test]
    fn fix_writes_new_files() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".cursor/rules/stack.mdc");
        let files = vec![GeneratedFile {
            path: path.clone(),
            content: "---\ngenerated-by: agentic-sync\n---\nContent\n".to_string(),
        }];
        let result = fix(&files, false);
        assert_eq!(result.written.len(), 1);
        assert!(path.exists());
        assert!(fs::read_to_string(&path).unwrap().contains("Content"));
    }

    #[test]
    fn fix_skips_unowned_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".cursor/rules/manual.mdc");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "---\nalwaysApply: true\n---\nHand-written\n").unwrap();

        let files = vec![GeneratedFile {
            path: path.clone(),
            content: "---\ngenerated-by: agentic-sync\n---\nGenerated\n".to_string(),
        }];
        let result = fix(&files, false);
        assert_eq!(result.skipped.len(), 1);
        assert!(fs::read_to_string(&path).unwrap().contains("Hand-written"));
    }

    #[test]
    fn fix_overwrites_unowned_when_forced() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".cursor/rules/manual.mdc");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "---\nalwaysApply: true\n---\nHand-written\n").unwrap();

        let files = vec![GeneratedFile {
            path: path.clone(),
            content: "---\ngenerated-by: agentic-sync\n---\nGenerated\n".to_string(),
        }];
        let result = fix(&files, true);
        assert_eq!(result.written.len(), 1);
        assert!(fs::read_to_string(&path).unwrap().contains("Generated"));
    }

    #[test]
    fn cleanup_deletes_stale_generated_files() {
        let dir = tempfile::tempdir().unwrap();
        let rules_dir = dir.path().join(".cursor/rules");
        fs::create_dir_all(&rules_dir).unwrap();
        // This file is generated but no longer has a source
        fs::write(
            rules_dir.join("old.mdc"),
            "---\ngenerated-by: agentic-sync\n---\nOld\n",
        )
        .unwrap();
        // This file is hand-written — should not be deleted
        fs::write(
            rules_dir.join("manual.mdc"),
            "---\nalwaysApply: true\n---\nManual\n",
        )
        .unwrap();

        let current: Vec<GeneratedFile> = vec![]; // no current generated files
        let deleted = cleanup(dir.path(), &current);
        assert_eq!(deleted.len(), 1);
        assert!(deleted[0].ends_with("old.mdc"));
        assert!(rules_dir.join("manual.mdc").exists());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

```bash
cargo test output::tests
```

- [ ] **Step 3: Implement has_generated_by_marker**

```rust
fn has_generated_by_marker(path: &Path) -> bool {
    let Ok(content) = std::fs::read_to_string(path) else {
        return false;
    };
    content.contains(GENERATED_BY_MARKER) || content.contains(GENERATED_BY_HTML_MARKER)
}
```

- [ ] **Step 4: Implement check**

```rust
pub fn check(files: &[GeneratedFile]) -> OutputResult {
    let mut result = OutputResult::default();

    for file in files {
        if !file.path.exists() {
            result.stale.push(file.path.clone());
            continue;
        }

        let Ok(existing) = std::fs::read_to_string(&file.path) else {
            result.stale.push(file.path.clone());
            continue;
        };

        if existing != file.content {
            if has_generated_by_marker(&file.path) {
                result.stale.push(file.path.clone());
            } else {
                result.warnings.push(format!(
                    "{}: exists without generated-by marker, would conflict",
                    file.path.display()
                ));
                result.skipped.push(file.path.clone());
            }
        }
    }

    result
}
```

- [ ] **Step 5: Implement fix**

```rust
pub fn fix(files: &[GeneratedFile], overwrite: bool) -> OutputResult {
    let mut result = OutputResult::default();

    for file in files {
        if file.path.exists() && !overwrite && !has_generated_by_marker(&file.path) {
            crate::log::warn_file(
                &file.path,
                "exists without generated-by marker, skipping (use --overwrite to force)",
            );
            result.skipped.push(file.path.clone());
            continue;
        }

        if let Some(parent) = file.path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    crate::log::error_file(&file.path, &format!("failed to create directory: {e}"));
                    continue;
                }
            }
        }

        match std::fs::write(&file.path, &file.content) {
            Ok(()) => result.written.push(file.path.clone()),
            Err(e) => {
                crate::log::error_file(&file.path, &format!("failed to write: {e}"));
            }
        }
    }

    result
}
```

- [ ] **Step 6: Implement cleanup**

```rust
pub fn cleanup(root: &Path, current_files: &[GeneratedFile]) -> Vec<PathBuf> {
    let current_paths: std::collections::HashSet<&Path> =
        current_files.iter().map(|f| f.path.as_path()).collect();

    let mut deleted = Vec::new();

    let dirs_to_scan = [
        root.join(".cursor/rules"),
        root.join(".cursor/skills"),
    ];

    for dir in &dirs_to_scan {
        if !dir.is_dir() {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            if current_paths.contains(path.as_path()) {
                continue;
            }
            if has_generated_by_marker(&path) {
                if std::fs::remove_file(&path).is_ok() {
                    crate::log::info(&format!("Deleted stale generated file: {}", path.display()));
                    deleted.push(path);
                }
            }
        }
    }

    deleted
}
```

- [ ] **Step 7: Run tests to verify they pass**

```bash
cargo test output::tests
```

Expected: PASS (7 tests).

- [ ] **Step 8: Commit**

```bash
git add src/output.rs
git commit -m "feat: output module with check, fix, cleanup, and ownership tracking"
```

---

### Task 14: Wire Up the Run Function

**Files:**
- Modify: `src/lib.rs`

- [ ] **Step 1: Implement the run function**

Replace the stub `run` in `src/lib.rs`:

```rust
pub fn run(root: &Path, mode: Mode, targets: &[Target]) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let root = std::fs::canonicalize(root)
        .map_err(|e| format!("Invalid path '{}': {e}", root.display()))?;

    let sources = discover::discover(&root);

    // Early exit if nothing to sync
    if sources.claude_md.is_none()
        && sources.rules.is_empty()
        && sources.skills.is_empty()
        && sources.mcp_json.is_none()
    {
        log::info("No Claude config found, nothing to sync.");
        return Ok(ExitCode::SUCCESS);
    }

    let config = parse::parse_all(&sources).map_err(|e| {
        log::error(&e);
        e
    })?;

    // Read raw CLAUDE.md for Copilot (verbatim copy)
    let claude_md_content = sources
        .claude_md
        .as_ref()
        .and_then(|p| std::fs::read_to_string(p).ok());

    // Generate files for requested targets
    let mut generated = Vec::new();

    for target in targets {
        match target {
            Target::Cursor => {
                generated.extend(generate::cursor::generate(&root, &config));
            }
            Target::Copilot => {
                generated.extend(generate::copilot::generate(
                    &root,
                    claude_md_content.as_deref(),
                ));
            }
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

                // Show diff if file exists
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
```

- [ ] **Step 2: Verify it compiles**

```bash
cargo build
```

- [ ] **Step 3: Commit**

```bash
git add src/lib.rs
git commit -m "feat: wire up run function with check, fix, and pr modes"
```

---

### Task 15: Integration Tests

**Files:**
- Create: `tests/integration/mod.rs`
- Create: `tests/integration/check_test.rs`
- Create: `tests/integration/fix_test.rs`
- Create: `tests/integration/cleanup_test.rs`

- [ ] **Step 1: Create integration test harness**

`tests/integration/mod.rs`:

```rust
mod check_test;
mod cleanup_test;
mod fix_test;

use std::fs;
use std::path::Path;

/// Set up a project directory with standard Claude config.
pub fn setup_basic_project(root: &Path) {
    fs::write(
        root.join("CLAUDE.md"),
        "Project preamble\n\n## Stack\nRust + Clap\n\n## Testing\nUse cargo test\n",
    )
    .unwrap();
}

pub fn setup_full_project(root: &Path) {
    setup_basic_project(root);

    // Rules
    fs::create_dir_all(root.join(".claude/rules")).unwrap();
    fs::write(
        root.join(".claude/rules/style.md"),
        "---\ndescription: Code style\n---\nUse 4-space indentation\n",
    )
    .unwrap();

    // Skills
    fs::create_dir_all(root.join(".claude/skills/explain-code")).unwrap();
    fs::write(
        root.join(".claude/skills/explain-code/SKILL.md"),
        "---\nname: explain-code\ndescription: Explains code\n---\nUse diagrams when explaining.\n",
    )
    .unwrap();

    // MCP
    fs::write(
        root.join(".mcp.json"),
        r#"{"context7": {"command": "npx", "args": ["-y", "@upstash/context7-mcp"]}}"#,
    )
    .unwrap();
}
```

- [ ] **Step 2: Write check integration test**

`tests/integration/check_test.rs`:

```rust
use std::process::ExitCode;

#[test]
fn check_reports_stale_on_fresh_project() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_basic_project(dir.path());

    let result = agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Check,
        &agentic_sync::all_targets(),
    )
    .unwrap();

    // Files don't exist yet, so they're stale
    assert_eq!(result, ExitCode::from(1));
}

#[test]
fn check_passes_after_fix() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_basic_project(dir.path());

    // Fix first
    agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &agentic_sync::all_targets(),
    )
    .unwrap();

    // Then check
    let result = agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Check,
        &agentic_sync::all_targets(),
    )
    .unwrap();

    assert_eq!(result, ExitCode::SUCCESS);
}
```

- [ ] **Step 3: Write fix integration test**

`tests/integration/fix_test.rs`:

```rust
use std::fs;

#[test]
fn fix_generates_all_target_files() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_full_project(dir.path());

    let result = agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &agentic_sync::all_targets(),
    )
    .unwrap();

    assert_eq!(result, std::process::ExitCode::SUCCESS);

    // Cursor rules
    assert!(dir.path().join(".cursor/rules/_project.mdc").exists());
    assert!(dir.path().join(".cursor/rules/stack.mdc").exists());
    assert!(dir.path().join(".cursor/rules/testing.mdc").exists());
    assert!(dir.path().join(".cursor/rules/style.mdc").exists());

    // Cursor skills
    assert!(dir.path().join(".cursor/skills/explain-code.mdc").exists());

    // Cursor MCP
    let mcp_content = fs::read_to_string(dir.path().join(".cursor/mcp.json")).unwrap();
    assert!(mcp_content.contains("mcpServers"));

    // Copilot
    let copilot = fs::read_to_string(dir.path().join(".github/copilot-instructions.md")).unwrap();
    assert!(copilot.contains("Generated by agentic-sync"));
    assert!(copilot.contains("## Stack"));
}

#[test]
fn fix_with_target_filter() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_basic_project(dir.path());

    agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &[agentic_sync::Target::Cursor],
    )
    .unwrap();

    assert!(dir.path().join(".cursor/rules/stack.mdc").exists());
    assert!(!dir.path().join(".github/copilot-instructions.md").exists());
}
```

- [ ] **Step 4: Write cleanup integration test**

`tests/integration/cleanup_test.rs`:

```rust
use std::fs;

#[test]
fn cleanup_removes_stale_generated_files() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_full_project(dir.path());

    // Generate everything
    agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &agentic_sync::all_targets(),
    )
    .unwrap();

    // Verify stack.mdc exists
    assert!(dir.path().join(".cursor/rules/stack.mdc").exists());

    // Remove ## Stack from CLAUDE.md
    fs::write(
        dir.path().join("CLAUDE.md"),
        "Project preamble\n\n## Testing\nUse cargo test\n",
    )
    .unwrap();

    // Fix again — should delete stack.mdc
    agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &agentic_sync::all_targets(),
    )
    .unwrap();

    assert!(!dir.path().join(".cursor/rules/stack.mdc").exists());
    assert!(dir.path().join(".cursor/rules/testing.mdc").exists());
}
```

- [ ] **Step 5: Run all integration tests**

```bash
cargo test --test integration
```

Expected: PASS (4 tests). If there are compilation issues with the test structure, you may need to adjust the test entrypoint. Create `tests/integration.rs` if needed:

```rust
mod integration;
```

- [ ] **Step 6: Run the full test suite**

```bash
cargo test
```

Expected: All unit + integration tests pass.

- [ ] **Step 7: Commit**

```bash
git add tests/
git commit -m "test: integration tests for check, fix, and cleanup"
```

---

### Task 16: Manual Smoke Test & hk Config

**Files:**
- Create: `hk.pkl` (project root)

- [ ] **Step 1: Create a test CLAUDE.md in the project itself**

```bash
cat > /Users/adam/src/agentic.md/CLAUDE.md << 'EOF'
# agentic-sync

Rust CLI that syncs Claude Code config to Cursor and Copilot.

## Build

```
cargo build --release
```

## Testing

```
cargo test
```

## Code Style

Use `cargo fmt` and `cargo clippy` before committing.
EOF
```

- [ ] **Step 2: Run the tool against itself**

```bash
cargo run -- --fix
```

Verify files were created:
```bash
ls -la .cursor/rules/
ls -la .github/
cat .cursor/rules/build.mdc
cat .github/copilot-instructions.md
```

- [ ] **Step 3: Run check mode to verify sync**

```bash
cargo run -- --check
```

Expected: Exit 0, "All files in sync."

- [ ] **Step 4: Create hk.pkl**

```pkl
amends "package://github.com/jdx/hk/releases/download/v1.18.1/hk@1.18.1#/Config.pkl"

local linters = new Mapping<String, Step> {
    ["agentic-sync"] {
        glob = List("CLAUDE.md", ".claude/**", ".mcp.json")
        check = "cargo run -- --check"
        fix = "cargo run -- --fix"
    }
    ["cargo-fmt"] {
        glob = List("**/*.rs")
        check = "cargo fmt --all -- --check"
        fix = "cargo fmt --all"
    }
}

hooks {
    ["fix"] {
        fix = true
        steps = linters
    }
    ["check"] {
        steps = linters
    }
}
```

Note: Replace `cargo run --` with the installed binary path once published. For local dev, `cargo run --` works.

- [ ] **Step 5: Verify hk integration**

```bash
hk check
hk fix
```

- [ ] **Step 6: Commit**

```bash
git add hk.pkl CLAUDE.md .cursor/ .github/
git commit -m "feat: hk integration and self-hosting smoke test"
```

---

### Task 17: PR Mode Polish

**Files:**
- Modify: `src/lib.rs` (the `Mode::Pr` branch is already implemented in Task 14)

- [ ] **Step 1: Write a test for PR output format**

Add to `tests/integration/check_test.rs`:

```rust
#[test]
fn pr_mode_outputs_markdown() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_basic_project(dir.path());

    // Capture stdout would require refactoring run() to accept a writer.
    // For now, just verify it exits 1 (stale) and doesn't panic.
    let result = agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Pr,
        &agentic_sync::all_targets(),
    )
    .unwrap();

    assert_eq!(result, std::process::ExitCode::from(1));
}
```

- [ ] **Step 2: Run the test**

```bash
cargo test --test integration
```

Expected: PASS.

- [ ] **Step 3: Manually verify PR output**

```bash
cargo run -- --pr
```

Expected: Markdown output listing stale files with "(new file)" notes.

- [ ] **Step 4: Commit**

```bash
git add tests/
git commit -m "test: PR mode integration test"
```

---

### Task 18: Final Cleanup & Release Build

- [ ] **Step 1: Run cargo fmt**

```bash
cargo fmt
```

- [ ] **Step 2: Run cargo clippy**

```bash
cargo clippy -- -D warnings
```

Fix any warnings.

- [ ] **Step 3: Run full test suite**

```bash
cargo test
```

Expected: All tests pass.

- [ ] **Step 4: Build release binary**

```bash
cargo build --release
ls -la target/release/agentic-sync
```

- [ ] **Step 5: Test release binary**

```bash
./target/release/agentic-sync --check
./target/release/agentic-sync --help
```

- [ ] **Step 6: Commit any formatting fixes**

```bash
git add -A
git commit -m "chore: fmt + clippy cleanup"
```

---

Plan complete and saved to `docs/superpowers/plans/2026-04-09-agentic-sync.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
