use std::fs::File;
use std::io::Write;

// Estructura mínima de un archivo .class (JVM)
// Esto es "lo más difícil": escribir binario puro que la JVM entienda.
fn main() -> std::io::Result<()> {
    let class_name = "HolaKujav";
    let mut file = File::create(format!("{}.class", class_name))?;

    // Magia binaria: Los archivos .class siempre empiezan con 0xCAFEBABE
    let magic: [u8; 4] = [0xCA, 0xFE, 0xBA, 0xBE];
    file.write_all(&magic)?;

    // Versión de la JVM (ej. Java 8 es 52.0)
    file.write_all(&[0x00, 0x00])?; // minor version
    file.write_all(&[0x00, 0x34])?; // major version (52)

    println!("✅ ¡Archivo {}.class generado!", class_name);
    println!("Próximo paso: Llenar el 'Constant Pool' para que el archivo sea ejecutable.");
    
    Ok(())
}