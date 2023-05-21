use log::{debug, trace};
use crate::vm::value::Value;

#[derive(Clone, PartialEq, Debug)]
pub struct Frame {
    name: String,
    return_position: Option<usize>,
    variables: Vec<Value>,
    data: Vec<Value>,
}

impl ToString for Frame {
    fn to_string(&self) -> String {
        format!("Frame: {}", self.name)
    }
}

impl Frame {

    // new frame with parameter as name
    pub fn new(name: String, return_position: Option<usize>, args: Vec<Value>) -> Frame {

        trace!("new frame {} with return position {:?}", name, return_position);

        Frame {
            name,
            return_position,
            variables: args,
            data: vec![],
        }
    }

    // get functio name
    pub fn get_name(&self) -> &String {
        &self.name
    }

    // get return position
    pub fn get_return_position(&self) -> Option<usize> {
        self.return_position
    }

    // print debug info if debug is enabled
    pub fn print_debug_info(&self) {
        debug!("frame: {}", self.name);
        debug!("return position: {:?}", self.return_position);
        self.print_stack_and_variables();
    }

    // print debug info
    pub fn print_stack_and_variables(&self) {
        debug!("variables: {:?}", self.variables);
        debug!("stack: {:?}", self.data);
    }

    // push a value to the stack
    pub fn push_value_to_stack(&mut self, value: Value) {
        trace!("push value {:?} to stack", value);
        self.data.push(value);
    }

    // push a value to a variable slot
    pub fn push_value_to_variable_slot(&mut self, slot: usize, value: Value) {

        trace!("push value {:?} to variable slot {}", value, slot);

        let variables = &mut self.variables;

        if variables.len() <= slot {
            variables.resize(slot + 1, Value::Null);
        }

        variables[slot] = value;
    }

    // move a value from the stack to a variable slot
    pub fn move_from_stack_to_variable_slot(&mut self, slot: usize) {
        let value = self.pop_value_from_stack();
        self.push_value_to_variable_slot(slot, value);
    }

    // copy value from the stack to a variable slot
    pub fn copy_from_stack_to_variable_slot(&mut self, slot: usize) {
        let value = self.get_top_value_on_stack();
        self.push_value_to_variable_slot(slot, value);
    }

    // copy from variable slot to stack
    pub fn copy_from_variable_slot_to_stack(&mut self, slot: usize) {
        let value = self.get_variable_or_panic(slot).clone();
        self.push_value_to_stack(value);
    }

    // return a clone of the top value on the stack
    pub fn get_top_value_on_stack(&self) -> Value {
        let value = self.data.last().expect("stack should have a value");
        return value.clone();
    }

    // pop a value from the stack
    pub fn pop_value_from_stack(&mut self) -> Value {
        let value = self.data.pop().expect("stack should have a value");
        trace!("pop value {:?} from stack", value);
        return value;
    }

    // pop 2 values from the stack
    pub fn pop_2_values_from_stack(&mut self) -> (Value, Value) {
        let rhs = self.pop_value_from_stack();
        let lhs = self.pop_value_from_stack();
        return (lhs, rhs);
    }

    // pop values from the stack
    pub fn pop_values_from_stack(&mut self, count: usize) -> Vec<Value> {
        trace!("pop {} values from stack", count);
        let mut values = vec![];
        for _ in 0..count {
            values.push(self.pop_value_from_stack());
        }
        return values;
    }

    // get the value from the variable slot
    pub fn get_variable_or_panic(&self, slot: usize) -> &Value {
        trace!("get value from variable slot {}", slot);
        let value = self.variables.get(slot).expect("variable slot should exist");
        return value;
    }

}

#[cfg(test)]
mod tests {

    use crate::vm::frame::Frame;
    use crate::vm::value::Value;

    #[test]
    fn test_get_name() {
        let frame = Frame::new("test".to_string(), None, vec![]);
        assert_eq!(frame.get_name(), "test");
    }

    #[test]
    fn test_get_return_position() {
        let frame = Frame::new("test".to_string(), Some(7), vec![]);
        assert_eq!(frame.get_return_position(), Some(7));
    }

    #[test]
    fn test_push_value_to_stack() {
        let mut frame = Frame::new("test".to_string(), None, vec![]);
        frame.push_value_to_stack(Value::Float(1.0));
        assert_eq!(frame.data.len(), 1);
        assert_eq!(frame.data[0], Value::Float(1.0));
    }

    #[test]
    fn test_push_value_to_variable_slot() {
        let mut frame = Frame::new("test".to_string(), None, vec![]);
        frame.push_value_to_variable_slot(0, Value::Float(1.0));
        assert_eq!(frame.variables.len(), 1);
        assert_eq!(frame.variables[0], Value::Float(1.0));
    }

    #[test]
    fn test_move_from_stack_to_variable_slot() {
        let mut frame = Frame::new("test".to_string(), None, vec![]);
        frame.push_value_to_stack(Value::Float(1.0));
        frame.move_from_stack_to_variable_slot(0);
        assert_eq!(frame.variables.len(), 1);
        assert_eq!(frame.variables[0], Value::Float(1.0));
    }

    #[test]
    fn test_copy_from_variable_slot_to_stack() {
        let mut frame = Frame::new("test".to_string(), None, vec![]);
        frame.push_value_to_variable_slot(0, Value::Float(1.0));
        frame.copy_from_variable_slot_to_stack(0);
        assert_eq!(frame.data.len(), 1);
        assert_eq!(frame.data[0], Value::Float(1.0));
    }

    #[test]
    fn test_get_variable_or_panic() {
        let mut frame = Frame::new("test".to_string(), None, vec![]);
        frame.push_value_to_variable_slot(0, Value::Float(1.0));
        assert_eq!(frame.get_variable_or_panic(0), &Value::Float(1.0));
    }

    #[test]
    fn test_pop_value_from_stack() {
        let mut frame = Frame::new("test".to_string(), None, vec![]);
        frame.push_value_to_stack(Value::Float(1.0));
        assert_eq!(frame.pop_value_from_stack(), Value::Float(1.0));
    }

    #[test]
    fn test_get_2_values_from_stack() {
        let mut frame = Frame::new("test".to_string(), None, vec![]);
        frame.push_value_to_stack(Value::Float(1.0));
        frame.push_value_to_stack(Value::Float(2.0));
        let (lhs, rhs) = frame.pop_2_values_from_stack();
        assert_eq!(lhs, Value::Float(1.0));
        assert_eq!(rhs, Value::Float(2.0));
    }

}