use std::collections::HashMap;

use crate::vm::value::Value;

pub struct Program {
    pub instructions: Vec<Instruction>,
    pub functions: HashMap<String, i32>,
}

#[derive(Clone, Debug)]
pub enum Instruction {

    Assert,
    Print,

    // Stack
    Push(Value),
    StoreLocalVariable(i32),
    LoadLocalVariable(i32),
    ExtendStackSize(i32),
    StackPop(i32),

    // Arrays
    LoadArrayIndex,
    ArrayLength,
    ArrayPack,
    DictionaryPack,

    // Instructions
    Call(i32),
    Jump(i32),
    JumpIfTrue(i32),
    JumpIfFalse(i32),
    ReturnValue,

    Equal,
    NotEqual,
    Add,
    Sub,
    Multiply,
    Divide,
    Pow,

    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,

    Halt(String)
}