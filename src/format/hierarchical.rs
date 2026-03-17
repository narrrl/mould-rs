use super::{ConfigItem, FormatHandler, FormatType, ItemStatus};
use serde_json::{Map, Value};
use std::fs;
use std::io;
use std::path::Path;

pub struct HierarchicalHandler {
    format_type: FormatType,
}

impl HierarchicalHandler {
    pub fn new(format_type: FormatType) -> Self {
        Self { format_type }
    }

    fn read_value(&self, path: &Path) -> io::Result<Value> {
        let content = fs::read_to_string(path)?;
        let value = match self.format_type {
            FormatType::Json => serde_json::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Yaml => serde_yaml::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Toml => toml::from_str(&content)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            _ => unreachable!(),
        };
        Ok(value)
    }

    fn write_value(&self, path: &Path, value: &Value) -> io::Result<()> {
        let content = match self.format_type {
            FormatType::Json => serde_json::to_string_pretty(value)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Yaml => serde_yaml::to_string(value)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Toml => {
                // toml requires the root to be a table
                if value.is_object() {
                    let toml_value: toml::Value = serde_json::from_value(value.clone())
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    toml::to_string_pretty(&toml_value)
                        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Root of TOML must be an object",
                    ));
                }
            }
            _ => unreachable!(),
        };
        fs::write(path, content)
    }
}

