use std::rc::Rc;

use serde::{Deserialize, Serialize};

use crate::{Expr, Function, Statement, TokenStream, Value};

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Hash, Serialize, Deserialize)]
pub enum OpCode {
    LoadConst(usize),
    LoadLocal(usize),
    LoadGlobal(usize),
    SetGlobal(usize),
    SetLocal(usize),
    Call(usize),
    Jump(usize),
    JumpIfFalse(usize),
    NewTable(usize),
    SetTable(usize),
    GetTable(usize),
    Add,
    Subtract,
    Multiply,
    Divide,
    Eq,
    Pop,
    Dup,
    Swap,
    Return,
}

pub struct Local {
    name: String,
    depth: usize,
}

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub struct Chunk {
    pub code: Vec<OpCode>,
    pub constants: Vec<Value>,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
        }
    }
    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }
    pub fn write(&mut self, opcode: OpCode) {
        self.code.push(opcode);
    }
    pub fn write_jump(&mut self, opcode: OpCode) -> usize {
        self.code.push(opcode);
        self.code.len() - 1
    }
    pub fn patch_jump(&mut self, offset: usize) {
        let current_ip = self.code.len();
        match self.code[offset] {
            OpCode::JumpIfFalse(ref mut target) => *target = current_ip,
            OpCode::Jump(ref mut target) => *target = current_ip,
            _ => panic!("Tentou fazer jump em uma função que não e Jump"),
        }
    }
}

pub struct Compiler {
    pub chunk: Chunk,
    locals: Vec<Local>,
    scope_depth: usize,
    loop_stack: Vec<Vec<usize>>,
}
impl Default for Compiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Compiler {
    pub fn new() -> Self {
        Self {
            chunk: Chunk::new(),
            locals: Vec::new(),
            scope_depth: 0,
            loop_stack: Vec::new(),
        }
    }

