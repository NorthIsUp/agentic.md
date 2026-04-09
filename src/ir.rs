use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectConfig {
    pub sections: Vec<Section>,
    pub skills: Vec<Skill>,
    pub mcp: Option<McpConfig>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub title: Option<String>,
    pub slug: String,
    pub body: String,
    pub source: SectionSource,
    pub target_overrides: HashMap<String, Vec<(String, String)>>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SectionSource {
    ClaudeMd,
    Rules,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Skill {
    pub name: String,
    pub description: Option<String>,
    pub paths: Vec<String>,
    pub disable_model_invocation: bool,
    pub target_overrides: HashMap<String, Vec<(String, String)>>,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct McpConfig {
    pub servers: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_config_default() {
        let config = ProjectConfig {
            sections: vec![],
            skills: vec![],
            mcp: None,
        };
        assert!(config.sections.is_empty());
        assert!(config.skills.is_empty());
        assert!(config.mcp.is_none());
    }

    #[test]
    fn section_slug_for_preamble() {
        let section = Section {
            title: None,
            slug: "_project".to_string(),
            body: "preamble content".to_string(),
            source: SectionSource::ClaudeMd,
            target_overrides: Default::default(),
            description: None,
        };
        assert_eq!(section.slug, "_project");
        assert!(section.title.is_none());
    }
}
