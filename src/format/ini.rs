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
    fn test_parse_ini() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "[server]\nport=8080\n[database]\nhost=localhost").unwrap();
        
        let handler = IniHandler;
        let vars = handler.parse(file.path()).unwrap();
        
        assert!(vars.iter().any(|v| v.path_string() == "server" && v.is_group));
        assert!(vars.iter().any(|v| v.path_string() == "server.port" && v.value.as_deref() == Some("8080")));
        assert!(vars.iter().any(|v| v.path_string() == "database.host" && v.value.as_deref() == Some("localhost")));
    }
}
