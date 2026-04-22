use std::process::ExitCode;

#[test]
fn check_reports_stale_on_fresh_project() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_basic_project(dir.path());
    let result = agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Check,
        &agentic_sync::all_targets(),
        agentic_sync::Prefer::default(),
        false,
        true,
    )
    .unwrap();
    assert_eq!(result, ExitCode::from(1));
}

#[test]
fn check_passes_after_fix() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_basic_project(dir.path());
    agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Fix { overwrite: false },
        &agentic_sync::all_targets(),
        agentic_sync::Prefer::default(),
        false,
        true,
    )
    .unwrap();
    let result = agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Check,
        &agentic_sync::all_targets(),
        agentic_sync::Prefer::default(),
        false,
        true,
    )
    .unwrap();
    assert_eq!(result, ExitCode::SUCCESS);
}

#[test]
fn pr_mode_exits_1_when_stale() {
    let dir = tempfile::tempdir().unwrap();
    super::setup_basic_project(dir.path());
    let result = agentic_sync::run(
        dir.path(),
        agentic_sync::Mode::Pr,
        &agentic_sync::all_targets(),
        agentic_sync::Prefer::default(),
        false,
        true,
    )
    .unwrap();
    assert_eq!(result, ExitCode::from(1));
}
