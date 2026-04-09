use crate::ir::{Section, SectionSource};
use std::collections::HashMap;

pub fn parse_claude_md(content: &str) -> Vec<Section> {
    let raw_sections = split_on_h2(content);
    let mut slug_counts: HashMap<String, usize> = HashMap::new();
    let mut sections = Vec::new();

    for (title, body) in raw_sections {
        let (overrides, clean_body) = parse_section_frontmatter(&body);
        let trimmed = clean_body.trim();
        if trimmed.is_empty() {
            continue;
        }

        let base_slug = match &title {
            Some(t) => slugify(t),
            None => "_project".to_string(),
        };

        let count = slug_counts.entry(base_slug.clone()).or_insert(0);
        *count += 1;
        let slug = if *count == 1 {
            base_slug
        } else {
            format!("{base_slug}-{count}")
        };

        sections.push(Section {
            title,
            slug,
            body: trimmed.to_string(),
            source: SectionSource::ClaudeMd,
            target_overrides: overrides,
            description: None,
        });
    }

    sections
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
    // Trim leading/trailing dashes
    collapsed.trim_matches('-').to_string()
}

/// Split content on `## ` at the start of a line.
/// Returns vec of (Option<title>, body).
fn split_on_h2(content: &str) -> Vec<(Option<String>, String)> {
    let mut sections = Vec::new();
    let mut current_title: Option<String> = None;
    let mut current_body = String::new();
    let mut in_preamble = true;

    for line in content.lines() {
        if let Some(heading) = line.strip_prefix("## ") {
            // Flush previous section
            if in_preamble {
                if !current_body.trim().is_empty() {
                    sections.push((None, current_body));
                }
                in_preamble = false;
            } else {
                sections.push((current_title, current_body));
            }
            current_title = Some(heading.trim().to_string());
            current_body = String::new();
        } else {
            if !current_body.is_empty() || !line.is_empty() || !in_preamble {
                current_body.push_str(line);
                current_body.push('\n');
            } else if in_preamble && !line.is_empty() {
                current_body.push_str(line);
                current_body.push('\n');
            }
        }
    }

    // Flush last section
    if in_preamble {
        if !current_body.trim().is_empty() {
            sections.push((None, current_body));
        }
    } else {
        sections.push((current_title, current_body));
    }

    sections
}

/// Parse optional frontmatter from a section body.
/// Frontmatter is a `---` delimited block right after the heading.
/// Lines matching `target: key=value` are collected as overrides.
/// Returns (overrides_map, remaining_body).
fn parse_section_frontmatter(body: &str) -> (HashMap<String, Vec<(String, String)>>, String) {
    let trimmed = body.trim_start();
    if !trimmed.starts_with("---") {
        return (HashMap::new(), body.to_string());
    }

    // Find the closing ---
    let after_open = &trimmed[3..].trim_start_matches(|c: char| c == '-');
    let after_first_newline = match after_open.find('\n') {
        Some(i) => &after_open[i + 1..],
        None => return (HashMap::new(), body.to_string()),
    };

    let close_pos = after_first_newline.find("\n---");
    let (fm_block, rest) = match close_pos {
        Some(pos) => {
            let fm = &after_first_newline[..pos];
            let after_close = &after_first_newline[pos + 4..]; // skip \n---
            // Skip past optional trailing dashes and newline
            let after_close = after_close.trim_start_matches('-');
            let after_close = if after_close.starts_with('\n') {
                &after_close[1..]
            } else {
                after_close
            };
            (fm, after_close.to_string())
        }
        None => return (HashMap::new(), body.to_string()),
    };

    let mut overrides: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for line in fm_block.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        // Format: target: key=value
        if let Some((target, kv)) = line.split_once(':') {
            let target = target.trim();
            let kv = kv.trim();
            if let Some((key, value)) = kv.split_once('=') {
                overrides
                    .entry(target.to_string())
                    .or_default()
                    .push((key.trim().to_string(), value.trim().to_string()));
            }
        }
    }

    (overrides, rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_on_h2_headings() {
        let content = "Preamble\n\n## Stack\nRust\n\n## Testing\nCargo test";
        let sections = parse_claude_md(content);
        assert_eq!(sections.len(), 3);
        assert_eq!(sections[0].slug, "_project");
        assert_eq!(sections[0].body, "Preamble");
        assert_eq!(sections[1].slug, "stack");
        assert_eq!(sections[1].body, "Rust");
        assert_eq!(sections[2].slug, "testing");
        assert_eq!(sections[2].body, "Cargo test");
    }

    #[test]
    fn skips_empty_sections() {
        let content = "## Empty\n\n## HasContent\nContent";
        let sections = parse_claude_md(content);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slug, "hascontent");
        assert_eq!(sections[0].body, "Content");
    }

    #[test]
    fn no_preamble_if_starts_with_heading() {
        let content = "## Stack\nRust";
        let sections = parse_claude_md(content);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].slug, "stack");
        assert_eq!(sections[0].title, Some("Stack".to_string()));
    }

    #[test]
    fn deduplicates_slugs() {
        let content = "## Testing\nUnit\n\n## Testing\nIntegration";
        let sections = parse_claude_md(content);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].slug, "testing");
        assert_eq!(sections[1].slug, "testing-2");
    }

    #[test]
    fn parses_cursor_frontmatter_overrides() {
        let content = "## Rules\n---\ncursor: alwaysApply=true\n---\nSome rules here";
        let sections = parse_claude_md(content);
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].body, "Some rules here");
        let cursor_overrides = sections[0].target_overrides.get("cursor").unwrap();
        assert_eq!(cursor_overrides.len(), 1);
        assert_eq!(
            cursor_overrides[0],
            ("alwaysApply".to_string(), "true".to_string())
        );
    }

    #[test]
    fn slugify_works() {
        assert_eq!(slugify("Commit Conventions"), "commit-conventions");
        assert_eq!(slugify("Hello---World"), "hello-world");
        assert_eq!(slugify("  Leading  "), "leading");
    }
}
