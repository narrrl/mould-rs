use std::io;
use std::path::Path;

pub mod env;
pub mod hierarchical;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemStatus {
    Present,
    MissingFromActive,
    MissingFromTemplate,
    Modified,
}

#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub key: String,
    pub path: String,
    pub value: Option<String>,
    pub template_value: Option<String>,
    pub default_value: Option<String>,
    pub depth: usize,
    pub is_group: bool,
    pub status: ItemStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    Env,
    Json,
    Yaml,
    Toml,
}

pub trait FormatHandler {
    fn parse(&self, path: &Path) -> io::Result<Vec<ConfigItem>>;
    fn merge(&self, path: &Path, vars: &mut Vec<ConfigItem>) -> io::Result<()>;
    fn write(&self, path: &Path, vars: &[ConfigItem]) -> io::Result<()>;
}

pub fn detect_format(path: &Path, override_format: Option<String>) -> FormatType {
    if let Some(fmt) = override_format {
        match fmt.to_lowercase().as_str() {
            "env" => return FormatType::Env,
            "json" => return FormatType::Json,
            "yaml" | "yml" => return FormatType::Yaml,
            "toml" => return FormatType::Toml,
            _ => {}
        }
    }

    let file_name = path.file_name().unwrap_or_default().to_string_lossy();
    if file_name.ends_with(".json") {
        FormatType::Json
    } else if file_name.ends_with(".yaml") || file_name.ends_with(".yml") {
        FormatType::Yaml
    } else if file_name.ends_with(".toml") {
        FormatType::Toml
    } else {
        FormatType::Env
    }
}

pub fn get_handler(format: FormatType) -> Box<dyn FormatHandler> {
    match format {
        FormatType::Env => Box::new(env::EnvHandler),
        FormatType::Json => Box::new(hierarchical::HierarchicalHandler::new(FormatType::Json)),
        FormatType::Yaml => Box::new(hierarchical::HierarchicalHandler::new(FormatType::Yaml)),
        FormatType::Toml => Box::new(hierarchical::HierarchicalHandler::new(FormatType::Toml)),
    }
}
