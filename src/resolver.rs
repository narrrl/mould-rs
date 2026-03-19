//! Automatically resolves relationships between template and active configuration files.
//!
//! The resolver allows `mould` to be run without explicit output arguments 
//! by intelligently guessing the counterpart of a given input file based 
//! on common naming conventions.

use std::path::{Path, PathBuf};

/// Logic for determining which files to parse and where to save the results.
pub struct TemplateResolver;

impl TemplateResolver {
    /// Determines the template and output paths based on the provided input.
    ///
    /// If an output path is explicitly provided via CLI arguments, it is used. 
    /// Otherwise, the resolver applies a set of heuristic rules to find a matching pairing.
    pub fn resolve(
        input: &Path,
        output_override: Option<PathBuf>,
    ) -> (PathBuf, PathBuf) {
        if let Some(out) = output_override {
            return (input.to_path_buf(), out);
        }

        // Apply automatic discovery rules based on file name patterns.
        if let Some((template, output)) = Self::discover_pairing(input) {
            (template, output)
        } else {
            // Fallback: If no pairing is found, use the input as both 
            // the template source and the save target.
            (input.to_path_buf(), input.to_path_buf())
        }
    }

    /// Attempts to find a known template/active pairing for a given file path.
    ///
    /// Naming Rules Applied:
    /// 1. `.env.example` <-> `.env` (Standard environment file pattern).
    /// 2. `compose.yml` -> `compose.override.yml` (Docker Compose convention).
    /// 3. `<name>.template.<ext>` -> `<name>.<ext>` (General template pattern).
    /// 4. `<name>.<ext>.example` -> `<name>.<ext>` (General example pattern).
    fn discover_pairing(path: &Path) -> Option<(PathBuf, PathBuf)> {
        let file_name = path.file_name()?.to_str()?;

        // Rule 1: Standard .env pairing
        if file_name == ".env" || file_name == ".env.example" {
            let dir = path.parent().unwrap_or_else(|| Path::new("."));
            return Some((dir.join(".env.example"), dir.join(".env")));
        }

        // Rule 2: Docker Compose pairing
        if file_name == "docker-compose.yml" || file_name == "docker-compose.yaml" || file_name == "compose.yml" {
            let dir = path.parent().unwrap_or_else(|| Path::new("."));
            let override_file = if file_name == "compose.yml" {
                "compose.override.yml"
            } else {
                "docker-compose.override.yml"
            };
            return Some((path.to_path_buf(), dir.join(override_file)));
        }

        // Rule 3: .template or .example suffix removal
        if file_name.contains(".template.") {
            let output_name = file_name.replace(".template.", ".");
            return Some((path.to_path_buf(), path.with_file_name(output_name)));
        }
        
        if file_name.ends_with(".example") {
            let output_name = &file_name[..file_name.len() - 8];
            return Some((path.to_path_buf(), path.with_file_name(output_name)));
        }

        // Inverse Rule 3: If running against the active file, look for the template counterpart.
        let template_candidates = [
            format!("{}.example", file_name),
            file_name.replace('.', ".template."),
        ];

        for t in template_candidates {
            let p = path.with_file_name(t);
            if p.exists() {
                return Some((p, path.to_path_buf()));
            }
        }

        None
    }
}