    pub fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }
    fn end_scope(&mut self) {
        while let Some(last) = self.locals.last() {
            if last.depth == self.scope_depth {
                self.chunk.write(OpCode::Pop);
                self.locals.pop();
            } else {
                break;
            }
        }
        self.scope_depth -= 1
    }
    fn add_local(&mut self, name: String) -> usize {
        let idx = self.locals.len();
        self.locals.push(Local {
            name,
            depth: self.scope_depth,
        });
        idx
    }

    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    pub fn compile_stmt(&mut self, statements: &[Statement]) {
        for stmt in statements.iter() {
            match stmt {
                Statement::Return(expr) => {
                    if let Some(expr) = expr {
                        self.compile_expr(expr);
                    } else {
                        let idx = self.chunk.add_constant(Value::Nil);
                        self.chunk.write(OpCode::LoadConst(idx));
                    }
                    self.chunk.write(OpCode::Return);
                }
                Statement::Expression(expr) => {
                    self.compile_expr(expr);
                    self.chunk.write(OpCode::Pop);
                }
                Statement::LocalVar(name, value) => {
                    self.compile_expr(value);
                    let idx = self.add_local(name.clone());
                    self.chunk.write(OpCode::SetLocal(idx));
                }
                Statement::Assign(left, right) => match left {
                    Expr::Identifier(name) => {
                        self.compile_expr(right);
                        if let Some(local_idx) = self.resolve_local(name) {
                            self.chunk.write(OpCode::SetLocal(local_idx));
                        } else {
                            let idx = self.chunk.add_constant(Value::String(name.clone()));
                            self.chunk.write(OpCode::SetGlobal(idx));
                        }
                    }
                    Expr::Index { target, key } => {
                        self.compile_expr(target);
                        self.compile_expr(key);
                        self.compile_expr(right);
                        self.chunk.write(OpCode::SetTable(0));
                    }
                    _ => panic!("Assign invalido"),
                },
                Statement::FunctionDef { name, params, body } => {
                    let mut compiler = Compiler::new();

                    let mut arity = 0;
                    for param in params {
                        compiler.add_local(param.to_string());
                        arity += 1;
                    }
                    compiler.compile_stmt(body);

                    let nil_idx = compiler.chunk.add_constant(Value::Nil);
                    compiler.chunk.write(OpCode::LoadConst(nil_idx));
                    compiler.chunk.write(OpCode::Return);

                    let chunk = compiler.chunk;
                    let idx = self.chunk.add_constant(Value::Function(Rc::new(Function {
                        name: name.to_string(),
                        arity,
                        chunk,
                    })));
                    self.chunk.write(OpCode::LoadConst(idx));
                    let name_idx = self.chunk.add_constant(Value::String(name.clone()));
                    self.chunk.write(OpCode::SetGlobal(name_idx));
                }
                Statement::If {
                    condition,
                    then_block,
                    else_block,
                } => {
                    self.compile_expr(condition);
                    let jump_if_false_idx = self.chunk.write_jump(OpCode::JumpIfFalse(0));
                    self.begin_scope();
                    self.compile_stmt(then_block);
                    self.end_scope();
                    if !else_block.is_empty() {
                        let jump_over_else_idx = self.chunk.write_jump(OpCode::Jump(0));
                        self.chunk.patch_jump(jump_if_false_idx);
                        self.begin_scope();
                        self.compile_stmt(else_block);
                        self.end_scope();

                        self.chunk.patch_jump(jump_over_else_idx);
                    } else {
                        self.chunk.patch_jump(jump_if_false_idx);
                    }
                }
                Statement::While(condition, do_block) => {
                    let loop_start = self.chunk.code.len();
                    self.loop_stack.push(Vec::new());

                    self.compile_expr(condition);
                    let exit_jump = self.chunk.write_jump(OpCode::JumpIfFalse(0));

                    self.compile_stmt(do_block);

                    self.chunk.write(OpCode::Jump(loop_start));
                    self.chunk.patch_jump(exit_jump);

                    let breaks = self.loop_stack.pop().unwrap();
                    for break_jump in breaks {
                        self.chunk.patch_jump(break_jump);
                    }
                }
                Statement::Break => {
                    if let Some(breaks) = self.loop_stack.last_mut() {
                        let jump_idx = self.chunk.write_jump(OpCode::Jump(0));
                        breaks.push(jump_idx);
                    } else {
                        panic!("Erro: 'break' fora de um loop!")
                    }
                }

                s => unimplemented!("Statement ainda não implementado: {:?}", s),
            }
        }
    }

    pub fn compile_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                let constant_idx = self.chunk.add_constant(Value::Number(*n));
                self.chunk.write(OpCode::LoadConst(constant_idx))
            }
            Expr::String(s) => {
                let constant_idx = self.chunk.add_constant(Value::String(s.clone()));
                self.chunk.write(OpCode::LoadConst(constant_idx))
            }
            Expr::Bool(b) => {
                let constant_idx = self.chunk.add_constant(Value::Bool(*b));
                self.chunk.write(OpCode::LoadConst(constant_idx))
            }
            Expr::BinaryOp {
                left,
                operator,
                right,
            } => {
                self.compile_expr(left);
                self.compile_expr(right);

                match operator {
                    TokenStream::Plus => self.chunk.write(OpCode::Add),
                    TokenStream::Minus => self.chunk.write(OpCode::Subtract),
                    TokenStream::Asterisk => self.chunk.write(OpCode::Multiply),
                    TokenStream::Slash => self.chunk.write(OpCode::Divide),

                    TokenStream::EqualEqual => self.chunk.write(OpCode::Eq),
                    t => unimplemented!("Operador não implementado no compilador: {:?}", t),
                }
            }
            Expr::UnaryOp {
                operator: _,
                argument,
            } => {
                self.compile_expr(argument);
            }
            Expr::Identifier(id) => {
                if let Some(local_idx) = self.resolve_local(id) {
                    self.chunk.write(OpCode::LoadLocal(local_idx));
                } else {
                    let constant_idx = self.chunk.add_constant(Value::String(id.clone()));
                    self.chunk.write(OpCode::LoadGlobal(constant_idx));
                }
            }
            Expr::Call { target, args } => {
                self.compile_expr(target);
                let arg_count = args.len();
                for arg in args {
                    self.compile_expr(arg);
                }
                self.chunk.write(OpCode::Call(arg_count));
            }
            Expr::Index { target, key } => {
                self.compile_expr(target);
                self.compile_expr(key);
                self.chunk.write(OpCode::GetTable(0));
            }
            Expr::TableConstructor(fields) => {
                self.chunk.write(OpCode::NewTable(fields.len()));

                for field in fields {
                    match field {
                        crate::TableField::Dynamic { key, value } => {
                            self.compile_expr(key);
                            self.compile_expr(value);
                            self.chunk.write(OpCode::SetTable(0));
                        }
                        crate::TableField::Named { key, value } => {
                            let key_idx = self.chunk.add_constant(Value::String(key.clone()));
                            self.chunk.write(OpCode::LoadConst(key_idx));
                            self.compile_expr(value);
                            self.chunk.write(OpCode::SetTable(0));
                        }
                        crate::TableField::List(expr) => {
                            self.compile_expr(expr);
                            self.chunk.write(OpCode::SetTable(0));
                        }
                    }
                }
            }
            Expr::MethodCall {
                target,
                method,
                args,
            } => {
                self.compile_expr(target);
                self.chunk.write(OpCode::Dup);

                let key_idx = self.chunk.add_constant(Value::String(method.clone()));
                self.chunk.write(OpCode::LoadConst(key_idx));
                self.chunk.write(OpCode::GetTable(0));

                self.chunk.write(OpCode::Swap);

                let mut arg_count = 1;

                for arg in args {
                    self.compile_expr(arg);
                    arg_count += 1;
                }
                self.chunk.write(OpCode::Call(arg_count));
            }
            Expr::Function { params, body } => {
                let mut compiler = Compiler::new();
                let mut arity = 0;
                for param in params {
                    compiler.add_local(param.clone());
                    arity += 1;
                }

                compiler.compile_stmt(body);
                let nil_idx = compiler.chunk.add_constant(Value::Nil);

                compiler.chunk.write(OpCode::LoadConst(nil_idx));
                compiler.chunk.write(OpCode::Return);

                let chunk = compiler.chunk;
                let idx = self.chunk.add_constant(Value::Function(Rc::new(Function {
                    name: "<anonymous>".to_string(),
                    arity,
                    chunk,
                })));

                self.chunk.write(OpCode::LoadConst(idx));
            }
            e => unimplemented!("Expressão não implementada: {:?}", e),
        }
    }
}
