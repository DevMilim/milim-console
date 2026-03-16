use std::{collections::HashMap, rc::Rc};

use crate::{Function, OpCode, Value};

pub enum VMResult {
    Ok,
    RuntimeError(&'static str),
}
#[derive(Clone, Debug, PartialEq)]
pub struct CallFrame {
    pub function: Rc<Function>,
    pub ip: usize,
    pub base_slot: usize,
}

fn print(args: &[Value]) -> Value {
    for arg in args {
        print!("{}", arg.to_string())
    }
    println!();
    Value::Nil
}
pub struct VM {
    frames: Vec<CallFrame>,
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new(main_func: Rc<Function>) -> Self {
        let mut globals = HashMap::new();

        globals.insert("print".to_string(), Value::NativeFuncion(print));
        let frame = CallFrame {
            function: main_func,
            ip: 0,
            base_slot: 0,
        };
        Self {
            stack: Vec::new(),
            globals,
            frames: vec![frame],
        }
    }

    pub fn print_bytecode(&self) {
        for (id, code) in self.frames[0].function.chunk.code.iter().enumerate() {
            println!("{}: {:?}", id, code);
        }
    }
    fn execute_call(&mut self, arg_count: usize) -> VMResult {
        let func_idx = self
            .stack
            .len()
            .checked_sub(arg_count + 1)
            .expect("Stack Overflow ao calcular func_idx");
        let func_value = self.stack[func_idx].clone();
        match func_value {
            Value::Function(function) => {
                if function.arity != arg_count {
                    return VMResult::RuntimeError("Numero de argumentos incompativel");
                }
                let frame = CallFrame {
                    function: function.clone(),
                    ip: 0,
                    base_slot: func_idx,
                };
                self.stack.remove(func_idx);
                self.frames.push(frame);
                VMResult::Ok
            }
            Value::NativeFuncion(native_func) => {
                let args = self.stack[func_idx + 1..].to_vec();
                self.stack.truncate(func_idx);

                let result = native_func(&args);
                self.stack.push(result);
                VMResult::Ok
            }
            _ => VMResult::RuntimeError("Tentativa de chamar algo que não e função"),
        }
    }
    pub fn run(&mut self) -> VMResult {
        loop {
            let frame = self.frames.last_mut().expect("Pilha de frames vazia!");

            let instruction = frame.function.chunk.code[frame.ip];
            frame.ip += 1;
            use OpCode::*;
            match instruction {
                LoadConst(idx) => {
                    let constant = frame.function.chunk.constants[idx].clone();
                    self.stack.push(constant);
                }
                Add => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if let (Value::Number(num_a), Value::Number(num_b)) = (a, b) {
                        self.stack.push(Value::Number(num_a + num_b));
                    } else {
                        return VMResult::RuntimeError("Os operadores devem ser numeros");
                    }
                }
                Subtract => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if let (Value::Number(num_a), Value::Number(num_b)) = (a, b) {
                        self.stack.push(Value::Number(num_a - num_b));
                    } else {
                        return VMResult::RuntimeError("Os operadores devem ser numeros");
                    }
                }
                Multiply => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if let (Value::Number(num_a), Value::Number(num_b)) = (a, b) {
                        self.stack.push(Value::Number(num_a * num_b));
                    } else {
                        return VMResult::RuntimeError("Os operadores devem ser numeros");
                    }
                }
                Divide => {
                    let b = self.stack.pop().unwrap();
                    let a = self.stack.pop().unwrap();

                    if let (Value::Number(num_a), Value::Number(num_b)) = (a, b) {
                        self.stack.push(Value::Number(num_a / num_b));
                    } else {
                        return VMResult::RuntimeError("Os operadores devem ser numeros");
                    }
                }
                Eq => {
                    let a = self.stack.pop().unwrap();
                    let b = self.stack.pop().unwrap();

                    self.stack.push(Value::Bool(a == b));
                }
                Return => {
                    let result = self.stack.pop().unwrap();
                    let frame_end = self.frames.pop().unwrap();
                    self.stack.truncate(frame_end.base_slot);

                    if self.frames.is_empty() {
                        return VMResult::Ok;
                    }
                    self.stack.push(result);
                }
                LoadGlobal(idx) => {
                    let name_val = frame.function.chunk.constants[idx].clone();

                    if let Value::String(name) = name_val {
                        if let Some(val) = self.globals.get(&name) {
                            self.stack.push(val.clone());
                        } else {
                            return VMResult::RuntimeError("Variavel global não definida");
                        }
                    } else {
                        return VMResult::RuntimeError(
                            "O identificador da variavel deve ser string",
                        );
                    }
                }
                LoadLocal(idx) => {
                    let value = self.stack[frame.base_slot + idx].clone();
                    self.stack.push(value);
                }
                SetLocal(idx) => {
                    let value = self.stack.pop().unwrap();
                    self.stack[frame.base_slot + idx] = value
                }
                SetGlobal(idx) => {
                    let name_val = &frame.function.chunk.constants[idx];

                    if let Value::String(name) = name_val {
                        let value = self.stack.pop().unwrap();
                        self.globals.insert(name.clone(), value);
                    } else {
                        return VMResult::RuntimeError(
                            "O identificador da variavel deve ser string",
                        );
                    }
                }
                Call(arg_count) => match self.execute_call(arg_count) {
                    VMResult::Ok => {}
                    VMResult::RuntimeError(msg) => return VMResult::RuntimeError(msg),
                },
                Pop => {
                    self.stack
                        .pop()
                        .expect("Stack Overflow: tentou dar por em uma pilha vazia");
                }
                Jump(target_ip) => frame.ip = target_ip,
                JumpIfFalse(target_ip) => {
                    let condition = self.stack.pop().expect("Pilha vazia no JumpIfFalse");

                    if !condition.is_truthy() {
                        frame.ip = target_ip;
                    }
                }
                NewTable(_) => todo!(),
                SetTable(_) => todo!(),
                GetTable(_) => todo!(),
            }
        }
    }
}
