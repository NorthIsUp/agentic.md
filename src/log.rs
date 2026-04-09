use std::path::Path;

fn is_github_actions() -> bool {
    std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true")
}

pub fn error(msg: &str) {
    if is_github_actions() {
        eprintln!("::error::{msg}");
    } else {
        eprintln!("error: {msg}");
    }
}

pub fn error_file(path: &Path, msg: &str) {
    if is_github_actions() {
        eprintln!("::error file={path}::{msg}", path = path.display());
    } else {
        eprintln!("error: {}: {msg}", path.display());
    }
}

pub fn warn(msg: &str) {
    if is_github_actions() {
        eprintln!("::warning::{msg}");
    } else {
        eprintln!("warning: {msg}");
    }
}

pub fn warn_file(path: &Path, msg: &str) {
    if is_github_actions() {
        eprintln!("::warning file={path}::{msg}", path = path.display());
    } else {
        eprintln!("warning: {}: {msg}", path.display());
    }
}

pub fn info(msg: &str) {
    if is_github_actions() {
        eprintln!("::notice::{msg}");
    } else {
        eprintln!("{msg}");
    }
}

pub fn group(name: &str) {
    if is_github_actions() {
        eprintln!("::group::{name}");
    }
}

pub fn endgroup() {
    if is_github_actions() {
        eprintln!("::endgroup::");
    }
}
