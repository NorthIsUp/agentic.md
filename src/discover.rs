use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct Sources {
    pub claude_md: Option<PathBuf>,
    pub rules: Vec<PathBuf>,
    pub skills: Vec<PathBuf>,
    pub mcp_json: Option<PathBuf>,
}

pub fn discover(root: &Path) -> Sources {
    let mut sources = Sources::default();

    let claude_md = root.join("CLAUDE.md");
    if claude_md.is_file() {
        sources.claude_md = Some(claude_md);
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

        let sources = discover(root);

        assert!(sources.claude_md.is_some());
        assert_eq!(sources.rules.len(), 1);
        assert!(sources.rules[0].ends_with("testing.md"));
        assert_eq!(sources.skills.len(), 1);
        assert!(sources.skills[0].ends_with("SKILL.md"));
        assert!(sources.mcp_json.is_some());
    }

    #[test]
    fn empty_dir_returns_empty_sources() {
        let dir = TempDir::new().unwrap();
        let sources = discover(dir.path());

        assert!(sources.claude_md.is_none());
        assert!(sources.rules.is_empty());
        assert!(sources.skills.is_empty());
        assert!(sources.mcp_json.is_none());
    }
}
