// src/main.rs
mod core; mod reader; mod parser; mod compiler;
use std::fs; use std::io::Write;

fn main() -> std::io::Result<()> {
    // Ejemplo potente: Una función que calcula el cuadrado de un número y lo devuelve
    let source_code = r#"
        fun cuadrado(n) {
            return n * n
        }

        let x = 5
        let resultado = cuadrado(x)
        
        print "El cuadrado de 5 es:"
        print resultado
    "#;

    println!("🔨 Compilando Kujav...");
    
    // 1. Análisis y generación del AST
    let ast = parser::parse_to_ast(source_code);
    let mut kujav = compiler::codegen::Compiler::new();
    
    // 2. Registro de constantes iniciales para la estructura de la clase JVM
    let cls_u = kujav.cp.add_utf8("Salida");
    let this_c = kujav.cp.add_class(cls_u);
    let obj_u = kujav.cp.add_utf8("java/lang/Object");
    let super_c = kujav.cp.add_class(obj_u);
    let m_n = kujav.cp.add_utf8("main");
    let m_t = kujav.cp.add_utf8("([Ljava/lang/String;)V");
    let c_a = kujav.cp.add_utf8("Code");

    // 3. Compilación de cada declaración del AST
    for stmt in ast { 
        kujav.compile_statement(stmt); 
    }
    
    // 4. Aseguramos el retorno final del método main
    kujav.current_bytecode.push(0xB1); 

    // 5. Creación del archivo .class siguiendo la especificación de la JVM
    let mut file = fs::File::create("Salida.class")?;
    
    // Magic Number y Versión (Java 5 / 49.0)
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x31])?; 
    
    // Pool de constantes (Constant Pool)
    file.write_all(&kujav.cp.to_bytes())?;
    
    // Access flags, this_class, super_class
    file.write_all(&[0x00, 0x21])?; // public class
    file.write_all(&this_c.to_be_bytes())?; 
    file.write_all(&super_c.to_be_bytes())?;
    
    // Interfaces y Campos (vacíos por ahora)
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; 

    // 6. MÉTODOS: Escribir el método 'main' y las funciones adicionales 'fun'
    let num_methods = (1 + kujav.methods.len()) as u16;
    file.write_all(&num_methods.to_be_bytes())?;

    // Escribir el método 'main'
    file.write_all(&[0x00, 0x09])?; // public static
    file.write_all(&m_n.to_be_bytes())?; 
    file.write_all(&m_t.to_be_bytes())?;
    file.write_all(&[0x00, 0x01])?; // attributes_count (Code)
    file.write_all(&c_a.to_be_bytes())?;
    
    let main_len: u32 = 12 + kujav.current_bytecode.len() as u32;
    file.write_all(&main_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x0A, 0x00, 0x0A])?; // max_stack y max_locals
    file.write_all(&(kujav.current_bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav.current_bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; // attributes del code

    // Escribir todas las funciones 'fun' definidas en el código Kujav
    for m in &kujav.methods {
        file.write_all(&[0x00, 0x09])?; // public static
        file.write_all(&m.name_idx.to_be_bytes())?; 
        file.write_all(&m.sig_idx.to_be_bytes())?;
        file.write_all(&[0x00, 0x01])?; // attributes_count (Code)
        file.write_all(&c_a.to_be_bytes())?;
        
        let attr_len: u32 = 12 + m.bytecode.len() as u32;
        file.write_all(&attr_len.to_be_bytes())?;
        file.write_all(&[0x00, 0x0A])?; // max_stack
        file.write_all(&m.max_locals.to_be_bytes())?;
        file.write_all(&(m.bytecode.len() as u32).to_be_bytes())?;
        file.write_all(&m.bytecode)?;
        file.write_all(&[0x00, 0x00, 0x00, 0x00])?;
    }

    // Atributos de clase (vacíos)
    file.write_all(&[0x00, 0x00])?; 
    
    println!("✅ Salida.class generada con éxito.");
    Ok(())
}