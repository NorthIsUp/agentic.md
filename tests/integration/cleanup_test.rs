use std::fs;

#[test]
fn cleanup_removes_stale_generated_files() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_full_project(dir.path());

    agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &agentic_sync::all_targets(),
    )
    .unwrap();
    assert!(dir.path().join(".cursor/rules/stack.mdc").exists());

    // Remove ## Stack from CLAUDE.md
    fs::write(
        dir.path().join("CLAUDE.md"),
        "Project preamble\n\n## Testing\nUse cargo test\n",
    )
    .unwrap();

    agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &agentic_sync::all_targets(),
    )
    .unwrap();

    assert!(!dir.path().join(".cursor/rules/stack.mdc").exists());
    assert!(dir.path().join(".cursor/rules/testing.mdc").exists());
}
