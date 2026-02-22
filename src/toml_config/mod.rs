use std::collections::BTreeMap;
use std::fs;

use crate::errors::{KujavError, KujavResult};

#[derive(Debug)]
pub struct KujavToml {
    pub package: Package,
    pub dependencies: BTreeMap<String, String>,
    pub java: JavaConfig,
    pub minecraft: Option<MinecraftConfig>,
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub main: String,
    pub edition: String,
}

#[derive(Debug, Default)]
pub struct JavaConfig {
    pub classpath: Vec<String>,
}

#[derive(Debug)]
pub struct MinecraftConfig {
    pub plugin_name: String,
    pub plugin_version: String,
    pub main_class: String,
    pub api: String,
}

impl KujavToml {
    pub fn from_path(path: &str) -> KujavResult<Self> {
        let content =
            fs::read_to_string(path).map_err(|_| KujavError::toml("missing kujav.toml"))?;

        let mut section = String::new();
        let mut package = Package {
            name: String::new(),
            version: String::new(),
            main: String::new(),
            edition: String::new(),
        };
        let mut dependencies = BTreeMap::new();
        let mut java = JavaConfig::default();
        let mut minecraft = MinecraftConfig {
            plugin_name: String::new(),
            plugin_version: String::new(),
            main_class: String::new(),
            api: String::new(),
        };
        let mut saw_minecraft = false;

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line.trim_matches(&['[', ']'][..]).to_string();
                if section == "minecraft" {
                    saw_minecraft = true;
                }
                continue;
            }
            let (k, v) = line
                .split_once('=')
                .ok_or_else(|| KujavError::toml(format!("invalid line: {line}")))?;
            let key = k.trim();
            let raw_value = v.trim();

            match section.as_str() {
                "package" => {
                    let value = trim_quoted(raw_value);
                    match key {
                        "name" => package.name = value,
                        "version" => package.version = value,
                        "main" => package.main = value,
                        "edition" => package.edition = value,
                        _ => {}
                    }
                }
                "dependencies" => {
                    dependencies.insert(key.to_string(), trim_quoted(raw_value));
                }
                "java" => {
                    if key == "classpath" {
                        java.classpath = parse_toml_string_array(raw_value)?;
                    }
                }
                "minecraft" => {
                    let value = trim_quoted(raw_value);
                    match key {
                        "plugin_name" => minecraft.plugin_name = value,
                        "plugin_version" => minecraft.plugin_version = value,
                        "main_class" => minecraft.main_class = value,
                        "api" => minecraft.api = value,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if package.name.is_empty() || package.version.is_empty() || package.main.is_empty() {
            return Err(KujavError::toml("missing required [package] keys"));
        }

        let minecraft = if saw_minecraft {
            if minecraft.plugin_name.is_empty() {
                minecraft.plugin_name = package.name.clone();
            }
            if minecraft.plugin_version.is_empty() {
                minecraft.plugin_version = package.version.clone();
            }
            if minecraft.main_class.is_empty() || minecraft.api.is_empty() {
                return Err(KujavError::toml(
                    "[minecraft] requires main_class and api (paper|spigot api version string)",
                ));
            }
            Some(minecraft)
        } else {
            None
        };

        Ok(Self {
            package,
            dependencies,
            java,
            minecraft,
        })
    }
}

fn trim_quoted(value: &str) -> String {
    value.trim().trim_matches('"').to_string()
}

fn parse_toml_string_array(raw: &str) -> KujavResult<Vec<String>> {
    let trimmed = raw.trim();
    if !(trimmed.starts_with('[') && trimmed.ends_with(']')) {
        return Err(KujavError::toml(format!(
            "expected TOML string array, found: {raw}"
        )));
    }
    let inner = &trimmed[1..trimmed.len() - 1];
    if inner.trim().is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    for item in inner.split(',') {
        let value = item.trim();
        if !(value.starts_with('"') && value.ends_with('"')) {
            return Err(KujavError::toml(format!(
                "classpath entries must be quoted strings, found: {value}"
            )));
        }
        result.push(trim_quoted(value));
    }
    Ok(result)
}
