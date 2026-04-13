mod check_test;
mod cleanup_test;
mod fix_test;

use std::fs;
use std::path::Path;

pub fn setup_basic_project(root: &Path) {
    fs::write(
        root.join("CLAUDE.md"),
        "Project preamble\n\n## Stack\nRust + Clap\n\n## Testing\nUse cargo test\n",
    )
    .unwrap();
}

pub fn setup_full_project(root: &Path) {
    setup_basic_project(root);
    fs::create_dir_all(root.join(".claude/rules")).unwrap();
    fs::write(
        root.join(".claude/rules/style.md"),
        "---\ndescription: Code style\n---\nUse 4-space indentation\n",
    )
    .unwrap();
    fs::create_dir_all(root.join(".claude/skills/explain-code")).unwrap();
    fs::write(
        root.join(".claude/skills/explain-code/SKILL.md"),
        "---\nname: explain-code\ndescription: Explains code\n---\nUse diagrams when explaining.\n",
    )
    .unwrap();
    fs::write(
        root.join(".mcp.json"),
        r#"{"context7": {"command": "npx", "args": ["-y", "@upstash/context7-mcp"]}}"#,
    )
    .unwrap();
}
