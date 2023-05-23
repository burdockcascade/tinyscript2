use log::LevelFilter;
use simplelog::{ColorChoice, Config, TerminalMode, TermLogger};
use crate::compiler::compile;

use crate::vm::program::Program;
use crate::vm::value::Value;
use crate::vm::VM;

pub mod vm;
mod compiler;

pub fn run(program: &str, main: &str, params: Value) -> Result<Value, String> {

    let _ = TermLogger::init(LevelFilter::Trace, Config::default(),TerminalMode::Mixed, ColorChoice::Auto);

    // Compile to bytecode
    let bytecode = compile(program).expect("program error");

    // Create new VM
    let vm: VM = VM::new(bytecode);

    // Execute
    vm.exec(main, params).map_err(|e| e.to_string())

}

