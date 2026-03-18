use super::{ConfigItem, FormatHandler, ItemStatus, ValueType, PathSegment};
use ini::Ini;
use std::path::Path;

pub struct IniHandler;

impl FormatHandler for IniHandler {
    fn parse(&self, path: &Path) -> anyhow::Result<Vec<ConfigItem>> {
        let conf = Ini::load_from_file(path)?;
        let mut vars = Vec::new();

        for (section, prop) in &conf {
            let section_name = section.unwrap_or_default();
            
            if !section_name.is_empty() {
                vars.push(ConfigItem {
                    key: section_name.to_string(),
                    path: vec![PathSegment::Key(section_name.to_string())],
                    value: None,
                    template_value: None,
                    default_value: None,
                    depth: 0,
                    is_group: true,
                    status: ItemStatus::Present,
                    value_type: ValueType::Null,
                });
            }

            for (key, value) in prop {
                let path = if section_name.is_empty() {
                    vec![PathSegment::Key(key.to_string())]
                } else {
                    vec![PathSegment::Key(section_name.to_string()), PathSegment::Key(key.to_string())]
                };

                vars.push(ConfigItem {
                    key: key.to_string(),
                    path,
                    value: Some(value.to_string()),
                    template_value: Some(value.to_string()),
                    default_value: Some(value.to_string()),
                    depth: if section_name.is_empty() { 0 } else { 1 },
                    is_group: false,
                    status: ItemStatus::Present,
                    value_type: ValueType::String,
                });
            }
        }

        Ok(vars)
    }

    fn write(&self, path: &Path, vars: &[ConfigItem]) -> anyhow::Result<()> {
        let mut conf = Ini::new();
        for var in vars {
            if !var.is_group {
                let val = var.value.as_deref()
                    .or(var.template_value.as_deref())
                    .unwrap_or("");
                
                if var.path.len() == 2 {
                    if let (PathSegment::Key(section), PathSegment::Key(key)) = (&var.path[0], &var.path[1]) {
                        conf.with_section(Some(section)).set(key, val);
                    }
                } else if var.path.len() == 1
                    && let PathSegment::Key(key) = &var.path[0] {
                        conf.with_section(None::<String>).set(key, val);
                    }
            }
        }
        conf.write_to_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_section_rename_write() {
        let handler = IniHandler;
        let mut vars = vec![
            ConfigItem {
                key: "server".to_string(),
                path: vec![PathSegment::Key("server".to_string())],
                value: None,
                template_value: None,
                default_value: None,
                depth: 0,
                is_group: true,
                status: ItemStatus::Present,
                value_type: ValueType::Null,
            },
            ConfigItem {
                key: "port".to_string(),
                path: vec![PathSegment::Key("server".to_string()), PathSegment::Key("port".to_string())],
                value: Some("8080".to_string()),
                template_value: Some("8080".to_string()),
                default_value: Some("8080".to_string()),
                depth: 1,
                is_group: false,
                status: ItemStatus::Present,
                value_type: ValueType::String,
            }
        ];

        // Rename "server" to "srv"
        vars[0].key = "srv".to_string();
        vars[0].path = vec![PathSegment::Key("srv".to_string())];
        
        // Update child path
        vars[1].path = vec![PathSegment::Key("srv".to_string()), PathSegment::Key("port".to_string())];
        
        let file = NamedTempFile::new().unwrap();
        handler.write(file.path(), &vars).unwrap();
        
        let content = std::fs::read_to_string(file.path()).unwrap();
        assert!(content.contains("[srv]"));
        assert!(content.contains("port=8080"));
        assert!(!content.contains("[server]"));
    }
}
