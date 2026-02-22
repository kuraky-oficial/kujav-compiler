use std::path::Path;

use crate::errors::{KujavError, KujavResult};
use crate::toml_config::KujavToml;

pub fn validate_java_classpath(cfg: &KujavToml) -> KujavResult<()> {
    for dep in &cfg.java.classpath {
        if !Path::new(dep).exists() {
            return Err(KujavError::dependency(format!(
                "java classpath dependency not found: {dep}"
            )));
        }
    }
    Ok(())
}
