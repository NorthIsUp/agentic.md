use crate::ir::{Section, SectionSource};
use std::collections::HashMap;
use std::path::Path;

pub fn parse_rule(path: &Path) -> Result<Section, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let filename = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");

    let (description, overrides, body) = parse_frontmatter(&content);

    let trimmed_body = body.trim().to_string();

    Ok(Section {
        title: Some(filename.to_string()),
        slug: slugify(filename),
        body: trimmed_body,
        source: SectionSource::Rules,
        target_overrides: overrides,
        description,
    })
}

fn slugify(text: &str) -> String {
    let mut result = String::new();
    for ch in text.chars() {
        if ch.is_alphanumeric() {
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push('-');
        }
    }
    // Collapse consecutive dashes
    let mut collapsed = String::new();
    let mut prev_dash = false;
    for ch in result.chars() {
        if ch == '-' {
            if !prev_dash {
                collapsed.push('-');
            }
            prev_dash = true;
        } else {
            collapsed.push(ch);
            prev_dash = false;
        }
    }
    collapsed.trim_matches('-').to_string()
}

type TargetOverrides = HashMap<String, Vec<(String, String)>>;

/// Parse YAML-style frontmatter from a rules file.
/// Returns (description, target_overrides, remaining_body).
fn parse_frontmatter(content: &str) -> (Option<String>, TargetOverrides, String) {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return (None, HashMap::new(), content.to_string());
    }

    // Find content after the opening ---
    let after_open = &trimmed[3..];
    let after_open = after_open.trim_start_matches('-');
    let after_first_newline = match after_open.find('\n') {
        Some(i) => &after_open[i + 1..],
        None => return (None, HashMap::new(), content.to_string()),
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
        None => return (None, HashMap::new(), content.to_string()),
    };

    let mut description = None;
    let mut overrides: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for line in fm_block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();
            if key == "description" {
                description = Some(value.to_string());
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

    (description, overrides, rest)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parses_rule_with_frontmatter() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("testing.md");
        fs::write(
            &path,
            "---\ndescription: Testing guidelines\ncursor: alwaysApply=false\n---\nRun cargo test before committing.",
        )
        .unwrap();

        let section = parse_rule(&path).unwrap();
        assert_eq!(section.slug, "testing");
        assert_eq!(section.description, Some("Testing guidelines".to_string()));
        assert_eq!(section.body, "Run cargo test before committing.");
        let cursor = section.target_overrides.get("cursor").unwrap();
        assert_eq!(cursor[0], ("alwaysApply".to_string(), "false".to_string()));
        assert_eq!(section.source, SectionSource::Rules);
    }

    #[test]
    fn parses_rule_without_frontmatter() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("style.md");
        fs::write(&path, "Use 4-space indentation.").unwrap();

        let section = parse_rule(&path).unwrap();
        assert_eq!(section.slug, "style");
        assert_eq!(section.description, None);
        assert_eq!(section.body, "Use 4-space indentation.");
        assert!(section.target_overrides.is_empty());
    }
}
