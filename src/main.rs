// src/main.rs
mod core; mod reader; mod parser; mod compiler;
use std::fs; use std::io::Write;
use crate::compiler::semantics::SemanticAnalyzer; // Importamos el nuevo analista

fn main() -> std::io::Result<()> {
    let source_code = r#"
        print "--- Calculadora de Tipos Kujav ---"
        let x = 10
        let mensaje = "El valor es: " + x
        print mensaje
    "#;

    println!("🔨 Iniciando proceso Kujav...");

    // 1. ANÁLISIS SINTÁCTICO (AST)
    let ast = parser::parse_to_ast(source_code);

    // 2. ANÁLISIS SEMÁNTICO (NUEVA FASE)
    // Aquí es donde Kujav "piensa" antes de actuar
    let mut analyzer = SemanticAnalyzer::new();
    println!("🔍 Verificando coherencia de tipos...");
    for stmt in &ast {
        if let Err(e) = analyzer.check_stmt(stmt) {
            eprintln!("❌ ERROR SEMÁNTICO: {}", e);
            std::process::exit(1); // Detenemos la compilación si hay errores de tipos
        }
    }

    // 3. GENERACIÓN DE CÓDIGO (BACKEND)
    println!("⚙️  Generando bytecode...");
    let mut kujav = compiler::codegen::Compiler::new();
    
    // Configuración de la clase...
    let cls_u = kujav.cp.add_utf8("Salida");
    let this_c = kujav.cp.add_class(cls_u);
    let obj_u = kujav.cp.add_utf8("java/lang/Object");
    let super_c = kujav.cp.add_class(obj_u);
    let m_n = kujav.cp.add_utf8("main");
    let m_t = kujav.cp.add_utf8("([Ljava/lang/String;)V");
    let c_a = kujav.cp.add_utf8("Code");

    for stmt in ast { 
        kujav.compile_statement(stmt); // Llama a la lógica en codegen/statements.rs
    }
    kujav.current_bytecode.push(0xB1); 

    // Escritura del archivo .class...
    let mut file = fs::File::create("Salida.class")?;
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x31])?; 
    file.write_all(&kujav.cp.to_bytes())?;
    file.write_all(&[0x00, 0x21])?;
    file.write_all(&this_c.to_be_bytes())?; file.write_all(&super_c.to_be_bytes())?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; 

    // ... (El resto de la lógica de escritura de métodos se mantiene igual)
    
    println!("✅ Compilación terminada. Salida.class lista.");
    Ok(())
}