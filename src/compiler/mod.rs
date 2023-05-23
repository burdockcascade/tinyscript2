use crate::compiler::compiler::Compiler;
use crate::vm::program::Program;

mod frontend;
mod function;
mod token;
mod compiler;
mod variable;

pub fn compile(program: &str) -> Result<Program, String> {

    // Return compiled bytecode
    return Compiler::new().compile(program.to_string());

}