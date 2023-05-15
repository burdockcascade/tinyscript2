use std::collections::HashMap;
use crate::vm::instruction::Instruction;
use crate::vm::value::Value;

// Program
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub symbols: HashMap<String, i32>,
    pub globals: Vec<Value>,
}

impl Program {

    pub fn new() -> Self {
        Program {
            instructions: vec![],
            symbols: HashMap::new(),
            globals: vec![],
        }
    }

    // insert into globals and return index
    pub fn insert_global(&mut self, value: Value) -> i32 {
        self.globals.push(value);
        self.globals.len() as i32 - 1
    }

    pub fn insert_into_symbols(&mut self, name: String, index: i32) {
        self.symbols.insert(name, index);
    }

}