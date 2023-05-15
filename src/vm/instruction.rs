use crate::vm::value::Value;

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
    LoadGlobal(i32),
    CreateObject,
    StackPop(i32),

    // Arrays & Dictionaries
    LoadIndexedValue,
    LoadObjectMember(String),
    ArrayLength,
    ArrayAdd,
    DictionaryAdd,

    // Instructions
    Call(i32),
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