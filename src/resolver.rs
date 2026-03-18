use std::path::{Path, PathBuf};

pub struct Rule {
    pub template_suffix: &'static str,
    pub active_suffix: &'static str,
    pub is_exact_match: bool,
}

pub const RULES: &[Rule] = &[
    // Exact matches
    Rule { template_suffix: "compose.yml", active_suffix: "compose.override.yml", is_exact_match: true },
    Rule { template_suffix: "compose.yaml", active_suffix: "compose.override.yaml", is_exact_match: true },
    Rule { template_suffix: "docker-compose.yml", active_suffix: "docker-compose.override.yml", is_exact_match: true },
    Rule { template_suffix: "docker-compose.yaml", active_suffix: "docker-compose.override.yaml", is_exact_match: true },
    
    // Pattern matches
    Rule { template_suffix: ".env.example", active_suffix: ".env", is_exact_match: false },
    Rule { template_suffix: ".env.template", active_suffix: ".env", is_exact_match: false },
    Rule { template_suffix: ".example.json", active_suffix: ".json", is_exact_match: false },
    Rule { template_suffix: ".template.json", active_suffix: ".json", is_exact_match: false },
    Rule { template_suffix: ".example.yml", active_suffix: ".yml", is_exact_match: false },
    Rule { template_suffix: ".template.yml", active_suffix: ".yml", is_exact_match: false },
    Rule { template_suffix: ".example.yaml", active_suffix: ".yaml", is_exact_match: false },
    Rule { template_suffix: ".template.yaml", active_suffix: ".yaml", is_exact_match: false },
    Rule { template_suffix: ".example.toml", active_suffix: ".toml", is_exact_match: false },
    Rule { template_suffix: ".template.toml", active_suffix: ".toml", is_exact_match: false },
    Rule { template_suffix: ".example.xml", active_suffix: ".xml", is_exact_match: false },
    Rule { template_suffix: ".template.xml", active_suffix: ".xml", is_exact_match: false },
    Rule { template_suffix: ".example.ini", active_suffix: ".ini", is_exact_match: false },
    Rule { template_suffix: ".template.ini", active_suffix: ".ini", is_exact_match: false },
    Rule { template_suffix: ".example.properties", active_suffix: ".properties", is_exact_match: false },
    Rule { template_suffix: ".template.properties", active_suffix: ".properties", is_exact_match: false },
];

pub const DEFAULT_CANDIDATES: &[&str] = &[
    ".env.example",
    "compose.yml",
    "docker-compose.yml",
    ".env.template",
    "compose.yaml",
    "docker-compose.yaml",
];

/// Helper to automatically determine the output file path based on common naming conventions.
pub fn determine_output_path(input: &Path) -> PathBuf {
    let file_name = input.file_name().unwrap_or_default().to_string_lossy();

    for rule in RULES {
        if rule.is_exact_match {
            if file_name == rule.template_suffix {
                return input.with_file_name(rule.active_suffix);
            }
        } else if file_name == rule.template_suffix {
            return input.with_file_name(rule.active_suffix);
        } else if let Some(base) = file_name.strip_suffix(rule.template_suffix) {
            return input.with_file_name(format!("{}{}", base, rule.active_suffix));
        }
    }

    input.with_extension(format!(
        "{}.out",
        input.extension().unwrap_or_default().to_string_lossy()
    ))
}

/// Discovers common configuration template files in the current directory.
pub fn find_input_file() -> Option<PathBuf> {
    // Priority 1: Exact matches for well-known defaults
    for &name in DEFAULT_CANDIDATES {
        let path = PathBuf::from(name);
        if path.exists() {
            return Some(path);
        }
    }

    // Priority 2: Pattern matches
    if let Ok(entries) = std::fs::read_dir(".") {
        let mut fallback = None;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();

            for rule in RULES {
                if !rule.is_exact_match && name_str.ends_with(rule.template_suffix) {
                    if name_str.contains(".env") || name_str.contains("compose") {
                        return Some(entry.path());
                    }
                    if fallback.is_none() {
                        fallback = Some(entry.path());
                    }
                    break;
                }
            }
        }
        if let Some(path) = fallback {
            return Some(path);
        }
    }

    None
}

/// Resolves the active and template paths given an input path.
/// Returns `(active_path, template_path)`.
pub fn resolve_paths(input: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    let file_name = input.file_name().unwrap_or_default().to_string_lossy();
    
    // Check if the input matches any known template pattern
    let mut is_template = false;
    for rule in RULES {
        if rule.is_exact_match {
            if file_name == rule.template_suffix {
                is_template = true;
                break;
            }
        } else if file_name.ends_with(rule.template_suffix) {
            is_template = true;
            break;
        }
    }

    // Fallback template detection
    if !is_template && (file_name.contains(".example") || file_name.contains(".template")) {
        is_template = true;
    }

    if is_template {
        let expected_active = determine_output_path(input);
        let active = if expected_active.exists() {
            Some(expected_active)
        } else {
            None
        };
        (active, Some(input.to_path_buf()))
    } else {
        // Input is treated as the active config
        let active = Some(input.to_path_buf());
        let mut template = None;
        
        // Try to reverse match rules to find a template
        for rule in RULES {
            if rule.is_exact_match {
                if file_name == rule.active_suffix {
                    let t = input.with_file_name(rule.template_suffix);
                    if t.exists() {
                        template = Some(t);
                        break;
                    }
                }
            } else if file_name.ends_with(rule.active_suffix) {
                if file_name == rule.active_suffix {
                    let t = input.with_file_name(rule.template_suffix);
                    if t.exists() {
                        template = Some(t);
                        break;
                    }
                } else if let Some(base) = file_name.strip_suffix(rule.active_suffix) {
                    let t = input.with_file_name(format!("{}{}", base, rule.template_suffix));
                    if t.exists() {
                        template = Some(t);
                        break;
                    }
                }
            }
        }
        
        // Fallback reverse detection
        if template.is_none() {
            let possible_templates = [
                format!("{}.example", file_name),
                format!("{}.template", file_name),
            ];
            for t in possible_templates {
                let p = input.with_file_name(t);
                if p.exists() {
                    template = Some(p);
                    break;
                }
            }
        }

        (active, template)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_output_path() {
        assert_eq!(determine_output_path(Path::new(".env.example")), PathBuf::from(".env"));
        assert_eq!(determine_output_path(Path::new("compose.yml")), PathBuf::from("compose.override.yml"));
        assert_eq!(determine_output_path(Path::new("config.template.json")), PathBuf::from("config.json"));
        assert_eq!(determine_output_path(Path::new("config.example")), PathBuf::from("config.example.out"));
    }
}
