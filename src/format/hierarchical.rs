use super::{ConfigItem, FormatHandler, FormatType, ItemStatus, ValueType, PathSegment};
use serde_json::{Map, Value};
use std::fs;
use std::path::Path;

pub struct HierarchicalHandler {
    format_type: FormatType,
}

impl HierarchicalHandler {
    pub fn new(format_type: FormatType) -> Self {
        Self { format_type }
    }

    fn read_value(&self, path: &Path) -> anyhow::Result<Value> {
        let content = fs::read_to_string(path)?;
        let value = match self.format_type {
            FormatType::Json => serde_json::from_str(&content)?,
            FormatType::Yaml => serde_yaml::from_str(&content)?,
            FormatType::Toml => toml::from_str(&content)?,
            FormatType::Xml => xml_to_json(&content)?,
            _ => unreachable!(),
        };
        Ok(value)
    }

    fn write_value(&self, path: &Path, value: &Value) -> anyhow::Result<()> {
        let content = match self.format_type {
            FormatType::Json => serde_json::to_string_pretty(value)?,
            FormatType::Yaml => serde_yaml::to_string(value)?,
            FormatType::Toml => {
                // toml requires the root to be a table
                if value.is_object() {
                    let toml_value: toml::Value = serde_json::from_value(value.clone())?;
                    toml::to_string_pretty(&toml_value)?
                } else {
                    anyhow::bail!("Root of TOML must be an object");
                }
            }
            FormatType::Xml => json_to_xml(value),
            _ => unreachable!(),
        };
        fs::write(path, content)?;
        Ok(())
    }
}

fn xml_to_json(content: &str) -> anyhow::Result<Value> {
    use quick_xml::reader::Reader;
    use quick_xml::events::Event;

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    
    fn parse_recursive(reader: &mut Reader<&[u8]>) -> anyhow::Result<Value> {
        let mut map = Map::new();
        let mut text = String::new();
        let mut buf = Vec::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    let val = parse_recursive(reader)?;
                    
                    if let Some(existing) = map.get_mut(&name) {
                        if let Some(arr) = existing.as_array_mut() {
                            arr.push(val);
                        } else {
                            let old = existing.take();
                            *existing = Value::Array(vec![old, val]);
                        }
                    } else {
                        map.insert(name, val);
                    }
                }
                Ok(Event::End(_)) => break,
                Ok(Event::Text(e)) => {
                    text.push_str(&String::from_utf8_lossy(e.as_ref()));
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buf.clear();
        }

        if map.is_empty() {
            if text.is_empty() {
                Ok(Value::Null)
            } else {
                Ok(Value::String(text))
            }
        } else {
            if !text.is_empty() {
                map.insert("$text".to_string(), Value::String(text));
            }
            Ok(Value::Object(map))
        }
    }

    // Move to the first start tag
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                let val = parse_recursive(&mut reader)?;
                let mut root = Map::new();
                root.insert(name, val);
                return Ok(Value::Object(root));
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buf.clear();
    }

    Ok(Value::Object(Map::new()))
}

fn json_to_xml(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut s = String::new();
            for (k, v) in map {
                if k == "$text" {
                    s.push_str(v.as_str().unwrap_or(""));
                } else if let Some(arr) = v.as_array() {
                    for item in arr {
                        s.push_str(&format!("<{}>", k));
                        s.push_str(&json_to_xml(item));
                        s.push_str(&format!("</{}>", k));
                    }
                } else {
                    s.push_str(&format!("<{}>", k));
                    s.push_str(&json_to_xml(v));
                    s.push_str(&format!("</{}>", k));
                }
            }
            s
        }
        Value::Array(arr) => {
            let mut s = String::new();
            for v in arr {
                s.push_str(&json_to_xml(v));
            }
            s
        }
        Value::String(v) => v.clone(),
        Value::Number(v) => v.to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Null => "".to_string(),
    }
}

