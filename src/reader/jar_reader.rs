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
        let mut file = archive.by_index(i).unwrap();
        if file.name().ends_with(".class") {
            // Guardamos el nombre de la clase (quitando el .class)
            let name = file.name().replace(".class", "");
            class_names.push(name);
            
            // Aquí es donde usaríamos cafebabe para leer métodos más adelante
            /*
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).unwrap();
            if let Ok(class) = parse_class(&buffer) {
                // println!("Clase encontrada: {}", class.this_class);
            }
            */
        }
    }

    Ok(JarMetadata { class_names })
}