use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use crate::compiler::Compiler;

use crate::vm::program::Program;
use crate::vm::value::Value;
use crate::vm::VM;

pub mod vm;
mod compiler;

pub fn compile(program: &str) -> Result<Program, String> {

    let _ = TermLogger::init(LevelFilter::Trace, Config::default(),TerminalMode::Mixed, ColorChoice::Auto);

    // Return compiled bytecode
    return Compiler::new().compile(program.to_string());

}

pub fn compile_and_run(program: &str, entry: String, params: Value) -> Result<Value, String> {
    
    // Compile to bytecode
    let bytecode = compile(program).expect("program error");

    // Create new VM
    let vm: VM = VM::new(bytecode);

    // Execute
    vm.exec(entry, params).map_err(|e| e.to_string())

}

