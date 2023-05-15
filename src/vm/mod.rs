use std::collections::HashMap;
use std::rc::Rc;

use log::{debug, error, info, trace};

use crate::vm::program::Program;
use crate::vm::value::Value;
use crate::vm::frame::Frame;
use crate::vm::instruction::Instruction;

pub mod value;
pub(crate) mod program;
pub(crate) mod instruction;
mod frame;

// Virtual Machine
pub struct VM {
    instructions: Vec<Instruction>,
    functions: HashMap<String, i32>,
    frames: Vec<Frame>,
    globals: Vec<Value>,
    ip: i32,
}

impl VM {

    pub fn new(program: Program) -> Self {
        VM {
            instructions: program.instructions,
            functions: program.functions,
            globals: program.globals,
            frames: vec![],
            ip: 0
        }
    }

    pub fn exec(mut self, entry: String, parameters: Value) -> Result<Value, String> {

        info!("Executing program");
        debug!("program started with {} instructions", self.instructions.len());

        if self.functions.contains_key(entry.as_str()) {
            self.ip = *self.functions.get(entry.as_str()).expect("no entry found");
        }

        trace!("{:?}", self.instructions);

        // do not run if no instructions
        if self.instructions.len() == 0 {
            debug!("no instructions to run");
            return Ok(Value::Null);
        }

        // push new frame
        self.frames.push(Frame::new(String::from("main"), -1, vec![parameters]));

        // set current frame
        let mut frame = self.frames.last_mut().expect("frame should be on the stack");

        // run instructions
        loop {

            let instruction = self.instructions.get(self.ip as usize).expect(&*format!("instruction #{} not found", self.ip));

            debug!("");
            debug!( "== loop [frame {}; ip:{} ({:?})]", frame.get_name(), self.ip, instruction);
            frame.print_debug_info();

            match instruction {

                Instruction::Assert => {
                    let output = frame.pop_value_from_stack();
                    trace!("asserting '{}' is true", output);
                    match output {
                        Value::Bool(val) => assert!(val),
                        _ => error!("unable to assert {}", output)
                    }

                    self.ip += 1;
                }

                Instruction::Print => {
                    let output = frame.pop_value_from_stack();
                    println!("{:?}", output.to_string());
                    self.ip += 1;
                }

                // FUNCTIONS

                Instruction::Call(arg_len) => {

                    // cut args from stack and then reverse order
                    let mut args = frame.pop_values_from_stack(*arg_len as usize);
                    args.reverse();

                    // pop functionref from stack
                    let name = frame.pop_value_from_stack().to_string();
                    let function_position = *self.functions.get(name.as_str()).expect("function not found");

                    // frame name with fp
                    let function_name = format!("{}[{}]", name, self.frames.len());

                    // push new frame onto frames
                    let next_ip = self.ip + 1;
                    self.frames.push(Frame::new(function_name, next_ip, args));

                    // set current frame
                    frame = self.frames.last_mut().expect("frame should be on the stack");

                    trace!("ip jumping from {} to {}", self.ip, function_position);
                    self.ip = function_position;

                }

                Instruction::Return(has_return_value) => {

                    let return_value = if *has_return_value {
                        frame.pop_value_from_stack()
                    } else {
                        Value::Null
                    };

                    if frame.get_return_position() == -1 {
                        trace!("returning {} from {}", return_value, frame.get_name());
                        return Ok(return_value);
                    }

                    // set instruction back to previous location
                    trace!("ip jumping from {} to {}", self.ip, frame.get_return_position());
                    self.ip = frame.get_return_position();

                    // remove last frame
                    self.frames.pop();

                    // set new current frame
                    frame = self.frames.last_mut().expect("frame should be on the stack");

                    // push return value onto stack
                    if *has_return_value {
                        frame.push_value_to_stack(return_value);
                    }

                }

                // create object from class
                Instruction::CreateObject => {
                    let class = frame.pop_value_from_stack();
                    match class {
                        Value::Class(class) => {
                            frame.push_value_to_stack(Value::Object(Rc::new(class.clone())));
                        },
                        _ => unreachable!("{} is not a class", class)
                    }
                    self.ip += 1;
                }

                // load member from object
                Instruction::LoadObjectMember(member) => {
                    let object = frame.pop_value_from_stack();
                    match object {
                        Value::Object(obj) => {
                            let value = obj.get(member).expect(&*format!("member '{}' not found in object", member));
                            trace!("pushing value '{}' onto stack", value);
                            frame.push_value_to_stack(value.clone());
                        },
                        _ => unreachable!("{} is not an object", object)
                    }
                    self.ip += 1;
                }

                // CONDITIONS

                Instruction::Jump(delta) => {
                    trace!("moving instruction pointer by {}", delta);
                    self.ip += delta;
                }

                Instruction::JumpIfTrue(delta) => {
                    let b = frame.pop_value_from_stack();
                    trace!("jumping if {} is true", b);
                    self.ip += if b == Value::Bool(true) { *delta } else { 1 };
                }

                Instruction::JumpIfFalse(delta) => {
                    let b = frame.pop_value_from_stack();
                    trace!("jumping if {} is false", b);
                    self.ip += if b == Value::Bool(false) { *delta } else { 1 };
                }

                // Push value onto stack
                Instruction::Push(variant) => {
                    frame.push_value_to_stack(variant.clone());
                    self.ip += 1
                }

                // get value from stack and store in variable
                Instruction::StoreLocalVariable(index) => {
                    frame.move_from_stack_to_variable_slot(*index as usize);
                    self.ip += 1;
                }

                // get value from variable and push onto stack
                Instruction::LoadLocalVariable(index) => {
                    frame.copy_from_variable_slot_to_stack(*index as usize);
                    self.ip += 1;
                }

                // load from global
                Instruction::LoadGlobal(index) => {
                    let value = self.globals.get(*index as usize).expect(&*format!("global '{}' not found", index));
                    frame.push_value_to_stack(value.clone());
                    self.ip += 1;
                }


                // add value to array
                Instruction::ArrayAdd => {
                    let (array, value) = frame.pop_2_values_from_stack();

                    if let Value::Array(mut v) = array {
                        v.push(value);
                        frame.push_value_to_stack(Value::Array(v));
                    }

                    self.ip += 1;

                }

                Instruction::LoadIndexedValue => {

                    let (v, index) = frame.pop_2_values_from_stack();
                    trace!("looking up {:?} in {:?}", index, v);

                    match (v, index) {
                        (Value::Array(items), Value::Integer(idx)) => {
                            let item = items[idx as usize].clone();
                            frame.push_value_to_stack(item);
                        },
                        (Value::Dictionary(keys), Value::String(key)) => {
                            let item = keys.get(&*key).expect(&*format!("key {} does not exist in dictionary", key.as_str())).clone();
                            frame.push_value_to_stack(item);
                        },
                        _ => {
                            error!("variable has no index");
                            break;
                        }
                    }

                    self.ip += 1;
                }

                Instruction::ArrayLength => {

                    let v = frame.pop_value_from_stack();

                    match v {
                        Value::Array(val) => frame.push_value_to_stack(Value::Integer(val.len() as i32)),
                        _ => unreachable!("can not get length on non-array {}", v)
                    }

                    self.ip += 1;

                }

                Instruction::DictionaryAdd => {
                    let (key, value) = frame.pop_2_values_from_stack();
                    let dict = frame.pop_value_from_stack();

                    if let Value::Dictionary(mut v) = dict {
                        v.insert(key.to_string(), value);
                        frame.push_value_to_stack(Value::Dictionary(v));
                    }

                    self.ip += 1;
                }

                // ARITHMETIC

                Instruction::Add => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs + rhs);
                    self.ip += 1;
                }

                Instruction::Sub => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs - rhs);
                    self.ip += 1;
                }

                Instruction::Multiply => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs * rhs);
                    self.ip += 1;
                }

                Instruction::Divide => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(lhs / rhs);
                    self.ip += 1;
                }

                Instruction::Pow => {
                    // todo
                    self.ip += 1;
                }


                // OPERANDS

                Instruction::Equal => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs == rhs));
                    self.ip += 1;
                }

                Instruction::NotEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs != rhs));
                    self.ip += 1;
                }

                Instruction::LessThan => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs < rhs));
                    self.ip += 1;
                }

                Instruction::LessThanOrEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs <= rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThan => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs > rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThanOrEqual => {
                    let (lhs, rhs) = frame.pop_2_values_from_stack();
                    frame.push_value_to_stack(Value::Bool(lhs >= rhs));
                    self.ip += 1;
                }

                // CONTROL

                Instruction::Halt(msg) => {
                    info!("{}", msg);
                    break;
                }

                _ => unreachable!("unknown instruction {:?}", instruction)

            }

            if self.ip == self.instructions.len() as i32 {
                debug!("end of script");
                break;
            }

            frame.print_stack_and_variables();

        }

        Ok(frame.pop_value_from_stack())

    }

}
