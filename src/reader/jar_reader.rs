#![allow(dead_code)]
use std::fs::File;
use zip::ZipArchive;

pub struct JarMetadata {
    pub class_names: Vec<String>,
}

pub fn read_jar(path: &str) -> Result<JarMetadata, String> {
    let file = File::open(path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;
    let mut class_names = Vec::new();

    for i in 0..archive.len() {
        let file = archive.by_index(i).unwrap();
        if file.name().ends_with(".class") {
            class_names.push(file.name().replace(".class", ""));
        }
    }
    Ok(JarMetadata { class_names })
}
