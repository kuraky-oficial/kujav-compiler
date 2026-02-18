// src/compiler/codegen/expressions.rs
use crate::compiler::codegen::Compiler;
use crate::parser::ast::Expr;

impl Compiler {
    pub fn is_string_expr(&self, expr: &Expr) -> bool {
        match expr {
            Expr::String(_) => true,
            Expr::Identifier(n) => self.variable_types.get(n).map(|t| t.as_str()) == Some("Ljava/lang/String;"),
            Expr::Binary(l, op, r) => op == "+" && (self.is_string_expr(l) || self.is_string_expr(r)),
            _ => false,
        }
    }

    pub fn compile_expression(&mut self, expr: Expr) {
        match expr {
            Expr::Number(val) => { 
                self.current_bytecode.push(0x10); self.current_bytecode.push(val as u8); 
            }
            Expr::Boolean(val) => { 
                self.current_bytecode.push(if val { 0x04 } else { 0x03 }); 
            }
            Expr::String(c) => {
                let u_idx = self.cp.add_utf8(&c);
                let s_idx = self.cp.add_string(u_idx);
                self.current_bytecode.push(0x12); self.current_bytecode.push(s_idx as u8);
            }
            Expr::Identifier(n) => {
                if let Some(&slot) = self.variables.get(&n) {
                    let is_i = self.variable_types.get(&n).map(|s| s.as_str()) == Some("I");
                    self.current_bytecode.push(if is_i { 0x15 } else { 0x19 }); 
                    self.current_bytecode.push(slot);
                }
            }
            Expr::ArrayLiteral(elems) => {
                self.current_bytecode.push(0x10); self.current_bytecode.push(elems.len() as u8);
                self.current_bytecode.push(0xBC); self.current_bytecode.push(10); // newarray T_INT
                for (i, e) in elems.into_iter().enumerate() {
                    self.current_bytecode.push(0x59);
                    self.current_bytecode.push(0x10); self.current_bytecode.push(i as u8);
                    self.compile_expression(e);
                    self.current_bytecode.push(0x4F); // iastore
                }
            }
            Expr::ArrayAccess(name, idx) => {
                // CORRECCIÓN: name ya es String, no necesitamos Borrow de Box
                if let Some(&slot) = self.variables.get(&name) {
                    self.current_bytecode.push(0x19); self.current_bytecode.push(slot); // aload
                    self.compile_expression(*idx);
                    self.current_bytecode.push(0x2E); // iaload
                }
            }
            Expr::Binary(l, op, r) => {
                if op == "+" && (self.is_string_expr(&l) || self.is_string_expr(&r)) {
                    let sb_text = self.cp.add_utf8("java/lang/StringBuilder");
                    let sb_c = self.cp.add_class(sb_text);
                    
                    self.current_bytecode.push(0xBB); 
                    self.current_bytecode.extend_from_slice(&sb_c.to_be_bytes());
                    self.current_bytecode.push(0x59);
                    
                    let init_n = self.cp.add_utf8("<init>");
                    let init_s = self.cp.add_utf8("()V");
                    let nt_init = self.cp.add_name_and_type(init_n, init_s);
                    let m_init = self.cp.add_method_ref(sb_c, nt_init);
                    self.current_bytecode.push(0xB7); 
                    self.current_bytecode.extend_from_slice(&m_init.to_be_bytes());

                    let l_str = self.is_string_expr(&l);
                    self.compile_expression(*l);
                    let sig_l = if l_str { "(Ljava/lang/String;)Ljava/lang/StringBuilder;" } else { "(I)Ljava/lang/StringBuilder;" };
                    
                    let app_n = self.cp.add_utf8("append");
                    let app_s_l = self.cp.add_utf8(sig_l);
                    let nt_l = self.cp.add_name_and_type(app_n, app_s_l);
                    let m_app_l = self.cp.add_method_ref(sb_c, nt_l);
                    self.current_bytecode.push(0xB6); 
                    self.current_bytecode.extend_from_slice(&m_app_l.to_be_bytes());

                    let r_str = self.is_string_expr(&r);
                    self.compile_expression(*r);
                    let sig_r = if r_str { "(Ljava/lang/String;)Ljava/lang/StringBuilder;" } else { "(I)Ljava/lang/StringBuilder;" };
                    
                    let app_s_r = self.cp.add_utf8(sig_r);
                    let nt_r = self.cp.add_name_and_type(app_n, app_s_r);
                    let m_app_r = self.cp.add_method_ref(sb_c, nt_r);
                    self.current_bytecode.push(0xB6); 
                    self.current_bytecode.extend_from_slice(&m_app_r.to_be_bytes());

                    let ts_n = self.cp.add_utf8("toString");
                    let ts_s = self.cp.add_utf8("()Ljava/lang/String;");
                    let nt_ts = self.cp.add_name_and_type(ts_n, ts_s);
                    let m_ts = self.cp.add_method_ref(sb_c, nt_ts);
                    self.current_bytecode.push(0xB6); 
                    self.current_bytecode.extend_from_slice(&m_ts.to_be_bytes());
                } else {
                    self.compile_expression(*l); 
                    self.compile_expression(*r);
                    match op.as_str() {
                        "+" => self.current_bytecode.push(0x60),
                        "-" => self.current_bytecode.push(0x64),
                        "*" => self.current_bytecode.push(0x68),
                        "/" => self.current_bytecode.push(0x6C),
                        "==" => self.current_bytecode.extend_from_slice(&[0xA0, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                        "<"  => self.current_bytecode.extend_from_slice(&[0xA2, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                        ">"  => self.current_bytecode.extend_from_slice(&[0xA4, 0x00, 0x07, 0x04, 0xA7, 0x00, 0x04, 0x03]),
                        "and" => self.current_bytecode.push(0x7E),
                        "or"  => self.current_bytecode.push(0x80),
                        _ => {}
                    }
                }
            }
            Expr::Call(name, args) => {
                for arg in args { self.compile_expression(arg); }
                let sal_u = self.cp.add_utf8("Salida");
                let cls_idx = self.cp.add_class(sal_u);
                
                let call_n = self.cp.add_utf8(&name);
                let call_s = self.cp.add_utf8("()I");
                let nt_idx = self.cp.add_name_and_type(call_n, call_s);
                let m_ref = self.cp.add_method_ref(cls_idx, nt_idx);
                
                self.current_bytecode.push(0xB8); 
                self.current_bytecode.extend_from_slice(&m_ref.to_be_bytes());
            }
            Expr::Input => {
                let sc_u = self.cp.add_utf8("java/util/Scanner");
                let scan_c = self.cp.add_class(sc_u);
                self.current_bytecode.push(0xBB); 
                self.current_bytecode.extend_from_slice(&scan_c.to_be_bytes());
                self.current_bytecode.push(0x59);
                
                let sys_u = self.cp.add_utf8("java/lang/System");
                let sys_c = self.cp.add_class(sys_u);
                let in_n = self.cp.add_utf8("in");
                let in_s = self.cp.add_utf8("Ljava/io/InputStream;");
                let sys_in_nt = self.cp.add_name_and_type(in_n, in_s);
                let f_in = self.cp.add_field_ref(sys_c, sys_in_nt);
                self.current_bytecode.push(0xB2); 
                self.current_bytecode.extend_from_slice(&f_in.to_be_bytes());
                
                let init_n = self.cp.add_utf8("<init>");
                let init_s = self.cp.add_utf8("(Ljava/io/InputStream;)V");
                let nt_init = self.cp.add_name_and_type(init_n, init_s);
                let m_init = self.cp.add_method_ref(scan_c, nt_init);
                self.current_bytecode.push(0xB7); 
                self.current_bytecode.extend_from_slice(&m_init.to_be_bytes());
                
                let next_n = self.cp.add_utf8("nextInt");
                let next_s = self.cp.add_utf8("()I");
                let nt_next = self.cp.add_name_and_type(next_n, next_s);
                let m_next = self.cp.add_method_ref(scan_c, nt_next);
                self.current_bytecode.push(0xB6); 
                self.current_bytecode.extend_from_slice(&m_next.to_be_bytes());
            }
            
        }
    }
}