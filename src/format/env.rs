use super::{EnvVar, FormatHandler};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

pub struct EnvHandler;

impl FormatHandler for EnvHandler {
    fn parse(&self, path: &Path) -> io::Result<Vec<EnvVar>> {
        let content = fs::read_to_string(path)?;
        let mut vars = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue; // Skip comments and empty lines
            }

            if let Some((key, val)) = line.split_once('=') {
                let parsed_val = val.trim().trim_matches('"').trim_matches('\'').to_string();
                vars.push(EnvVar {
                    key: key.trim().to_string(),
                    value: parsed_val.clone(),
                    default_value: parsed_val,
                });
            }
        }

        Ok(vars)
    }

    fn merge(&self, path: &Path, vars: &mut Vec<EnvVar>) -> io::Result<()> {
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
                    var.value = parsed_val;
                } else {
                    vars.push(EnvVar {
                        key: key.to_string(),
                        value: parsed_val.clone(),
                        default_value: String::new(),
                    });
                }
            }
        }

        Ok(())
    }

    fn write(&self, path: &Path, vars: &[EnvVar]) -> io::Result<()> {
        let mut file = fs::File::create(path)?;
        for var in vars {
            writeln!(file, "{}={}", var.key, var.value)?;
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
        assert_eq!(vars[0].value, "value1");
        assert_eq!(vars[0].default_value, "value1");
        assert_eq!(vars[1].key, "KEY2");
        assert_eq!(vars[1].value, "value2");
        assert_eq!(vars[2].key, "KEY3");
        assert_eq!(vars[2].value, "value3");
        assert_eq!(vars[3].key, "EMPTY");
        assert_eq!(vars[3].value, "");
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
        assert_eq!(vars[0].value, "custom1");
        assert_eq!(vars[0].default_value, "default1");

        assert_eq!(vars[1].key, "KEY2");
        assert_eq!(vars[1].value, "default2");
        assert_eq!(vars[1].default_value, "default2");

        assert_eq!(vars[2].key, "KEY3");
        assert_eq!(vars[2].value, "custom3");
        assert_eq!(vars[2].default_value, "");
    }

    #[test]
    fn test_write_env() {
        let file = NamedTempFile::new().unwrap();
        let vars = vec![EnvVar {
            key: "KEY1".to_string(),
            value: "value1".to_string(),
            default_value: "def".to_string(),
        }];

        let handler = EnvHandler;
        handler.write(file.path(), &vars).unwrap();

        let content = fs::read_to_string(file.path()).unwrap();
        assert_eq!(content.trim(), "KEY1=value1");
    }
}
