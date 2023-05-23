use crate::vm::value::Value;

pub struct Variable {
    pub name: String,
    pub index: usize,
    pub value: Value,
}

impl Variable {
    pub fn new(name: String, index: usize, value: Value) -> Variable {
        Variable {
            name,
            index,
            value,
        }
    }
}