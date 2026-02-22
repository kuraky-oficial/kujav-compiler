use std::fs;
use std::io::Write;

use zip::write::FileOptions;

use crate::compiler::codegen::{Compiler, MethodInfo};
use crate::compiler::semantics::SemanticAnalyzer;
use crate::errors::{KujavError, KujavResult};
use crate::parser;

pub fn check_only(source: &str) -> KujavResult<()> {
    let ast = parser::parse_to_ast(source);
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast).map_err(KujavError::semantic)?;
    Ok(())
}

pub fn compile_to_class(class_name: &str, source: &str, out_path: &str) -> KujavResult<()> {
    let ast = parser::parse_to_ast(source);
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.analyze(&ast).map_err(KujavError::semantic)?;

    let mut kujav = Compiler::new();
    let cls_u = kujav.cp.add_utf8(class_name);
    let this_c = kujav.cp.add_class(cls_u);
    let obj_super_u = kujav.cp.add_utf8("java/lang/Object");
    let super_c = kujav.cp.add_class(obj_super_u);

    let m_n = kujav.cp.add_utf8("main");
    let m_t = kujav.cp.add_utf8("([Ljava/lang/String;)V");
    let c_a = kujav.cp.add_utf8("Code");

    for stmt in ast {
        kujav.compile_statement(stmt);
    }
    kujav.current_bytecode.push(0xB1);

    let mut file = fs::File::create(out_path)?;
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE, 0x00, 0x00, 0x00, 0x31])?;
    file.write_all(&kujav.cp.to_bytes())?;
    file.write_all(&[0x00, 0x21])?;
    file.write_all(&this_c.to_be_bytes())?;
    file.write_all(&super_c.to_be_bytes())?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?;

    let num_methods = (1 + kujav.methods.len()) as u16;
    file.write_all(&num_methods.to_be_bytes())?;

    write_main_method(&mut file, &kujav, m_n, m_t, c_a)?;
    for method in &kujav.methods {
        write_custom_method(&mut file, method, c_a)?;
    }

    file.write_all(&[0x00, 0x00])?;
    Ok(())
}

pub fn package_jar(class_name: &str, class_path: &str, jar_path: &str) -> KujavResult<()> {
    let file = fs::File::create(jar_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = FileOptions::default();

    zip.start_file("META-INF/MANIFEST.MF", options)
        .map_err(|e| KujavError::bytecode(e.to_string()))?;
    let manifest = format!("Manifest-Version: 1.0\nMain-Class: {class_name}\n\n");
    zip.write_all(manifest.as_bytes())?;

    zip.start_file(format!("{class_name}.class"), options)
        .map_err(|e| KujavError::bytecode(e.to_string()))?;
    zip.write_all(&fs::read(class_path)?)?;
    zip.finish()
        .map_err(|e| KujavError::bytecode(e.to_string()))?;
    Ok(())
}

fn write_main_method(
    file: &mut fs::File,
    kujav: &Compiler,
    m_n: u16,
    m_t: u16,
    c_a: u16,
) -> KujavResult<()> {
    file.write_all(&[0x00, 0x09])?;
    file.write_all(&m_n.to_be_bytes())?;
    file.write_all(&m_t.to_be_bytes())?;
    file.write_all(&[0x00, 0x01])?;
    file.write_all(&c_a.to_be_bytes())?;

    let main_len: u32 = 12 + kujav.current_bytecode.len() as u32;
    file.write_all(&main_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x0A, 0x00, 0x0A])?;
    file.write_all(&(kujav.current_bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&kujav.current_bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?;
    Ok(())
}

fn write_custom_method(file: &mut fs::File, m: &MethodInfo, c_a: u16) -> KujavResult<()> {
    file.write_all(&[0x00, 0x09])?;
    file.write_all(&m.name_idx.to_be_bytes())?;
    file.write_all(&m.sig_idx.to_be_bytes())?;
    file.write_all(&[0x00, 0x01])?;
    file.write_all(&c_a.to_be_bytes())?;

    let attr_len: u32 = 12 + m.bytecode.len() as u32;
    file.write_all(&attr_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x0A])?;
    file.write_all(&m.max_locals.to_be_bytes())?;
    file.write_all(&(m.bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&m.bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?;
    Ok(())
}
