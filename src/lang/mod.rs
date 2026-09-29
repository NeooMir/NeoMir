pub mod ast;
pub mod parser;
pub mod token;
pub mod vm;

use parser::Parser;
use token::Lexer;
use vm::VirtualMachine;

pub fn compile_source(source: &str) -> Result<VirtualMachine, String> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let program = parser.parse_program()?;
    Ok(VirtualMachine::compile(&program))
}
