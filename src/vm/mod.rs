use std::cell::RefCell;
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
    functions: HashMap<String, usize>,
    frames: Vec<Frame>,
    globals: Vec<Value>,
    ip: usize,
}

impl VM {

    pub fn new(program: Program) -> Self {
        VM {
            instructions: program.instructions,
            functions: program.symbols,
            globals: program.globals,
            frames: vec![],
            ip: 0
        }
    }

    pub fn exec(mut self, entry: &str, parameters: Option<Vec<Value>>) -> Result<Value, String> {

        info!("Executing program");
        debug!("program started with {} instructions", self.instructions.len());

        if self.functions.contains_key(entry) {
            self.ip = *self.functions.get(entry).expect("no entry found");
        }

        trace!("{:?}", self.instructions);

        // do not run if no instructions
        if self.instructions.len() == 0 {
            error!("no instructions to run");
            return Ok(Value::Null);
        }

        // push new frame
        self.frames.push(Frame::new(String::from("main"), None, parameters));

        // set current frame
        let mut frame = self.frames.last_mut().expect("frame should be on the stack");

        // run instructions
        loop {

            let instruction = self.instructions.get(self.ip as usize).expect(&*format!("instruction #{} should exist", self.ip));

            debug!("");
            debug!("== loop [frame {}; ip:{} ({:?})]", frame.get_name(), self.ip, instruction);
            frame.print_debug_info();

            match instruction {

                Instruction::Assert => {

                    let output = frame.pop_value_from_stack();
                    trace!("asserting '{}' is true", output);

                    match output {
                        Value::Bool(val) => assert!(val),
                        _ => panic!("unable to assert {}", output)
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
                    let function_position = *self.functions.get(name.as_str()).expect("function should exist");

                    // frame name with fp
                    let function_name = format!("{}[{}]", name, self.frames.len());

                    let a = if args.is_empty() {
                        None
                    } else {
                        Some(args)
                    };

                    // push new frame onto frames
                    let next_ip = self.ip + 1;
                    self.frames.push(Frame::new(function_name, Some(next_ip), a));

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

                    if frame.get_return_position() == None {
                        trace!("returning {} from {}", return_value, frame.get_name());
                        return Ok(return_value);
                    }

                    // set instruction back to previous location
                    trace!("ip jumping from {} to {:?}", self.ip, frame.get_return_position());
                    self.ip = frame.get_return_position().expect("return position should be set");

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
                            frame.push_value_to_stack(Value::Object(Rc::new(RefCell::new(class.clone()))));
                        },
                        _ => unreachable!("{} is not a class", class)
                    }
                    self.ip += 1;
                }


                //==================================================================================
                // CONTROL FLOW

                Instruction::JumpForward(delta) => {
                    trace!("jumping forward by {}", delta);
                    self.ip += *delta as usize;
                }

                Instruction::JumpBackward(delta) => {
                    trace!("jumping backward by {}", delta);
                    self.ip -= *delta as usize;
                }

                Instruction::JumpIfFalse(delta) => {

                    let b = frame.pop_value_from_stack();
                    trace!("jumping if {} is false", b);

                    match b {
                        Value::Bool(false) =>{
                            if *delta > 0 {
                                self.ip += *delta as usize;
                            } else {
                                self.ip -= *delta as usize;
                            }
                        },
                        _ => self.ip += 1
                    }
                }


                //==================================================================================
                // STACK

                // Push value onto stack
                Instruction::StackPush(variant) => {
                    frame.push_value_to_stack(variant.clone());
                    self.ip += 1
                }


                //==================================================================================
                // VARIABLES

                // get value from stack and store in variable
                Instruction::MoveToLocalVariable(index) => {
                    frame.move_from_stack_to_variable_slot(*index);
                    self.ip += 1;
                }

                Instruction::CopyToLocalVariable(index) => {
                    frame.copy_from_stack_to_variable_slot(*index);
                    self.ip += 1;
                }

                // get value from variable and push onto stack
                Instruction::LoadLocalVariable(index) => {
                    frame.copy_from_variable_slot_to_stack(*index);
                    self.ip += 1;
                }

                // load from global
                Instruction::LoadGlobal(index) => {
                    let value = self.globals.get(*index).expect(&*format!("global '{}'should exist", index));
                    frame.push_value_to_stack(value.clone());
                    self.ip += 1;
                }

                //==================================================================================
                // ARRAYS

                // get array length
                Instruction::ArrayLength => {

                    let array = frame.pop_value_from_stack();
                    trace!("got array {:?}", array);

                    if let Value::Array(val) = array {
                        frame.push_value_to_stack(Value::Integer(val.borrow().len() as i32));
                    } else {
                        panic!("can not get length on non-array {}", array)
                    }

                    self.ip += 1;

                }

                // add value to array
                Instruction::ArrayAdd => {

                    let value = frame.pop_value_from_stack();
                    trace!("got value {:?}", value);

                    let array = frame.pop_value_from_stack();
                    trace!("got array {:?}", array);

                    if let Value::Array(mut v) = array {
                        v.borrow_mut().push(value);
                        frame.push_value_to_stack(Value::Array(v));
                    }

                    self.ip += 1;
                }

                //==================================================================================
                // DICTIONARY

                Instruction::DictionaryAdd => {
                    let value = frame.pop_value_from_stack();
                    trace!("got value {:?}", value);

                    let key = frame.pop_value_from_stack();
                    trace!("got key {:?}", key);

                    let dict = frame.pop_value_from_stack();
                    trace!("got dict {:?}", dict);

                    if let Value::Dictionary(mut v) = dict {
                        v.borrow_mut().insert(key.to_string(), value);
                        frame.push_value_to_stack(Value::Dictionary(v));
                    }

                    self.ip += 1;
                }

                //==================================================================================
                // KEY VALUE

                Instruction::GetCollectionItemByKey => {

                    let key = frame.pop_value_from_stack();
                    trace!("got key {:?}", key);

                    let collection = frame.pop_value_from_stack();
                    trace!("got key holder {:?}", collection);

                    match collection {

                        Value::Array(items) => {

                            trace!("got array {:?}", items);

                            if let Value::Integer(index) = key {
                                let borrowed_items = items.borrow();
                                let array_value = borrowed_items.get(index as usize).expect(&*format!("array index {} should exist", index));
                                frame.push_value_to_stack(array_value.clone());
                            } else {
                                panic!("can not get index on non-integer {}", key)
                            }
                        },

                        Value::Dictionary(items) => {

                            trace!("got dictionary {:?}", items);

                            if let Value::String(index) = key {
                                let items_borrowed = items.borrow();
                                let v2 = items_borrowed.get(index.as_str()).expect(&*format!("key '{}' should exist in dictionary", index));
                                frame.push_value_to_stack(v2.clone());
                            } else {
                                panic!("can not get index on non-string {}", key)
                            }
                        }

                        _ => panic!("can not get index on non-collection {}", key)

                    }

                    self.ip += 1;
                }

                Instruction::SetCollectionItemByKey => {

                    let key = frame.pop_value_from_stack();
                    trace!("got key {:?}", key);

                    let value = frame.pop_value_from_stack();
                    trace!("got value {:?}", value);

                    let collection = frame.pop_value_from_stack();
                    trace!("got collection {:?}", collection);

                    match collection {
                        Value::Array(items) => {
                            if let Value::Integer(index) = key {
                                items.borrow_mut()[index as usize] = value;
                                frame.push_value_to_stack(Value::Array(items));
                            } else {
                                panic!("can not get index on non-integer {}", key)
                            }
                        },
                        Value::Dictionary(items) => {
                            if let Value::String(index) = key {
                                trace!("setting key value {:?} {:?}", index, value);
                                items.borrow_mut().insert(index, value);
                                frame.push_value_to_stack(Value::Dictionary(items));
                            } else {
                                panic!("can not get index on non-string {}", key)
                            }
                        }
                        _ => panic!("can not get index on non-collection")
                    }

                    self.ip += 1;
                }

                //==================================================================================
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
                    // todo: implement
                    self.ip += 1;
                }

                //==================================================================================
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

                //==================================================================================
                // CONTROL

                Instruction::Halt(msg) => {
                    info!("{}", msg);
                    break;
                }

                _ => unreachable!("unknown instruction {:?}", instruction)

            }

            if self.ip == self.instructions.len() {
                debug!("end of script");
                break;
            }

            frame.trace_stack_and_variables();

        }

        Ok(frame.pop_value_from_stack())

    }

}