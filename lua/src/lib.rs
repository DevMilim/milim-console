mod ast;
mod compiler;
mod lexer;
mod parser;
mod scope;
mod token;
mod utils;
mod values;
mod vm;

use std::{fs::File, io::Read, rc::Rc};

pub use ast::*;
pub use compiler::*;
pub use lexer::*;
pub use parser::*;
pub use scope::*;
pub use token::*;
pub use utils::*;
pub use values::*;
pub use vm::*;

pub struct Lua {
    vm: VM,
}
impl Lua {
    pub fn compile_and_run(file: &str) {
        let mut file = File::open(file).unwrap();
        let mut input = String::new();
        file.read_to_string(&mut input).unwrap();
        let lex = Lexer::new(&input);
        let mut parser = Parser::new(lex);

        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        compiler.compile_stmt(&program.stmts);
        compiler.chunk.write(OpCode::Return);

        let main_func = Rc::new(Function {
            name: "main".to_owned(),
            arity: 0,
            chunk: compiler.chunk,
        });

        VM::new(main_func).run();
    }
    pub fn compile(file: &str, output: &str) {
        let mut file = File::open(file).unwrap();
        let mut input = String::new();
        file.read_to_string(&mut input).unwrap();
        let lex = Lexer::new(&input);
        let mut parser = Parser::new(lex);

        let program = parser.parse_program().unwrap();

        let mut compiler = Compiler::new();
        compiler.compile_stmt(&program.stmts);
        compiler.chunk.write(OpCode::Return);

        let main_func = Function {
            name: "main".to_owned(),
            arity: 0,
            chunk: compiler.chunk,
        };

        save_bytecode(&main_func, output).unwrap()
    }

    pub fn run(&mut self) {
        self.vm.run();
    }
    pub fn load(file: &str) -> Self {
        let code = load_bytecode(file).unwrap();
        Self {
            vm: VM::new(Rc::new(code)),
        }
    }
}
