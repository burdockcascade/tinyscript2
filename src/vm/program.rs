use std::collections::HashMap;
use crate::vm::instruction::Instruction;
use crate::vm::value::Value;

// Program
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub functions: HashMap<String, i32>,
    pub globals: Vec<Value>,
}

impl Program {

    pub fn new() -> Self {
        Program {
            instructions: vec![],
            functions: HashMap::new(),
            globals: vec![],
        }
    }

    // insert into globals and return index
    pub fn insert_global(&mut self, value: Value) -> i32 {
        self.globals.push(value);
        self.globals.len() as i32 - 1
    }

}