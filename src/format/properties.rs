use super::{ConfigItem, FormatHandler, ItemStatus, ValueType};
use java_properties::{LineContent, PropertiesIter, PropertiesWriter};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

pub struct PropertiesHandler;

impl FormatHandler for PropertiesHandler {
    fn parse(&self, path: &Path) -> io::Result<Vec<ConfigItem>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let iter = PropertiesIter::new(reader);

        let mut vars = Vec::new();
        let mut groups = std::collections::HashSet::new();

        for line_result in iter {
            let line = line_result.map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            
            if let LineContent::KVPair(path, value) = line.consume_content() {
                // Add groups based on dot notation
                let parts: Vec<&str> = path.split('.').collect();
                let mut current_path = String::new();
                
                for (i, part) in parts.iter().enumerate().take(parts.len().saturating_sub(1)) {
                    if !current_path.is_empty() {
                        current_path.push('.');
                    }
                    current_path.push_str(part);
                    
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

                vars.push(ConfigItem {
                    key: parts.last().unwrap_or(&"").to_string(),
                    path: path.clone(),
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

    fn write(&self, path: &Path, vars: &[ConfigItem]) -> io::Result<()> {
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        let mut prop_writer = PropertiesWriter::new(writer);
        
        for var in vars {
            if !var.is_group {
                let val = var.value.as_deref()
                    .or(var.template_value.as_deref())
                    .unwrap_or("");
                prop_writer.write(&var.path, val)
                    .map_err(io::Error::other)?;
            }
        }

        prop_writer.finish().map_err(io::Error::other)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_parse_properties() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "server.port=8080\ndatabase.host=localhost").unwrap();
        
        let handler = PropertiesHandler;
        let vars = handler.parse(file.path()).unwrap();
        
        assert!(vars.iter().any(|v| v.path == "server" && v.is_group));
        assert!(vars.iter().any(|v| v.path == "server.port" && v.value.as_deref() == Some("8080")));
    }
}
