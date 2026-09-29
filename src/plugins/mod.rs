pub mod engine;
pub mod ipc;

pub use engine::PluginEngine;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub enabled: bool,
    pub entry: Option<String>,
}

impl PluginMetadata {
    pub fn from_json(json: &str) -> Option<Self> {
        let id = extract_json_field(json, "id")?;
        let name = extract_json_field(json, "name").unwrap_or_else(|| id.clone());
        let version = extract_json_field(json, "version").unwrap_or_else(|| "1.0.0".to_string());
        let author = extract_json_field(json, "author").unwrap_or_else(|| "Неизвестный автор".to_string());
        let description = extract_json_field(json, "description").unwrap_or_else(|| "".to_string());
        let enabled = extract_json_bool(json, "enabled").unwrap_or(true);
        let entry = extract_json_field(json, "entry");

        Some(Self {
            id,
            name,
            version,
            author,
            description,
            enabled,
            entry,
        })
    }

    pub fn to_json(&self) -> String {
        let entry_line = if let Some(ref e) = self.entry {
            format!(",\n  \"entry\": \"{}\"", escape_json(e))
        } else {
            "".to_string()
        };

        format!(
            "{{\n  \"id\": \"{}\",\n  \"name\": \"{}\",\n  \"version\": \"{}\",\n  \"author\": \"{}\",\n  \"description\": \"{}\",\n  \"enabled\": {}{}\n}}\n",
            escape_json(&self.id),
            escape_json(&self.name),
            escape_json(&self.version),
            escape_json(&self.author),
            escape_json(&self.description),
            self.enabled,
            entry_line
        )
    }
}

fn extract_json_field(json: &str, field: &str) -> Option<String> {
    let key = format!("\"{}\"", field);
    let key_pos = json.find(&key)?;
    let after_key = &json[key_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();

    if !after_colon.starts_with('"') {
        return None;
    }

    let mut result = String::new();
    let mut chars = after_colon[1..].chars();
    let mut escaped = false;

    for c in chars.by_ref() {
        if escaped {
            match c {
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                '"' => result.push('"'),
                other => result.push(other),
            }
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            return Some(result);
        } else {
            result.push(c);
        }
    }
    None
}

fn extract_json_bool(json: &str, field: &str) -> Option<bool> {
    let key = format!("\"{}\"", field);
    let key_pos = json.find(&key)?;
    let after_key = &json[key_pos + key.len()..];
    let colon_pos = after_key.find(':')?;
    let after_colon = after_key[colon_pos + 1..].trim_start();

    if after_colon.starts_with("true") {
        Some(true)
    } else if after_colon.starts_with("false") {
        Some(false)
    } else {
        None
    }
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

pub struct PluginManager;

impl PluginManager {
    pub fn plugins_dir() -> PathBuf {
        PluginEngine::plugins_dir()
    }

    pub fn list_plugins() -> Vec<PluginMetadata> {
        let engine = PluginEngine::global().lock().unwrap();
        engine.list_plugins()
    }

    pub fn install_plug_file(file_path: &Path) -> Result<PluginMetadata, String> {
        let mut engine = PluginEngine::global().lock().unwrap();
        engine.install_plugin(file_path)
    }

    pub fn set_plugin_enabled(id: &str, enabled: bool) -> Result<(), String> {
        let mut engine = PluginEngine::global().lock().unwrap();
        engine.set_plugin_enabled(id, enabled)
    }

    pub fn delete_plugin(id: &str) -> Result<(), String> {
        let mut engine = PluginEngine::global().lock().unwrap();
        engine.delete_plugin(id)
    }

    pub fn launch_plugin(id: &str) -> Result<(), String> {
        let mut engine = PluginEngine::global().lock().unwrap();
        engine.launch_plugin(id)
    }
}
