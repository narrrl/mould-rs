//! Handler for flat `.env` (Environment) configuration files.
//!
//! This handler manages simple `KEY=VALUE` pairs. It does not support 
//! native nesting or grouping, treating all entries as root-level variables.

use super::{ConfigItem, FormatHandler, ItemStatus, ValueType, PathSegment};
use std::fs;
use std::io::Write;
use std::path::Path;

/// A format handler for parsing and writing `.env` files.
pub struct EnvHandler;

impl FormatHandler for EnvHandler {
    /// Parses an environment file into a flat list of `ConfigItem`s.
    fn parse(&self, path: &Path) -> anyhow::Result<Vec<ConfigItem>> {
        if !path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(path)?;
        let mut vars = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            // Skip empty lines and comments.
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().to_string();
                
                vars.push(ConfigItem {
                    key: key.clone(),
                    path: vec![PathSegment::Key(key)],
                    value: Some(value.clone()),
                    template_value: Some(value.clone()),
                    default_value: Some(value.clone()),
                    depth: 0,
                    is_group: false,
                    status: ItemStatus::Present,
                    value_type: ValueType::String,
                });
            }
        }

        Ok(vars)
    }

    /// Writes the list of variables back to a flat `.env` file.
    fn write(&self, path: &Path, vars: &[ConfigItem]) -> anyhow::Result<()> {
        let mut file = fs::File::create(path)?;
        for var in vars {
            // .env files ignore structural groups.
            if !var.is_group {
                let val = var.value.as_deref()
                    .or(var.template_value.as_deref())
                    .unwrap_or("");
                writeln!(file, "{}={}", var.key, val)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_env_example() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "# Comment\nKEY1=value1\n  KEY2 = value2  ").unwrap();
        
        let handler = EnvHandler;
        let vars = handler.parse(file.path()).unwrap();
        
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].key, "KEY1");
        assert_eq!(vars[0].value.as_deref(), Some("value1"));
        assert_eq!(vars[1].key, "KEY2");
        assert_eq!(vars[1].value.as_deref(), Some("value2"));
    }

    #[test]
    fn test_write_env() {
        let file = NamedTempFile::new().unwrap();
        let vars = vec![ConfigItem {
            key: "KEY1".to_string(),
            path: vec![PathSegment::Key("KEY1".to_string())],
            value: Some("value1".to_string()),
            template_value: None,
            default_value: None,
            depth: 0,
            is_group: false,
            status: ItemStatus::Present,
            value_type: ValueType::String,
        }];

        let handler = EnvHandler;
        handler.write(file.path(), &vars).unwrap();

        let content = fs::read_to_string(file.path()).unwrap();
        assert_eq!(content.trim(), "KEY1=value1");
    }

    #[test]
    fn test_merge_env() {
        let template = NamedTempFile::new().unwrap();
        writeln!(template.as_file(), "KEY1=template_val\nKEY2=default_val").unwrap();

        let mut active_vars = vec![ConfigItem {
            key: "KEY1".to_string(),
            path: vec![PathSegment::Key("KEY1".to_string())],
            value: Some("active_val".to_string()),
            template_value: None,
            default_value: None,
            depth: 0,
            is_group: false,
            status: ItemStatus::Present,
            value_type: ValueType::String,
        }];

        let handler = EnvHandler;
        handler.merge(template.path(), &mut active_vars).unwrap();

        assert_eq!(active_vars.len(), 2);
        
        // KEY1 should be marked modified
        let key1 = active_vars.iter().find(|v| v.key == "KEY1").unwrap();
        assert_eq!(key1.status, ItemStatus::Modified);
        assert_eq!(key1.value.as_deref(), Some("active_val"));
        assert_eq!(key1.template_value.as_deref(), Some("template_val"));

        // KEY2 should be marked missing
        let key2 = active_vars.iter().find(|v| v.key == "KEY2").unwrap();
        assert_eq!(key2.status, ItemStatus::MissingFromActive);
        assert_eq!(key2.value, None);
        assert_eq!(key2.template_value.as_deref(), Some("default_val"));
    }
}
