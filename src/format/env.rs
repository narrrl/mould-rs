use super::{ConfigItem, FormatHandler, ItemStatus, ValueType};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct EnvHandler;

impl FormatHandler for EnvHandler {
    fn parse(&self, path: &Path) -> io::Result<Vec<ConfigItem>> {
        let content = fs::read_to_string(path)?;
        let mut vars = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // Skip comments and empty lines
            }

            if let Some((key, val)) = line.split_once('=') {
                let parsed_val = val.trim().trim_matches('"').trim_matches('\'').to_string();
                vars.push(ConfigItem {
                    key: key.trim().to_string(),
                    path: key.trim().to_string(),
                    value: Some(parsed_val.clone()),
                    template_value: Some(parsed_val.clone()),
                    default_value: Some(parsed_val),
                    depth: 0,
                    is_group: false,
                    status: ItemStatus::Present,
                    value_type: ValueType::String,
                });
            }
        }

        Ok(vars)
    }

    fn merge(&self, path: &Path, vars: &mut Vec<ConfigItem>) -> io::Result<()> {
        if !path.exists() {
            return Ok(());
        }

        let content = fs::read_to_string(path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = line.split_once('=') {
                let key = key.trim();
                let parsed_val = val.trim().trim_matches('"').trim_matches('\'').to_string();

                if let Some(var) = vars.iter_mut().find(|v| v.key == key) {
                    if var.value.as_deref() != Some(&parsed_val) {
                        var.value = Some(parsed_val);
                        var.status = ItemStatus::Modified;
                    }
                } else {
                    vars.push(ConfigItem {
                        key: key.to_string(),
                        path: key.to_string(),
                        value: Some(parsed_val),
                        template_value: None,
                        default_value: None,
                        depth: 0,
                        is_group: false,
                        status: ItemStatus::MissingFromTemplate,
                        value_type: ValueType::String,
                    });
                }
            }
        }
        
        // Mark missing from active
        for var in vars.iter_mut() {
            if var.status == ItemStatus::Present && var.value.is_none() {
                var.status = ItemStatus::MissingFromActive;
            }
        }

        Ok(())
    }

    fn write(&self, path: &Path, vars: &[ConfigItem]) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        for var in vars {
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
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_env_example() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            "# A comment\nKEY1=value1\nKEY2=\"value2\"\nKEY3='value3'\nEMPTY="
        )
        .unwrap();

        let handler = EnvHandler;
        let vars = handler.parse(file.path()).unwrap();
        assert_eq!(vars.len(), 4);
        assert_eq!(vars[0].key, "KEY1");
        assert_eq!(vars[0].value.as_deref(), Some("value1"));
        assert_eq!(vars[1].key, "KEY2");
        assert_eq!(vars[1].value.as_deref(), Some("value2"));
        assert_eq!(vars[2].key, "KEY3");
        assert_eq!(vars[2].value.as_deref(), Some("value3"));
        assert_eq!(vars[3].key, "EMPTY");
        assert_eq!(vars[3].value.as_deref(), Some(""));
    }

    #[test]
    fn test_merge_env() {
        let mut example_file = NamedTempFile::new().unwrap();
        writeln!(example_file, "KEY1=default1\nKEY2=default2").unwrap();
        let handler = EnvHandler;
        let mut vars = handler.parse(example_file.path()).unwrap();

        let mut env_file = NamedTempFile::new().unwrap();
        writeln!(env_file, "KEY1=custom1\nKEY3=custom3").unwrap();

        handler.merge(env_file.path(), &mut vars).unwrap();

        assert_eq!(vars.len(), 3);
        assert_eq!(vars[0].key, "KEY1");
        assert_eq!(vars[0].value.as_deref(), Some("custom1"));
        assert_eq!(vars[0].status, ItemStatus::Modified);

        assert_eq!(vars[1].key, "KEY2");
        assert_eq!(vars[1].value.as_deref(), Some("default2"));
        
        assert_eq!(vars[2].key, "KEY3");
        assert_eq!(vars[2].value.as_deref(), Some("custom3"));
        assert_eq!(vars[2].status, ItemStatus::MissingFromTemplate);
    }

    #[test]
    fn test_write_env() {
        let file = NamedTempFile::new().unwrap();
        let vars = vec![ConfigItem {
            key: "KEY1".to_string(),
            path: "KEY1".to_string(),
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
}
