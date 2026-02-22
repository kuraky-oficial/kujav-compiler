use std::collections::BTreeMap;
use std::fs;

use crate::errors::{KujavError, KujavResult};

#[derive(Debug)]
pub struct KujavToml {
    pub package: Package,
    pub dependencies: BTreeMap<String, String>,
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub main: String,
    pub edition: String,
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

        for raw_line in content.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if line.starts_with('[') && line.ends_with(']') {
                section = line.trim_matches(&['[', ']'][..]).to_string();
                continue;
            }
            let (k, v) = line
                .split_once('=')
                .ok_or_else(|| KujavError::toml(format!("invalid line: {line}")))?;
            let key = k.trim();
            let value = v.trim().trim_matches('"').to_string();

            match section.as_str() {
                "package" => match key {
                    "name" => package.name = value,
                    "version" => package.version = value,
                    "main" => package.main = value,
                    "edition" => package.edition = value,
                    _ => {}
                },
                "dependencies" => {
                    dependencies.insert(key.to_string(), value);
                }
                _ => {}
            }
        }

        if package.name.is_empty() || package.version.is_empty() || package.main.is_empty() {
            return Err(KujavError::toml("missing required [package] keys"));
        }
        Ok(Self {
            package,
            dependencies,
        })
    }
}
