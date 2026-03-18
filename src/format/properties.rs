use super::{ConfigItem, FormatHandler, ItemStatus, ValueType, PathSegment};
use java_properties::{LineContent, PropertiesIter, PropertiesWriter};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::Path;

pub struct PropertiesHandler;

impl FormatHandler for PropertiesHandler {
    fn parse(&self, path: &Path) -> anyhow::Result<Vec<ConfigItem>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let iter = PropertiesIter::new(reader);

        let mut vars = Vec::new();
        let mut groups = std::collections::HashSet::new();

        for line_result in iter {
            let line = line_result?;
            
            if let LineContent::KVPair(path, value) = line.consume_content() {
                // Add groups based on dot notation
                let parts: Vec<&str> = path.split('.').collect();
                let mut current_path = Vec::new();
                
                for (i, part) in parts.iter().enumerate().take(parts.len().saturating_sub(1)) {
                    current_path.push(PathSegment::Key(part.to_string()));
                    
                    if groups.insert(current_path.clone()) {
                        vars.push(ConfigItem {
                            key: part.to_string(),
                            path: current_path.clone(),
                            value: None,
                            template_value: None,
                            default_value: None,
                            depth: i,
                            is_group: true,
                            status: ItemStatus::Present,
                            value_type: ValueType::Null,
                        });
                    }
                }

                let mut final_path = current_path.clone();
                let last_key = parts.last().unwrap_or(&"").to_string();
                final_path.push(PathSegment::Key(last_key.clone()));

                vars.push(ConfigItem {
                    key: last_key,
                    path: final_path,
                    value: Some(value.clone()),
                    template_value: Some(value.clone()),
                    default_value: Some(value.clone()),
                    depth: parts.len().saturating_sub(1),
                    is_group: false,
                    status: ItemStatus::Present,
                    value_type: ValueType::String,
                });
            }
        }

        // We don't sort here to preserve the original file order!
        Ok(vars)
    }

    fn write(&self, path: &Path, vars: &[ConfigItem]) -> anyhow::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut prop_writer = PropertiesWriter::new(writer);
        
        for var in vars {
            if !var.is_group {
                let val = var.value.as_deref()
                    .or(var.template_value.as_deref())
                    .unwrap_or("");
                prop_writer.write(&var.path_string(), val)?;
            }
        }

        prop_writer.finish()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_group_rename_write() {
        let handler = PropertiesHandler;
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
        assert!(content.contains("srv.port=8080"));
        assert!(!content.contains("server.port=8080"));
    }
}
