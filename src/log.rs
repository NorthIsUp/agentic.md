pub fn error(msg: &str) {
    if std::env::var("GITHUB_ACTIONS").as_deref() == Ok("true") {
        eprintln!("::error::{msg}");
    } else {
        eprintln!("error: {msg}");
    }
}
