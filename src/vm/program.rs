use std::collections::HashMap;
use crate::vm::instruction::Instruction;

// Program
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub functions: HashMap<String, i32>,
}
