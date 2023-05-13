use log::{debug, trace};
use crate::vm::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Frame {
    pub name: String,
    pub return_position: i32,
    variables: Vec<Value>,
    pub stack: Vec<Value>,
}

impl Frame {

    // new frame with parameter as name
    pub fn new(name: String, return_position: i32, args: Vec<Value>) -> Frame {

        debug!("new frame {} with return position {}", name, return_position);

        Frame {
            name,
            return_position,
            variables: args,
            stack: vec![],
        }
    }

    pub fn push_value_to_stack(&mut self, value: Value) {
        self.stack.push(value);
    }

    pub fn push_value_to_variable_slot(&mut self, slot: usize, value: Value) {

        let variables = &mut self.variables;

        if variables.len() <= slot {
            variables.resize(slot + 1, Value::Null);
        }
        variables[slot] = value;
    }

    pub fn move_from_stack_to_variable_slot(&mut self, slot: usize) {
        let value = self.pop_value_from_stack();
        self.push_value_to_variable_slot(slot, value);
    }

    pub fn pop_value_from_stack(&mut self) -> Value {
        trace!("pop value from stack");
        let v = self.stack.pop().expect("value on stack");
        return v;
    }

    pub fn pop_2_values(&mut self) -> (Value, Value) {
        let rhs = self.pop_value_from_stack();
        let lhs = self.pop_value_from_stack();
        return (lhs, rhs);
    }

    pub fn pop_values_from_stack(&mut self, count: usize) -> Vec<Value> {
        trace!("pop {} values from stack", count);
        let mut values = vec![];
        for _ in 0..count {
            values.push(self.pop_value_from_stack());
        }
        return values;
    }

    pub fn get_variable_or_panic(&self, slot: usize) -> &Value {
        trace!("get variable from slot {}", slot);
        return self.variables.get(slot).expect("value in variable slot");
    }

}