use crate::ir::McpConfig;
use std::path::Path;

pub fn parse_mcp(path: &Path) -> Result<McpConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let value: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("Invalid JSON in {}: {e}", path.display()))?;

    let servers = if let Some(mcp_servers) = value.get("mcpServers") {
        mcp_servers.clone()
    } else {
        value
    };

    Ok(McpConfig { servers })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn parses_flat_mcp_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".mcp.json");
        fs::write(&path, r#"{"context7": {"command": "npx"}}"#).unwrap();

        let config = parse_mcp(&path).unwrap();
        assert!(config.servers.get("context7").is_some());
        assert_eq!(
            config.servers["context7"]["command"],
            serde_json::json!("npx")
        );
    }

    #[test]
    fn parses_wrapped_mcp_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(".mcp.json");
        fs::write(
            &path,
            r#"{"mcpServers": {"linear": {"command": "node", "args": ["server.js"]}}}"#,
        )
        .unwrap();

        let config = parse_mcp(&path).unwrap();
        assert!(config.servers.get("linear").is_some());
        assert_eq!(
            config.servers["linear"]["command"],
            serde_json::json!("node")
        );
    }
}
