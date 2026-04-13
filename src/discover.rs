use crate::Prefer;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Sources {
    pub claude_md: Option<PathBuf>,
    pub rules: Vec<PathBuf>,
    pub skills: Vec<PathBuf>,
    pub mcp_json: Option<PathBuf>,
}

pub fn discover(root: &Path, prefer: Prefer) -> Sources {
    let mut sources = Sources::default();

    // Discovery order depends on preference
    let priority: &[&str] = match prefer {
        Prefer::Claude => &["CLAUDE.md", "AGENTS.md", "AGENT.md"],
        Prefer::Agents => &["AGENTS.md", "AGENT.md", "CLAUDE.md"],
    };
    for name in priority {
        let path = root.join(name);
        if path.is_file() {
            sources.claude_md = Some(path);
            break;
        }
    }

    let rules_dir = root.join(".claude/rules");
    if rules_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&rules_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") && path.is_file() {
                sources.rules.push(path);
            }
        }
        sources.rules.sort();
    }

    let skills_dir = root.join(".claude/skills");
    if skills_dir.is_dir()
        && let Ok(entries) = std::fs::read_dir(&skills_dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            let skill_md = path.join("SKILL.md");
            if path.is_dir() && skill_md.is_file() {
                sources.skills.push(skill_md);
            }
        }
        sources.skills.sort();
    }

    let mcp = root.join(".mcp.json");
    if mcp.is_file() {
        sources.mcp_json = Some(mcp);
    }

    sources
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn discovers_all_sources() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create CLAUDE.md
        fs::write(root.join("CLAUDE.md"), "# Project").unwrap();

        // Create .claude/rules/testing.md
        fs::create_dir_all(root.join(".claude/rules")).unwrap();
        fs::write(root.join(".claude/rules/testing.md"), "# Testing").unwrap();

        // Create .claude/skills/explain-code/SKILL.md
        fs::create_dir_all(root.join(".claude/skills/explain-code")).unwrap();
        fs::write(
            root.join(".claude/skills/explain-code/SKILL.md"),
            "# Explain",
        )
        .unwrap();

        // Create .mcp.json
        fs::write(root.join(".mcp.json"), "{}").unwrap();

        let sources = discover(root, Prefer::Claude);

        assert!(sources.claude_md.is_some());
        assert_eq!(sources.rules.len(), 1);
        assert!(sources.rules[0].ends_with("testing.md"));
        assert_eq!(sources.skills.len(), 1);
        assert!(sources.skills[0].ends_with("SKILL.md"));
        assert!(sources.mcp_json.is_some());
    }

    #[test]
    fn discovers_agents_md() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join("AGENTS.md"), "# Agents").unwrap();
        let sources = discover(root, Prefer::Claude);
        assert!(sources.claude_md.is_some());
        assert!(sources.claude_md.unwrap().ends_with("AGENTS.md"));
    }

    #[test]
    fn discovers_agent_md() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join("AGENT.md"), "# Agent").unwrap();
        let sources = discover(root, Prefer::Claude);
        assert!(sources.claude_md.is_some());
        assert!(sources.claude_md.unwrap().ends_with("AGENT.md"));
    }

    #[test]
    fn claude_md_takes_priority_over_agents_md() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join("CLAUDE.md"), "# Claude").unwrap();
        fs::write(root.join("AGENTS.md"), "# Agents").unwrap();
        let sources = discover(root, Prefer::Claude);
        assert!(sources.claude_md.unwrap().ends_with("CLAUDE.md"));
    }

    #[test]
    fn prefer_agents_reverses_priority() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join("CLAUDE.md"), "# Claude").unwrap();
        fs::write(root.join("AGENTS.md"), "# Agents").unwrap();
        let sources = discover(root, Prefer::Agents);
        assert!(sources.claude_md.unwrap().ends_with("AGENTS.md"));
    }

    #[test]
    fn prefer_agents_falls_back_to_claude() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join("CLAUDE.md"), "# Claude").unwrap();
        let sources = discover(root, Prefer::Agents);
        assert!(sources.claude_md.unwrap().ends_with("CLAUDE.md"));
    }

    #[test]
    fn empty_dir_returns_empty_sources() {
        let dir = TempDir::new().unwrap();
        let sources = discover(dir.path(), Prefer::Claude);

        assert!(sources.claude_md.is_none());
        assert!(sources.rules.is_empty());
        assert!(sources.skills.is_empty());
        assert!(sources.mcp_json.is_none());
    }
}
