mod core;
use crate::core::constant_pool::ConstantPool;
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut cp = ConstantPool::new();
    
    // 1. Llenamos el Constant Pool con lo mínimo necesario
    let class_name_utf8 = cp.add_utf8("HolaKujav");
    let this_class = cp.add_class(class_name_utf8);
    
    let super_name_utf8 = cp.add_utf8("java/lang/Object");
    let super_class = cp.add_class(super_name_utf8);

    let mut file = File::create("HolaKujav.class")?;

    // --- ESCRIBIENDO EL BINARIO ---
    // Magic & Version (Ya lo tenías)
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE])?; 
    file.write_all(&[0x00, 0x00, 0x00, 0x34])?; // Java 8 (v52)

    // Escribir Constant Pool
    file.write_all(&cp.to_bytes())?;

    // Access Flags: 0x0021 (Public + Super)
    file.write_all(&[0x00, 0x21])?;

    // This Class & Super Class
    file.write_all(&this_class.to_be_bytes())?;
    file.write_all(&super_class.to_be_bytes())?;

    // Interfaces, Fields, Methods, Attributes (Todos en 0 por ahora)
    file.write_all(&[0x00, 0x00])?; // Interfaces count
    file.write_all(&[0x00, 0x00])?; // Fields count
    file.write_all(&[0x00, 0x00])?; // Methods count
    file.write_all(&[0x00, 0x00])?; // Attributes count

    println!("✅ Clase 'HolaKujav' generada con estructura legal de la JVM.");
    Ok(())
}