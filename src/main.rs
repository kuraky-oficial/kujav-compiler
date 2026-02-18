mod core;
mod reader; // Añade esto

fn main() -> std::io::Result<()> {
    // --- NUEVA PRUEBA DE LECTURA ---
    // Intenta leer un JAR (ejemplo: el rt.jar de Java o cualquier librería)
    println!("🔎 Escaneando librerías...");
    if let Ok(meta) = reader::jar_reader::read_jar("D:\\Java\\lib\\plugin.jar") {
        for class in meta.class_names.iter().take(5) {
            println!("  Clase disponible: {}", class);
        }
    }
    
    // ... el resto de tu código de generación de bytes ...
    // (el que ya funciona y genera HolaKujav.class)
    Ok(())
}