mod core;
use crate::core::constant_pool::ConstantPool;
use std::fs::File;
use std::io::Write;

fn main() -> std::io::Result<()> {
    let mut cp = ConstantPool::new();

    // 1. Nombres de Clase
    let cls_utf8 = cp.add_utf8("HolaKujav");
    let this_class = cp.add_class(cls_utf8);
    let obj_utf8 = cp.add_utf8("java/lang/Object");
    let super_class = cp.add_class(obj_utf8);

    // 2. Referencias para Imprimir
    let sys_cls_utf8 = cp.add_utf8("java/lang/System");
    let sys_cls = cp.add_class(sys_cls_utf8);
    let out_name_utf8 = cp.add_utf8("out");
    let out_type_utf8 = cp.add_utf8("Ljava/io/PrintStream;");
    let out_nt = cp.add_name_and_type(out_name_utf8, out_type_utf8);
    let field_out = cp.add_field_ref(sys_cls, out_nt);

    let ps_cls_utf8 = cp.add_utf8("java/io/PrintStream");
    let ps_cls = cp.add_class(ps_cls_utf8);
    let println_name_utf8 = cp.add_utf8("println");
    let println_type_utf8 = cp.add_utf8("(Ljava/lang/String;)V");
    let println_nt = cp.add_name_and_type(println_name_utf8, println_type_utf8);
    let method_println = cp.add_method_ref(ps_cls, println_nt);

    // 3. El mensaje
    let msg_utf8 = cp.add_utf8("Hola desde Kujav!");
    let msg_str = cp.add_string(msg_utf8);

    // 4. Nombre del método y atributo Code
    let main_name_utf8 = cp.add_utf8("main");
    let main_type_utf8 = cp.add_utf8("([Ljava/lang/String;)V");
    let code_attr_utf8 = cp.add_utf8("Code");

    let mut file = File::create("HolaKujav.class")?;

    // --- ESCRITURA ---
    file.write_all(&[0xCA, 0xFE, 0xBA, 0xBE])?; // Magic
    file.write_all(&[0x00, 0x00, 0x00, 0x34])?; // Version
    file.write_all(&cp.to_bytes())?;           // Constant Pool
    file.write_all(&[0x00, 0x21])?;             // Access: Public
    file.write_all(&this_class.to_be_bytes())?;
    file.write_all(&super_class.to_be_bytes())?;
    file.write_all(&[0x00, 0x00])?;             // Interfaces
    file.write_all(&[0x00, 0x00])?;             // Fields

    // --- MÉTODOS (1: main) ---
    file.write_all(&[0x00, 0x01])?;             // Methods count
    file.write_all(&[0x00, 0x09])?;             // Public Static
    file.write_all(&main_name_utf8.to_be_bytes())?;
    file.write_all(&main_type_utf8.to_be_bytes())?;
    file.write_all(&[0x00, 0x01])?;             // Attributes count (Code)

    // --- ATRIBUTO CODE (Bytecode real) ---
    // Bytecode: getstatic (field_out), ldc (msg_str), invokevirtual (method_println), return
    let bytecode = [
        0xB2, (field_out >> 8) as u8, (field_out & 0xFF) as u8,
        0x12, (msg_str & 0xFF) as u8,
        0xB6, (method_println >> 8) as u8, (method_println & 0xFF) as u8,
        0xB1
    ];

    file.write_all(&code_attr_utf8.to_be_bytes())?;
    let attr_len: u32 = 12 + bytecode.len() as u32;
    file.write_all(&attr_len.to_be_bytes())?;
    file.write_all(&[0x00, 0x02])?; // max_stack
    file.write_all(&[0x00, 0x01])?; // max_locals
    file.write_all(&(bytecode.len() as u32).to_be_bytes())?;
    file.write_all(&bytecode)?;
    file.write_all(&[0x00, 0x00, 0x00, 0x00])?; // Exceptions & Sub-attributes

    file.write_all(&[0x00, 0x00])?; // Class Attributes

    println!("✅ ¡Kujav ha creado un ejecutable binario real!");
    Ok(())
}