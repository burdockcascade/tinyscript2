use std::collections::HashMap;

use crate::vm::value::Value;

// Program
pub struct Program {
    pub instructions: Vec<Instruction>,
    pub functions: HashMap<String, i32>,
}

// Instruction
#[derive(Clone, Debug)]
pub enum Instruction {

    // Built-in Functions
    Assert,
    Print,

    // Stack
    Push(Value),
    StoreLocalVariable(i32),
    LoadLocalVariable(i32),
    ExtendStackSize(i32),
    StackPop(i32),

    // Arrays & Dictionaries
    LoadIndexedValue,
    ArrayLength,
    ArrayAdd,
    DictionaryAdd,

    // Instructions
    Call(i32),
    Jump(i32),
    JumpIfTrue(i32),
    JumpIfFalse(i32),
    ReturnValue,

    // Operators
    Equal,
    NotEqual,
    Add,
    Sub,
    Multiply,
    Divide,
    Pow,

    // Comparison
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    // Halt Program
    Halt(String)

}