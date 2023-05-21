use crate::vm::value::Value;

// Instruction
#[derive(Clone, Debug)]
pub enum Instruction {

    // Built-in Functions
    Assert,
    Print,

    // Stack
    StackPush(Value),
    StackPop(i32),

    // Variables
    MoveToLocalVariable(usize),
    CopyToLocalVariable(usize),
    LoadLocalVariable(usize),

    // Global
    StoreGlobal(usize),
    LoadGlobal(usize),

    // Objects
    CreateObject,
    LoadObjectMember(String),
    SetObjectMember(String),

    // Dictionaries
    LoadIndexedValue,
    DictionaryAdd,

    // Arrays
    ArrayLength,
    ArrayAdd,

    // Instructions
    Call(usize),
    Jump(i32),
    JumpIfTrue(i32),
    JumpIfFalse(i32),
    Return(bool),

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