fn flatten(value: &Value, prefix: &str, depth: usize, key_name: &str, vars: &mut Vec<ConfigItem>) {
    let path = if prefix.is_empty() {
        key_name.to_string()
    } else if key_name.is_empty() {
        prefix.to_string()
    } else {
        format!("{}.{}", prefix, key_name)
    };

    match value {
        Value::Object(map) => {
            if !path.is_empty() {
                vars.push(ConfigItem {
                    key: key_name.to_string(),
                    path: path.clone(),
                    value: None,
                    template_value: None,
                    default_value: None,
                    depth,
                    is_group: true,
                    status: ItemStatus::Present,
                });
            }
            let next_depth = if path.is_empty() { depth } else { depth + 1 };
            for (k, v) in map {
                flatten(v, &path, next_depth, k, vars);
            }
        }
        Value::Array(arr) => {
            if !path.is_empty() {
                vars.push(ConfigItem {
                    key: key_name.to_string(),
                    path: path.clone(),
                    value: None,
                    template_value: None,
                    default_value: None,
                    depth,
                    is_group: true,
                    status: ItemStatus::Present,
                });
            }
            let next_depth = if path.is_empty() { depth } else { depth + 1 };
            for (i, v) in arr.iter().enumerate() {
                let array_key = format!("[{}]", i);
                flatten(v, &path, next_depth, &array_key, vars);
            }
        }
        Value::String(s) => {
            vars.push(ConfigItem {
                key: key_name.to_string(),
                path: path.clone(),
                value: Some(s.clone()),
                template_value: Some(s.clone()),
                default_value: Some(s.clone()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
            });
        }
        Value::Number(n) => {
            let s = n.to_string();
            vars.push(ConfigItem {
                key: key_name.to_string(),
                path: path.clone(),
                value: Some(s.clone()),
                template_value: Some(s.clone()),
                default_value: Some(s.clone()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
            });
        }
        Value::Bool(b) => {
            let s = b.to_string();
            vars.push(ConfigItem {
                key: key_name.to_string(),
                path: path.clone(),
                value: Some(s.clone()),
                template_value: Some(s.clone()),
                default_value: Some(s.clone()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
            });
        }
        Value::Null => {
            vars.push(ConfigItem {
                key: key_name.to_string(),
                path: path.clone(),
                value: Some("".to_string()),
                template_value: Some("".to_string()),
                default_value: Some("".to_string()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
            });
        }
    }
}

impl FormatHandler for HierarchicalHandler {
    fn parse(&self, path: &Path) -> io::Result<Vec<ConfigItem>> {
        let value = self.read_value(path)?;
        let mut vars = Vec::new();
        flatten(&value, "", 0, "", &mut vars);
        Ok(vars)
    }

    fn merge(&self, path: &Path, vars: &mut Vec<ConfigItem>) -> io::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let existing_value = self.read_value(path)?;
        let mut existing_vars = Vec::new();
        flatten(&existing_value, "", 0, "", &mut existing_vars);

        for var in vars.iter_mut() {
            if let Some(existing) = existing_vars.iter().find(|v| v.path == var.path) {
                if var.value != existing.value {
                    var.value = existing.value.clone();
                    var.status = ItemStatus::Modified;
                }
            } else {
                var.status = ItemStatus::MissingFromActive;
            }
        }
        
        // Find keys in active that are not in template
        let mut to_add = Vec::new();
        for existing in existing_vars {
            if !vars.iter().any(|v| v.path == existing.path) {
                let mut new_item = existing.clone();
                new_item.status = ItemStatus::MissingFromTemplate;
                new_item.template_value = None;
                new_item.default_value = None;
                to_add.push(new_item);
            }
        }
        
        // Basic insertion logic for extra keys (could be improved to insert at correct depth/position)
        vars.extend(to_add);

        Ok(())
    }

    fn write(&self, path: &Path, vars: &[ConfigItem]) -> io::Result<()> {
        let mut root = Value::Object(Map::new());
        for var in vars {
            if !var.is_group {
                let val = var.value.as_deref()
                    .or(var.template_value.as_deref())
                    .unwrap_or("");
                insert_into_value(&mut root, &var.path, val);
            }
        }
        self.write_value(path, &root)
    }
}

fn insert_into_value(root: &mut Value, path: &str, new_val_str: &str) {
    let mut parts = path.split('.');
    let last_part = match parts.next_back() {
        Some(p) => p,
        None => return,
    };

    let mut current = root;
    for part in parts {
        let (key, idx) = parse_array_key(part);
        if !current.is_object() {
            *current = Value::Object(Map::new());
        }
        let map = current.as_object_mut().unwrap();

        let next_node = map.entry(key.to_string()).or_insert_with(|| {
            if idx.is_some() {
                Value::Array(Vec::new())
            } else {
                Value::Object(Map::new())
            }
        });

        if let Some(i) = idx {
            if !next_node.is_array() {
                *next_node = Value::Array(Vec::new());
            }
            let arr = next_node.as_array_mut().unwrap();
            while arr.len() <= i {
                arr.push(Value::Object(Map::new()));
            }
            current = &mut arr[i];
        } else {
            current = next_node;
        }
    }

    let (final_key, final_idx) = parse_array_key(last_part);
    if !current.is_object() {
        *current = Value::Object(Map::new());
    }
    let map = current.as_object_mut().unwrap();

    // Attempt basic type inference
    let final_val = if let Ok(n) = new_val_str.parse::<i64>() {
        Value::Number(n.into())
    } else if let Ok(b) = new_val_str.parse::<bool>() {
        Value::Bool(b)
    } else {
        Value::String(new_val_str.to_string())
    };

    if let Some(i) = final_idx {
        let next_node = map
            .entry(final_key.to_string())
            .or_insert_with(|| Value::Array(Vec::new()));
        if !next_node.is_array() {
            *next_node = Value::Array(Vec::new());
        }
        let arr = next_node.as_array_mut().unwrap();
        while arr.len() <= i {
            arr.push(Value::Null);
        }
        arr[i] = final_val;
    } else {
        map.insert(final_key.to_string(), final_val);
    }
}

fn parse_array_key(part: &str) -> (&str, Option<usize>) {
    if part.ends_with(']') && part.contains('[') {
        let start_idx = part.find('[').unwrap();
        let key = &part[..start_idx];
        let idx = part[start_idx + 1..part.len() - 1].parse::<usize>().ok();
        (key, idx)
    } else {
        (part, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flatten_unflatten() {
        let mut vars = Vec::new();
        let json = serde_json::json!({
            "services": {
                "web": {
                    "ports": ["8080:80"],
                    "environment": {
                        "DEBUG": true
                    }
                }
            }
        });

        flatten(&json, "", &mut vars);
        assert_eq!(vars.len(), 2);

        let mut root = Value::Object(Map::new());
        for var in vars {
            insert_into_value(&mut root, &var.key, &var.value);
        }

        // When unflattening, it parses bool back
        let unflattened_json = serde_json::to_string(&root).unwrap();
        assert!(unflattened_json.contains("\"8080:80\""));
        assert!(unflattened_json.contains("true"));
    }
}
