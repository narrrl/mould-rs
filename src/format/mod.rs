use std::path::Path;

pub mod env;
pub mod hierarchical;
pub mod ini;
pub mod properties;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemStatus {
    Present,
    MissingFromActive,
    Modified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    String,
    Number,
    Bool,
    Null,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathSegment {
    Key(String),
    Index(usize),
}

impl std::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::Key(k) => write!(f, "{}", k),
            PathSegment::Index(i) => write!(f, "[{}]", i),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfigItem {
    pub key: String,
    pub path: Vec<PathSegment>,
    pub value: Option<String>,
    pub template_value: Option<String>,
    pub default_value: Option<String>,
    pub depth: usize,
    pub is_group: bool,
    pub status: ItemStatus,
    pub value_type: ValueType,
}

impl ConfigItem {
    pub fn path_string(&self) -> String {
        let mut s = String::new();
        for (i, segment) in self.path.iter().enumerate() {
            match segment {
                PathSegment::Key(k) => {
                    if i > 0 {
                        s.push('.');
                    }
                    s.push_str(k);
                }
                PathSegment::Index(idx) => {
                    s.push_str(&format!("[{}]", idx));
                }
            }
        }
        s
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormatType {
    Env,
    Json,
    Yaml,
    Toml,
    Xml,
    Ini,
    Properties,
}

pub trait FormatHandler {
    fn parse(&self, path: &Path) -> anyhow::Result<Vec<ConfigItem>>;
    fn merge(&self, path: &Path, vars: &mut Vec<ConfigItem>) -> anyhow::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let template_vars = self.parse(path).unwrap_or_default();

        for var in vars.iter_mut() {
            if let Some(template_var) = template_vars.iter().find(|v| v.path == var.path) {
                var.template_value = template_var.value.clone();
                var.default_value = template_var.value.clone();
                
                if var.value != template_var.value {
                    var.status = ItemStatus::Modified;
                } else {
                    var.status = ItemStatus::Present;
                }
            } else {
                // Exists in active, but not in template
                var.status = ItemStatus::Present;
            }
        }
        
        // Add items from template that are missing in active
        for template_var in template_vars {
            if !vars.iter().any(|v| v.path == template_var.path) {
                let mut new_item = template_var.clone();
                new_item.status = ItemStatus::MissingFromActive;
                new_item.value = None;
                vars.push(new_item);
            }
        }

        Ok(())
    }
    fn write(&self, path: &Path, vars: &[ConfigItem]) -> anyhow::Result<()>;
}

pub fn detect_format(path: &Path, override_format: Option<String>) -> FormatType {
    if let Some(fmt) = override_format {
        match fmt.to_lowercase().as_str() {
            "env" => return FormatType::Env,
            "json" => return FormatType::Json,
            "yaml" | "yml" => return FormatType::Yaml,
            "toml" => return FormatType::Toml,
            "xml" => return FormatType::Xml,
            "ini" => return FormatType::Ini,
            "properties" => return FormatType::Properties,
            _ => {}
        }
    }

    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or_default();
    match ext {
        "json" => FormatType::Json,
        "yaml" | "yml" => FormatType::Yaml,
        "toml" => FormatType::Toml,
        "xml" => FormatType::Xml,
        "ini" => FormatType::Ini,
        "properties" => FormatType::Properties,
        _ => FormatType::Env,
    }
}

pub fn get_handler(format: FormatType) -> Box<dyn FormatHandler> {
    match format {
        FormatType::Env => Box::new(env::EnvHandler),
        FormatType::Json => Box::new(hierarchical::HierarchicalHandler::new(FormatType::Json)),
        FormatType::Yaml => Box::new(hierarchical::HierarchicalHandler::new(FormatType::Yaml)),
        FormatType::Toml => Box::new(hierarchical::HierarchicalHandler::new(FormatType::Toml)),
        FormatType::Xml => Box::new(hierarchical::HierarchicalHandler::new(FormatType::Xml)),
        FormatType::Ini => Box::new(ini::IniHandler),
        FormatType::Properties => Box::new(properties::PropertiesHandler),
    }
}
