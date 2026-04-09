use std::path::Path;

use crate::ir::ProjectConfig;

use super::GeneratedFile;

pub fn generate(root: &Path, config: &ProjectConfig) -> Vec<GeneratedFile> {
    let mut files = Vec::new();

    for section in &config.sections {
        let mut always_apply = "true".to_string();
        let mut description = section
            .description
            .clone()
            .or_else(|| section.title.clone())
            .unwrap_or_default();

        // Apply cursor overrides
        if let Some(overrides) = section.target_overrides.get("cursor") {
            for (key, value) in overrides {
                match key.as_str() {
                    "alwaysApply" => always_apply = value.clone(),
                    "description" => description = value.clone(),
                    _ => {}
                }
            }
        }

        let mut content = String::new();
        content.push_str("---\n");
        content.push_str("generated-by: agentic-sync\n");
        content.push_str(&format!("alwaysApply: {always_apply}\n"));
        if !description.is_empty() {
            content.push_str(&format!("description: {description}\n"));
        }
        content.push_str("---\n\n");
        content.push_str(&section.body);
        content.push('\n');

        let path = root.join(format!(".cursor/rules/{}.mdc", section.slug));
        files.push(GeneratedFile { path, content });
    }

    for skill in &config.skills {
        let mut always_apply = if skill.disable_model_invocation {
            "false".to_string()
        } else {
            "true".to_string()
        };
        let mut description = skill.description.clone().unwrap_or_default();
        let mut globs = if skill.paths.is_empty() {
            String::new()
        } else {
            skill.paths.join(", ")
        };

        // Apply cursor overrides
        if let Some(overrides) = skill.target_overrides.get("cursor") {
            for (key, value) in overrides {
                match key.as_str() {
                    "alwaysApply" => always_apply = value.clone(),
                    "description" => description = value.clone(),
                    "globs" => globs = value.clone(),
                    _ => {}
                }
            }
        }

        let mut content = String::new();
        content.push_str("---\n");
        content.push_str("generated-by: agentic-sync\n");
        content.push_str(&format!("alwaysApply: {always_apply}\n"));
        if !description.is_empty() {
            content.push_str(&format!("description: {description}\n"));
        }
        if !globs.is_empty() {
            content.push_str(&format!("globs: {globs}\n"));
        }
        content.push_str("---\n\n");
        content.push_str(&skill.body);
        content.push('\n');

        let path = root.join(format!(".cursor/skills/{}.mdc", skill.name));
        files.push(GeneratedFile { path, content });
    }

    if let Some(ref mcp) = config.mcp {
        let wrapper = serde_json::json!({ "mcpServers": mcp.servers });
        let content = serde_json::to_string_pretty(&wrapper).unwrap_or_default();
        let path = root.join(".cursor/mcp.json");
        files.push(GeneratedFile {
            path,
            content: content + "\n",
        });
    }

    files
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{McpConfig, Section, SectionSource, Skill};
    use std::collections::HashMap;
    use tempfile::TempDir;

    #[test]
    fn generates_cursor_rules_from_sections() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let config = ProjectConfig {
            sections: vec![
                Section {
                    title: Some("Stack".to_string()),
                    slug: "stack".to_string(),
                    body: "Use Rust.".to_string(),
                    source: SectionSource::ClaudeMd,
                    target_overrides: HashMap::new(),
                    description: None,
                },
                Section {
                    title: Some("Testing".to_string()),
                    slug: "testing".to_string(),
                    body: "Run cargo test.".to_string(),
                    source: SectionSource::ClaudeMd,
                    target_overrides: HashMap::new(),
                    description: Some("Testing guidelines".to_string()),
                },
            ],
            skills: vec![],
            mcp: None,
        };

        let files = generate(root, &config);
        assert_eq!(files.len(), 2);

        assert_eq!(files[0].path, root.join(".cursor/rules/stack.mdc"));
        assert!(files[0].content.contains("generated-by: agentic-sync"));
        assert!(files[0].content.contains("alwaysApply: true"));
        assert!(files[0].content.contains("description: Stack"));
        assert!(files[0].content.contains("Use Rust."));

        assert_eq!(files[1].path, root.join(".cursor/rules/testing.mdc"));
        assert!(files[1].content.contains("generated-by: agentic-sync"));
        assert!(files[1].content.contains("alwaysApply: true"));
        assert!(files[1].content.contains("description: Testing guidelines"));
        assert!(files[1].content.contains("Run cargo test."));
    }

    #[test]
    fn applies_cursor_overrides() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let mut overrides = HashMap::new();
        overrides.insert(
            "cursor".to_string(),
            vec![("alwaysApply".to_string(), "false".to_string())],
        );

        let config = ProjectConfig {
            sections: vec![Section {
                title: Some("Rules".to_string()),
                slug: "rules".to_string(),
                body: "Some rules.".to_string(),
                source: SectionSource::ClaudeMd,
                target_overrides: overrides,
                description: None,
            }],
            skills: vec![],
            mcp: None,
        };

        let files = generate(root, &config);
        assert_eq!(files.len(), 1);
        assert!(files[0].content.contains("alwaysApply: false"));
    }

    #[test]
    fn generates_cursor_skills() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let config = ProjectConfig {
            sections: vec![],
            skills: vec![Skill {
                name: "explain-code".to_string(),
                description: Some("Explains code".to_string()),
                paths: vec!["src/**/*.rs".to_string()],
                disable_model_invocation: false,
                target_overrides: HashMap::new(),
                body: "Explain the code.".to_string(),
            }],
            mcp: None,
        };

        let files = generate(root, &config);
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0].path,
            root.join(".cursor/skills/explain-code.mdc")
        );
        assert!(files[0].content.contains("generated-by: agentic-sync"));
        assert!(files[0].content.contains("alwaysApply: true"));
        assert!(files[0].content.contains("description: Explains code"));
        assert!(files[0].content.contains("globs: src/**/*.rs"));
        assert!(files[0].content.contains("Explain the code."));
    }

    #[test]
    fn generates_mcp_json() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let servers = serde_json::json!({
            "my-server": {
                "command": "npx",
                "args": ["-y", "my-server"]
            }
        });

        let config = ProjectConfig {
            sections: vec![],
            skills: vec![],
            mcp: Some(McpConfig { servers }),
        };

        let files = generate(root, &config);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, root.join(".cursor/mcp.json"));

        let parsed: serde_json::Value = serde_json::from_str(&files[0].content).unwrap();
        assert!(parsed.get("mcpServers").is_some());
        assert!(parsed["mcpServers"]["my-server"]["command"]
            .as_str()
            .unwrap()
            == "npx");
    }

    #[test]
    fn disable_model_invocation_maps_to_always_apply_false() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        let config = ProjectConfig {
            sections: vec![],
            skills: vec![Skill {
                name: "auto-format".to_string(),
                description: None,
                paths: vec![],
                disable_model_invocation: true,
                target_overrides: HashMap::new(),
                body: "Format on save.".to_string(),
            }],
            mcp: None,
        };

        let files = generate(root, &config);
        assert_eq!(files.len(), 1);
        assert!(files[0].content.contains("alwaysApply: false"));
    }
}
