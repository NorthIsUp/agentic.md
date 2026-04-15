use std::path::Path;

const SECTION_BEGIN: &str = "# BEGIN generated-by: agentic-sync";
const SECTION_END: &str = "# END generated-by: agentic-sync";

fn desired_section(root: &Path, files: &[std::path::PathBuf]) -> String {
    let mut lines = vec![SECTION_BEGIN.to_string()];
    for file in files {
        if let Ok(rel) = file.strip_prefix(root) {
            let rel_str = rel.to_string_lossy().replace('\\', "/");
            lines.push(format!("{rel_str} linguist-generated=true"));
        }
    }
    lines.push(SECTION_END.to_string());
    lines.join("\n")
}

/// Returns true if .gitattributes already contains the up-to-date managed section.
pub fn is_in_sync(root: &Path, files: &[std::path::PathBuf]) -> bool {
    let path = root.join(".gitattributes");
    let desired = desired_section(root, files);
    match std::fs::read_to_string(&path) {
        Ok(content) => extract_section(&content) == Some(desired),
        Err(_) => false,
    }
}

/// Upsert the managed section in .gitattributes. Returns true if the file was written.
pub fn fix(root: &Path, files: &[std::path::PathBuf]) -> Result<bool, String> {
    let path = root.join(".gitattributes");
    let desired = desired_section(root, files);
    let existing = std::fs::read_to_string(&path).unwrap_or_default();
    let new_content = replace_section(&existing, &desired);
    if new_content == existing {
        return Ok(false);
    }
    std::fs::write(&path, &new_content)
        .map_err(|e| format!("failed to write .gitattributes: {e}"))?;
    Ok(true)
}

fn extract_section(content: &str) -> Option<String> {
    let begin = content.find(SECTION_BEGIN)?;
    let end_offset = content[begin..].find(SECTION_END)?;
    let end = begin + end_offset + SECTION_END.len();
    Some(content[begin..end].to_string())
}

fn replace_section(existing: &str, section: &str) -> String {
    if let Some(begin) = existing.find(SECTION_BEGIN) {
        let after_begin = &existing[begin..];
        if let Some(end_offset) = after_begin.find(SECTION_END) {
            let end = begin + end_offset + SECTION_END.len();
            // Consume the trailing newline after END if present
            let after_end = if existing[end..].starts_with('\n') {
                end + 1
            } else {
                end
            };
            format!("{}{}\n{}", &existing[..begin], section, &existing[after_end..])
        } else {
            // Malformed: BEGIN without END — replace from BEGIN to EOF
            format!("{}{}\n", &existing[..begin], section)
        }
    } else if existing.is_empty() {
        format!("{section}\n")
    } else {
        // Append with a blank line separator
        let sep = if existing.ends_with('\n') { "\n" } else { "\n\n" };
        format!("{existing}{sep}{section}\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_files(root: &Path, names: &[&str]) -> Vec<std::path::PathBuf> {
        names.iter().map(|n| root.join(n)).collect()
    }

    #[test]
    fn desired_section_format() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let files = make_files(root, &[".cursor/rules/stack.mdc", ".github/copilot-instructions.md"]);
        let section = desired_section(root, &files);
        assert!(section.starts_with(SECTION_BEGIN));
        assert!(section.ends_with(SECTION_END));
        assert!(section.contains(".cursor/rules/stack.mdc linguist-generated=true"));
        assert!(section.contains(".github/copilot-instructions.md linguist-generated=true"));
    }

    #[test]
    fn fix_creates_gitattributes_from_scratch() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let files = make_files(root, &[".cursor/rules/stack.mdc"]);

        let written = fix(root, &files).unwrap();
        assert!(written);

        let content = fs::read_to_string(root.join(".gitattributes")).unwrap();
        assert!(content.contains(SECTION_BEGIN));
        assert!(content.contains(".cursor/rules/stack.mdc linguist-generated=true"));
        assert!(content.contains(SECTION_END));
    }

    #[test]
    fn fix_appends_to_existing_gitattributes() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        fs::write(root.join(".gitattributes"), "*.log text\n").unwrap();

        let files = make_files(root, &[".cursor/rules/stack.mdc"]);
        fix(root, &files).unwrap();

        let content = fs::read_to_string(root.join(".gitattributes")).unwrap();
        assert!(content.starts_with("*.log text\n"));
        assert!(content.contains(SECTION_BEGIN));
        assert!(content.contains(SECTION_END));
    }

    #[test]
    fn fix_replaces_existing_section() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let old = format!(
            "*.log text\n{SECTION_BEGIN}\n.cursor/rules/old.mdc linguist-generated=true\n{SECTION_END}\n"
        );
        fs::write(root.join(".gitattributes"), &old).unwrap();

        let files = make_files(root, &[".cursor/rules/new.mdc"]);
        fix(root, &files).unwrap();

        let content = fs::read_to_string(root.join(".gitattributes")).unwrap();
        assert!(content.contains("*.log text"));
        assert!(!content.contains("old.mdc"));
        assert!(content.contains(".cursor/rules/new.mdc linguist-generated=true"));
        // Only one begin/end pair
        assert_eq!(content.matches(SECTION_BEGIN).count(), 1);
    }

    #[test]
    fn fix_idempotent() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let files = make_files(root, &[".cursor/rules/stack.mdc"]);

        fix(root, &files).unwrap();
        let first = fs::read_to_string(root.join(".gitattributes")).unwrap();

        let written = fix(root, &files).unwrap();
        assert!(!written); // nothing changed
        let second = fs::read_to_string(root.join(".gitattributes")).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn is_in_sync_detects_stale() {
        let dir = TempDir::new().unwrap();
        let root = dir.path();
        let files = make_files(root, &[".cursor/rules/stack.mdc"]);

        assert!(!is_in_sync(root, &files));
        fix(root, &files).unwrap();
        assert!(is_in_sync(root, &files));

        // Change the file set — now out of sync
        let files2 = make_files(root, &[".cursor/rules/other.mdc"]);
        assert!(!is_in_sync(root, &files2));
    }
}
