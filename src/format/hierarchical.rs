use super::{EnvVar, FormatHandler, FormatType};
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
            FormatType::Json => serde_json::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Yaml => serde_yaml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Toml => toml::from_str(&content).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            _ => unreachable!(),
        };
        Ok(value)
    }

    fn write_value(&self, path: &Path, value: &Value) -> io::Result<()> {
        let content = match self.format_type {
            FormatType::Json => serde_json::to_string_pretty(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Yaml => serde_yaml::to_string(value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
            FormatType::Toml => {
                // toml requires the root to be a table
                if value.is_object() {
                    let toml_value: toml::Value = serde_json::from_value(value.clone()).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                    toml::to_string_pretty(&toml_value).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                } else {
                    return Err(io::Error::new(io::ErrorKind::InvalidData, "Root of TOML must be an object"));
                }
            }
            _ => unreachable!(),
        };
        fs::write(path, content)
    }
}

fn flatten(value: &Value, prefix: &str, vars: &mut Vec<EnvVar>) {
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                let new_prefix = if prefix.is_empty() {
                    k.clone()
                } else {
                    format!("{}.{}", prefix, k)
                };
                flatten(v, &new_prefix, vars);
            }
        }
        Value::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                let new_prefix = format!("{}[{}]", prefix, i);
                flatten(v, &new_prefix, vars);
            }
        }
        Value::String(s) => {
            vars.push(EnvVar {
                key: prefix.to_string(),
                value: s.clone(),
                default_value: s.clone(),
            });
        }
        Value::Number(n) => {
            let s = n.to_string();
            vars.push(EnvVar {
                key: prefix.to_string(),
                value: s.clone(),
                default_value: s.clone(),
            });
        }
        Value::Bool(b) => {
            let s = b.to_string();
            vars.push(EnvVar {
                key: prefix.to_string(),
                value: s.clone(),
                default_value: s.clone(),
            });
        }
        Value::Null => {
            vars.push(EnvVar {
                key: prefix.to_string(),
                value: "".to_string(),
                default_value: "".to_string(),
            });
        }
    }
}

// Removed unused update_leaf and update_leaf_value functions

impl FormatHandler for HierarchicalHandler {
    fn parse(&self, path: &Path) -> io::Result<Vec<EnvVar>> {
        let value = self.read_value(path)?;
        let mut vars = Vec::new();
        flatten(&value, "", &mut vars);
        Ok(vars)
    }

    fn merge(&self, path: &Path, vars: &mut Vec<EnvVar>) -> io::Result<()> {
        if !path.exists() {
            return Ok(());
        }
        let existing_value = self.read_value(path)?;
        let mut existing_vars = Vec::new();
        flatten(&existing_value, "", &mut existing_vars);

        for var in vars.iter_mut() {
            if let Some(existing) = existing_vars.iter().find(|v| v.key == var.key) {
                var.value = existing.value.clone();
            }
        }
        Ok(())
    }

    fn write(&self, path: &Path, vars: &[EnvVar]) -> io::Result<()> {
        // For writing hierarchical formats, we ideally want to preserve the original structure.
        // But we don't have it here. We should parse the template again to get the structure!
        // Oh wait, `write` is called with only `vars`.
        // If we want to construct the tree from scratch, it's very difficult to guess array vs object
        // and data types without the original template.
        // Let's change the trait or just keep a copy of the template?
        // Actually, if we require the user to have the template, we can just parse the template, update the leaves, and write.
        // We'll write a reconstruction algorithm that just creates objects based on keys.
        let mut root = Value::Object(Map::new());
        for var in vars {
            insert_into_value(&mut root, &var.key, &var.value);
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
        let next_node = map.entry(final_key.to_string()).or_insert_with(|| Value::Array(Vec::new()));
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
        let idx = part[start_idx+1..part.len()-1].parse::<usize>().ok();
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