//! This module defines the unified data model used by `mould` to represent 
//! configuration data across all supported file formats.
//!
//! By normalizing heterogeneous structures (like nested YAML or flat .env) 
//! into a standard tree-like representation, the TUI logic remains 
//! independent of the underlying file format.

use std::path::Path;

pub mod env;
pub mod hierarchical;
pub mod ini;
pub mod properties;

/// Represents the status of a configuration item relative to a template.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemStatus {
    /// Item exists in the active configuration and matches the template (or no template exists).
    Present,
    /// Item exists in the template but is missing from the active configuration.
    MissingFromActive,
    /// Item has been changed by the user during the current session.
    Modified,
}

/// Hints about the original data type to ensure correct serialization during writes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    /// Standard text.
    String,
    /// Numeric values (integers or floats).
    Number,
    /// True/False values.
    Bool,
    /// Representing an explicit null or empty value.
    Null,
}

/// A single segment in a hierarchical configuration path.
///
/// For example, `services[0].image` would be represented as:
/// `[Key("services"), Index(0), Key("image")]`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PathSegment {
    /// A named key in an object/map.
    Key(String),
    /// A numeric index in an array/list.
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

/// The unified representation of a single configuration entry.
///
/// This model is used for UI rendering and internal manipulation. 
/// Format-specific handlers are responsible for translating their native 
/// data into this structure.
#[derive(Debug, Clone)]
pub struct ConfigItem {
    /// The short display name of the key (e.g., `port`).
    pub key: String,
    /// The full hierarchical path defining this item's location in the config tree.
    pub path: Vec<PathSegment>,
    /// The active value of the configuration entry.
    pub value: Option<String>,
    /// The value found in the template file (if any).
    pub template_value: Option<String>,
    /// A fallback value to use if the item is missing.
    pub default_value: Option<String>,
    /// Visual depth in the tree (used for indentation in the TUI).
    pub depth: usize,
    /// True if this item represents a structural node (object or array) rather than a leaf value.
    pub is_group: bool,
    /// Comparison status relative to the template.
    pub status: ItemStatus,
    /// Metadata about the original data type.
    pub value_type: ValueType,
}

impl ConfigItem {
    /// Returns a human-readable string representation of the full path (e.g., `server.port`).
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

/// Supported configuration file formats.
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

/// Defines the interface for parsing, merging, and writing configuration files.
///
/// Implementing this trait allows `mould` to support new file formats.
pub trait FormatHandler {
    /// Parses a file into the unified `Vec<ConfigItem>` representation.
    fn parse(&self, path: &Path) -> anyhow::Result<Vec<ConfigItem>>;

    /// Merges an active configuration with a template file.
    ///
    /// This identifies missing keys, marks modifications, and syncs default values.
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

    /// Writes the unified representation back to the original file format.
    fn write(&self, path: &Path, vars: &[ConfigItem]) -> anyhow::Result<()>;
}

/// Automatically detects the configuration format based on file extension or an explicit override.
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

/// Factory function to return the appropriate handler implementation for a given format.
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