fn flatten(value: &Value, current_path: Vec<PathSegment>, key_name: Option<String>, depth: usize, vars: &mut Vec<ConfigItem>) {
    let mut next_path = current_path.clone();
    
    if let Some(ref k) = key_name {
        if !current_path.is_empty() {
            // It's a key in an object, so append to path
            next_path.push(PathSegment::Key(k.clone()));
        } else {
            // First element, maybe root
            if !k.is_empty() {
                next_path.push(PathSegment::Key(k.clone()));
            }
        }
    }

    let display_key = match next_path.last() {
        Some(PathSegment::Key(k)) => k.clone(),
        Some(PathSegment::Index(i)) => format!("[{}]", i),
        None => "".to_string(),
    };

    match value {
        Value::Object(map) => {
            if !next_path.is_empty() {
                vars.push(ConfigItem {
                    key: display_key,
                    path: next_path.clone(),
                    value: None,
                    template_value: None,
                    default_value: None,
                    depth,
                    is_group: true,
                    status: ItemStatus::Present,
                    value_type: ValueType::Null,
                });
            }
            let next_depth = if next_path.is_empty() { depth } else { depth + 1 };
            for (k, v) in map {
                flatten(v, next_path.clone(), Some(k.clone()), next_depth, vars);
            }
        }
        Value::Array(arr) => {
            if !next_path.is_empty() {
                vars.push(ConfigItem {
                    key: display_key,
                    path: next_path.clone(),
                    value: None,
                    template_value: None,
                    default_value: None,
                    depth,
                    is_group: true,
                    status: ItemStatus::Present,
                    value_type: ValueType::Null,
                });
            }
            let next_depth = if next_path.is_empty() { depth } else { depth + 1 };
            for (i, v) in arr.iter().enumerate() {
                let mut arr_path = next_path.clone();
                arr_path.push(PathSegment::Index(i));
                flatten(v, arr_path, None, next_depth, vars);
            }
        }
        Value::String(s) => {
            vars.push(ConfigItem {
                key: display_key,
                path: next_path.clone(),
                value: Some(s.clone()),
                template_value: Some(s.clone()),
                default_value: Some(s.clone()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
                value_type: ValueType::String,
            });
        }
        Value::Number(n) => {
            let s = n.to_string();
            vars.push(ConfigItem {
                key: display_key,
                path: next_path.clone(),
                value: Some(s.clone()),
                template_value: Some(s.clone()),
                default_value: Some(s.clone()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
                value_type: ValueType::Number,
            });
        }
        Value::Bool(b) => {
            let s = b.to_string();
            vars.push(ConfigItem {
                key: display_key,
                path: next_path.clone(),
                value: Some(s.clone()),
                template_value: Some(s.clone()),
                default_value: Some(s.clone()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
                value_type: ValueType::Bool,
            });
        }
        Value::Null => {
            vars.push(ConfigItem {
                key: display_key,
                path: next_path.clone(),
                value: Some("".to_string()),
                template_value: Some("".to_string()),
                default_value: Some("".to_string()),
                depth,
                is_group: false,
                status: ItemStatus::Present,
                value_type: ValueType::Null,
            });
        }
    }
}

impl FormatHandler for HierarchicalHandler {
    fn parse(&self, path: &Path) -> anyhow::Result<Vec<ConfigItem>> {
        let value = self.read_value(path)?;
        let mut vars = Vec::new();
        flatten(&value, Vec::new(), Some("".to_string()), 0, &mut vars);
        Ok(vars)
    }

    fn write(&self, path: &Path, vars: &[ConfigItem]) -> anyhow::Result<()> {
        let mut root = Value::Object(Map::new());
        for var in vars {
            if !var.is_group {
                let val = var.value.as_deref()
                    .or(var.template_value.as_deref())
                    .unwrap_or("");
                insert_into_value(&mut root, &var.path, val, var.value_type);
            }
        }
        self.write_value(path, &root)
    }
}

fn insert_into_value(root: &mut Value, path: &[PathSegment], new_val_str: &str, value_type: ValueType) {
    if path.is_empty() {
        return;
    }

    let mut current = root;
    
    // Traverse all but the last segment
    for i in 0..path.len() - 1 {
        let segment = &path[i];
        let next_segment = &path[i + 1];

        match segment {
            PathSegment::Key(key) => {
                if !current.is_object() {
                    *current = Value::Object(Map::new());
                }
                let map = current.as_object_mut().unwrap();
                
                let next_node = map.entry(key.clone()).or_insert_with(|| {
                    match next_segment {
                        PathSegment::Index(_) => Value::Array(Vec::new()),
                        PathSegment::Key(_) => Value::Object(Map::new()),
                    }
                });
                current = next_node;
            }
            PathSegment::Index(idx) => {
                if !current.is_array() {
                    *current = Value::Array(Vec::new());
                }
                let arr = current.as_array_mut().unwrap();
                while arr.len() <= *idx {
                    match next_segment {
                        PathSegment::Index(_) => arr.push(Value::Array(Vec::new())),
                        PathSegment::Key(_) => arr.push(Value::Object(Map::new())),
                    }
                }
                current = &mut arr[*idx];
            }
        }
    }

    // Handle the final segment
    let final_segment = &path[path.len() - 1];
    
    let final_val = match value_type {
        ValueType::Number => {
            if let Ok(n) = new_val_str.parse::<i64>() {
                Value::Number(n.into())
            } else if let Ok(f) = new_val_str.parse::<f64>() {
                if let Some(n) = serde_json::Number::from_f64(f) {
                    Value::Number(n)
                } else {
                    Value::String(new_val_str.to_string())
                }
            } else {
                Value::String(new_val_str.to_string())
            }
        }
        ValueType::Bool => {
            if let Ok(b) = new_val_str.parse::<bool>() {
                Value::Bool(b)
            } else {
                Value::String(new_val_str.to_string())
            }
        }
        ValueType::Null if new_val_str.is_empty() => Value::Null,
        _ => Value::String(new_val_str.to_string()),
    };

    match final_segment {
        PathSegment::Key(key) => {
            if !current.is_object() {
                *current = Value::Object(Map::new());
            }
            let map = current.as_object_mut().unwrap();
            map.insert(key.clone(), final_val);
        }
        PathSegment::Index(idx) => {
            if !current.is_array() {
                *current = Value::Array(Vec::new());
            }
            let arr = current.as_array_mut().unwrap();
            while arr.len() <= *idx {
                arr.push(Value::Null);
            }
            arr[*idx] = final_val;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_flatten_unflatten() {
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

        flatten(&json, Vec::new(), Some("".to_string()), 0, &mut vars);
        assert_eq!(vars.len(), 6);

        let mut root = Value::Object(Map::new());
        for var in vars {
            if !var.is_group {
                insert_into_value(&mut root, &var.path, var.value.as_deref().unwrap_or(""), var.value_type);
            }
        }

        let unflattened_json = serde_json::to_string(&root).unwrap();
        assert!(unflattened_json.contains("\"8080:80\""));
        assert!(unflattened_json.contains("true"));
    }

    #[test]
    fn test_type_preservation() {
        let mut vars = Vec::new();
        let json = serde_json::json!({
            "port_num": 8080,
            "port_str": "8080",
            "is_enabled": true,
            "is_enabled_str": "true",
            "float_num": 42.42,
            "float_str": "42.42"
        });

        flatten(&json, Vec::new(), Some("".to_string()), 0, &mut vars);
        
        let mut root = Value::Object(Map::new());
        for var in vars {
            if !var.is_group {
                insert_into_value(&mut root, &var.path, var.value.as_deref().unwrap_or(""), var.value_type);
            }
        }

        let unflattened = root.as_object().unwrap();
        
        assert!(unflattened["port_num"].is_number(), "port_num should be a number");
        assert_eq!(unflattened["port_num"].as_i64(), Some(8080));
        
        assert!(unflattened["port_str"].is_string(), "port_str should be a string");
        assert_eq!(unflattened["port_str"].as_str(), Some("8080"));
        
        assert!(unflattened["is_enabled"].is_boolean(), "is_enabled should be a boolean");
        assert_eq!(unflattened["is_enabled"].as_bool(), Some(true));
        
        assert!(unflattened["is_enabled_str"].is_string(), "is_enabled_str should be a string");
        assert_eq!(unflattened["is_enabled_str"].as_str(), Some("true"));
        
        assert!(unflattened["float_num"].is_number(), "float_num should be a number");
        assert_eq!(unflattened["float_num"].as_f64(), Some(42.42));
        
        assert!(unflattened["float_str"].is_string(), "float_str should be a string");
        assert_eq!(unflattened["float_str"].as_str(), Some("42.42"));
    }

    #[test]
    fn test_yaml_flatten_unflatten() {
        let yaml_str = "
server:
  port: 8080
  port_str: \"8080\"
  enabled: true
";
        let yaml_val: Value = serde_yaml::from_str(yaml_str).unwrap();
        let mut vars = Vec::new();
        flatten(&yaml_val, Vec::new(), Some("".to_string()), 0, &mut vars);
        
        let mut root = Value::Object(Map::new());
        for var in vars {
            if !var.is_group {
                insert_into_value(&mut root, &var.path, var.value.as_deref().unwrap_or(""), var.value_type);
            }
        }
        
        let unflattened_yaml = serde_yaml::to_string(&root).unwrap();
        assert!(unflattened_yaml.contains("port: 8080"));
        assert!(unflattened_yaml.contains("port_str: '8080'") || unflattened_yaml.contains("port_str: \"8080\""));
        assert!(unflattened_yaml.contains("enabled: true"));
    }

    #[test]
    fn test_toml_flatten_unflatten() {
        let toml_str = "
[server]
port = 8080
port_str = \"8080\"
enabled = true
";
        let toml_val: toml::Value = toml::from_str(toml_str).unwrap();
        let json_val: Value = serde_json::to_value(toml_val).unwrap();

        let mut vars = Vec::new();
        flatten(&json_val, Vec::new(), Some("".to_string()), 0, &mut vars);
        
        let mut root = Value::Object(Map::new());
        for var in vars {
            if !var.is_group {
                insert_into_value(&mut root, &var.path, var.value.as_deref().unwrap_or(""), var.value_type);
            }
        }
        
        let toml_root: toml::Value = serde_json::from_value(root).unwrap();
        let unflattened_toml = toml::to_string(&toml_root).unwrap();
        
        assert!(unflattened_toml.contains("port = 8080"));
        assert!(unflattened_toml.contains("port_str = \"8080\""));
        assert!(unflattened_toml.contains("enabled = true"));
    }

    #[test]
    fn test_xml_flatten_unflatten() {
        let xml_str = "<config><server><port>8080</port><enabled>true</enabled></server></config>";
        
        let json_val = xml_to_json(xml_str).unwrap();

        let mut vars = Vec::new();
        flatten(&json_val, Vec::new(), Some("".to_string()), 0, &mut vars);
        
        let mut root = Value::Object(Map::new());
        for var in vars {
            if !var.is_group {
                insert_into_value(&mut root, &var.path, var.value.as_deref().unwrap_or(""), var.value_type);
            }
        }
        
        let unflattened_xml = json_to_xml(&root);

        assert!(unflattened_xml.contains("<port>8080</port>"));
        assert!(unflattened_xml.contains("<enabled>true</enabled>"));
        assert!(unflattened_xml.contains("<config>") && unflattened_xml.contains("</config>"));
    }
}
