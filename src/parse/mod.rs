pub mod claude_md;
pub mod mcp;
pub mod rules;
pub mod skills;

use crate::discover::Sources;
use crate::ir::ProjectConfig;

pub fn parse_all(sources: &Sources) -> Result<ProjectConfig, String> {
    let mut sections = Vec::new();

    if let Some(ref path) = sources.claude_md {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        sections.extend(claude_md::parse_claude_md(&content));
    }

    for path in &sources.rules {
        match rules::parse_rule(path) {
            Ok(section) => sections.push(section),
            Err(e) => crate::log::warn(&e),
        }
    }

    let mut skill_list = Vec::new();
    for path in &sources.skills {
        match skills::parse_skill(path) {
            Ok(skill) => skill_list.push(skill),
            Err(e) => crate::log::warn(&e),
        }
    }

    let mcp_config = match &sources.mcp_json {
        Some(path) => match mcp::parse_mcp(path) {
            Ok(config) => Some(config),
            Err(e) => {
                crate::log::warn(&e);
                None
            }
        },
        None => None,
    };

    Ok(ProjectConfig {
        sections,
        skills: skill_list,
        mcp: mcp_config,
    })
}
