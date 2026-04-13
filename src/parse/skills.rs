use crate::ir::Skill;
use std::collections::HashMap;
use std::path::Path;

pub fn parse_skill(path: &Path) -> Result<Skill, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let dir_name = path
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let (fm_fields, overrides, body) = parse_frontmatter(&content);

    let name = fm_fields
        .get("name")
        .cloned()
        .unwrap_or_else(|| dir_name.to_string());

    let description = fm_fields.get("description").cloned();

    let paths = fm_fields
        .get("paths")
        .map(|p| p.split(',').map(|s| s.trim().to_string()).collect())
        .unwrap_or_default();

    let disable_model_invocation = fm_fields
        .get("disable-model-invocation")
        .is_some_and(|v| v == "true");

    Ok(Skill {
        name,
        description,
        paths,
        disable_model_invocation,
        target_overrides: overrides,
        body: body.trim().to_string(),
    })
}

type TargetOverrides = HashMap<String, Vec<(String, String)>>;

/// Parse YAML-style frontmatter from a SKILL.md file.
/// Returns (field_map, target_overrides, remaining_body).
fn parse_frontmatter(content: &str) -> (HashMap<String, String>, TargetOverrides, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (HashMap::new(), HashMap::new(), content.to_string());
    }

    let after_open = &trimmed[3..];
    let after_open = after_open.trim_start_matches('-');
    let after_first_newline = match after_open.find('\n') {
        Some(i) => &after_open[i + 1..],
        None => return (HashMap::new(), HashMap::new(), content.to_string()),
    };

    let close_pos = after_first_newline.find("\n---");
    let (fm_block, rest) = match close_pos {
        Some(pos) => {
            let fm = &after_first_newline[..pos];
            let after_close = &after_first_newline[pos + 4..];
            let after_close = after_close.trim_start_matches('-');
            let after_close = after_close.strip_prefix('\n').unwrap_or(after_close);
            (fm, after_close.to_string())
        }
        None => return (HashMap::new(), HashMap::new(), content.to_string()),
    };

    let known_fields = ["name", "description", "paths", "disable-model-invocation"];
    let mut fields = HashMap::new();
    let mut overrides: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for line in fm_block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if known_fields.contains(&key) {
                fields.insert(key.to_string(), value.to_string());
            } else {
                // target: key=value override
                if let Some((k, v)) = value.split_once('=') {
                    overrides
                        .entry(key.to_string())
                        .or_default()
                        .push((k.trim().to_string(), v.trim().to_string()));
                }
            }
        }
    }

    (fields, overrides, rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parses_skill_with_full_frontmatter() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("explain-code");
        fs::create_dir_all(&skill_dir).unwrap();
        let path = skill_dir.join("SKILL.md");
        fs::write(
            &path,
            "---\nname: Explain Code\ndescription: Explains code in detail\npaths: src/**/*.rs, tests/**/*.rs\ndisable-model-invocation: true\ncursor: shortcut=ctrl+e\n---\nExplain the selected code.",
        )
        .unwrap();

        let skill = parse_skill(&path).unwrap();
        assert_eq!(skill.name, "Explain Code");
        assert_eq!(
            skill.description,
            Some("Explains code in detail".to_string())
        );
        assert_eq!(skill.paths, vec!["src/**/*.rs", "tests/**/*.rs"]);
        assert!(skill.disable_model_invocation);
        let cursor = skill.target_overrides.get("cursor").unwrap();
        assert_eq!(cursor[0], ("shortcut".to_string(), "ctrl+e".to_string()));
        assert_eq!(skill.body, "Explain the selected code.");
    }

    #[test]
    fn falls_back_to_directory_name() {
        let dir = TempDir::new().unwrap();
        let skill_dir = dir.path().join("my-skill");
        fs::create_dir_all(&skill_dir).unwrap();
        let path = skill_dir.join("SKILL.md");
        fs::write(&path, "Just some instructions.").unwrap();

        let skill = parse_skill(&path).unwrap();
        assert_eq!(skill.name, "my-skill");
        assert!(skill.description.is_none());
        assert!(skill.paths.is_empty());
        assert!(!skill.disable_model_invocation);
        assert_eq!(skill.body, "Just some instructions.");
    }
}
