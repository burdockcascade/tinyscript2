use std::collections::HashMap;

use log::{debug, error, info, trace};

use crate::vm::program::{Instruction, Program};
use crate::vm::value::Value;
use crate::vm::frame::Frame;

pub mod value;
pub(crate) mod program;
mod frame;

// Virtual Machine
pub struct VM {
    instructions: Vec<Instruction>,
    functions: HashMap<String, i32>,
    frames: Vec<Frame>,
    ip: i32,
    fp: i32,
}

impl VM {

    pub fn new(program: Program) -> Self {
        VM {
            instructions: program.instructions,
            functions: program.functions,
            frames: vec![],
            ip: 0,
            fp: 0
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
        let mut current_frame = self.frames.get_mut(self.fp as usize).expect("no frame found");

        // run instructions
        loop {

            let instruction = self.instructions.get(self.ip as usize).expect(&*format!("instruction #{} not found", self.ip));

            debug!("");
            debug!( "== loop [fp:{} ({:?}), ip:{} ({:?})]", self.fp, current_frame.name, self.ip, instruction);
            debug!("> variables: {:?}", current_frame.variables);
            debug!("> stack: {:?}", current_frame.stack);

            match instruction {

                Instruction::Assert => {
                    let output = current_frame.pop_value_from_stack();
                    trace!("asserting '{}' is true", output);
                    match output {
                        Value::Bool(val) => assert!(val),
                        _ => error!("unable to assert {}", output)
                    }

                    self.ip += 1;
                }

                Instruction::Print => {
                    let output = current_frame.pop_value_from_stack();
                    println!("{:?}", output.to_string());
                    self.ip += 1;
                }

                // FUNCTIONS

                Instruction::Call(arg_len) => {

                    // cut args from stack and then reverse order
                    let mut args = current_frame.pop_values_from_stack(*arg_len as usize);
                    args.reverse();

                    // pop functionref from stack
                    let name = current_frame.pop_value_from_stack().to_string();
                    let funcpos = *self.functions.get(name.to_string().as_str()).expect("function not found");

                    // frame name with fp
                    let fname = format!("{}[{}]", name, self.fp);

                    // push new frame onto frames
                    self.frames.push(Frame::new(fname, self.ip + 1, args));

                    // set fp to current stack length
                    self.fp += 1;

                    // set current frame
                    current_frame = self.frames.get_mut(self.fp as usize).expect("frame found");

                    trace!("ip jumping from {} to {}", self.ip, funcpos);
                    self.ip = funcpos;

                }

                Instruction::ReturnValue => {

                    trace!("returning from {} to position {}", current_frame.name, current_frame.return_position);

                    let return_value = current_frame.pop_value_from_stack();
                    trace!("popped return {}", return_value);

                    if current_frame.return_position < 0 {
                        trace!("returning {} from {}", return_value, current_frame.name);
                        return Ok(return_value);
                    }

                    // set instruction back to previous location
                    trace!("ip jumping from {} to {}", self.ip, current_frame.return_position);
                    self.ip = current_frame.return_position;

                    // decrement fp
                    trace!("decrementing fp from {} to {}", self.fp, self.fp - 1);
                    self.fp -= 1;

                    // remove last frame
                    self.frames.pop();

                    // set new current frame
                    current_frame = self.frames.get_mut(self.fp as usize).expect("no frame found");

                    // push return value onto stack
                    current_frame.push_value_to_stack(return_value);

                }

                // Objects

                Instruction::LoadObjectMember(member) => {
                    let object = current_frame.pop_value_from_stack();
                    match object {
                        Value::Object(obj) => {
                            let value = obj.get(member).expect(&*format!("member '{}' not found in object", member));
                            trace!("pushing value '{}' onto stack", value);
                            current_frame.push_value_to_stack(value.clone());
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
                    let b = current_frame.pop_value_from_stack();
                    self.ip += if b == Value::Bool(true) { *delta } else { 1 };
                }

                Instruction::JumpIfFalse(delta) => {
                    let b = current_frame.pop_value_from_stack();
                    self.ip += if b == Value::Bool(false) { *delta } else { 1 };
                }

                // Push value onto stack
                Instruction::Push(variant) => {
                    trace!("pushing {:?} into stack", variant);
                    current_frame.push_value_to_stack(variant.clone());
                    self.ip += 1
                }

                // get value from stack and store in variable
                Instruction::StoreLocalVariable(index) => {
                    current_frame.move_from_stack_to_variable_slot(*index as usize);
                    self.ip += 1;
                }

                // get value from variable and push onto stack
                Instruction::LoadLocalVariable(index) => {
                    let var = current_frame.get_variable_or_panic(*index as usize);
                    trace!("copying {} from variable {} onto stack", var, index);
                    current_frame.push_value_to_stack(var.clone());
                    self.ip += 1;
                }

                // ARRAYS

                Instruction::ArrayAdd => {
                    let (array, value) = current_frame.pop_2_values();

                    if let Value::Array(mut v) = array {
                        v.push(value);
                        current_frame.push_value_to_stack(Value::Array(v));
                    }

                    self.ip += 1;

                }

                Instruction::LoadIndexedValue => {

                    let (v, index) = current_frame.pop_2_values();
                    trace!("looking up {:?} in {:?}", index, v);

                    match (v, index) {
                        (Value::Array(items), Value::Integer(idx)) => {
                            let item = items[idx as usize].clone();
                            current_frame.push_value_to_stack(item);
                        },
                        (Value::Dictionary(keys), Value::String(key)) => {
                            let item = keys.get(&*key).expect(&*format!("key {} does not exist in dictionary", key.as_str())).clone();
                            current_frame.push_value_to_stack(item);
                        },
                        _ => {
                            error!("variable has no index");
                            break;
                        }
                    }

                    self.ip += 1;
                }

                Instruction::ArrayLength => {

                    let v = current_frame.pop_value_from_stack();

                    match v {
                        Value::Array(val) => current_frame.push_value_to_stack(Value::Integer(val.len() as i32)),
                        _ => unreachable!("can not get length on non-array {}", v)
                    }

                    self.ip += 1;

                }

                Instruction::DictionaryAdd => {
                    let (key, value) = current_frame.pop_2_values();
                    let dict = current_frame.pop_value_from_stack();

                    if let Value::Dictionary(mut v) = dict {
                        v.insert(key.to_string(), value);
                        current_frame.push_value_to_stack(Value::Dictionary(v));
                    }

                    self.ip += 1;
                }

                // ARITHMETIC

                Instruction::Add => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("adding {:?} and {:?}", lhs, rhs);
                    current_frame.push_value_to_stack(lhs + rhs);
                    self.ip += 1;
                }

                Instruction::Sub => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!(" subtracting {:?} from {:?}", rhs, lhs);
                    current_frame.push_value_to_stack(lhs - rhs);
                    self.ip += 1;
                }

                Instruction::Multiply => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("multiplying {:?} and {:?}", rhs, lhs);
                    current_frame.push_value_to_stack(lhs * rhs);
                    self.ip += 1;
                }

                Instruction::Divide => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("dividing {:?} by {:?}", lhs, rhs);
                    current_frame.push_value_to_stack(lhs / rhs);
                    self.ip += 1;
                }

                Instruction::Pow => {
                    // todo
                    self.ip += 1;
                }


                // OPERANDS

                Instruction::Equal => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("equal {:?} and {:?}",  lhs, rhs);
                    current_frame.push_value_to_stack(Value::Bool(lhs == rhs));
                    self.ip += 1;
                }

                Instruction::NotEqual => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("not equal {:?} and {:?}", lhs, rhs);
                    current_frame.push_value_to_stack(Value::Bool(lhs != rhs));
                    self.ip += 1;
                }

                Instruction::LessThan => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("comparing {:?} less than {:?}", rhs, lhs);
                    current_frame.push_value_to_stack(Value::Bool(lhs < rhs));
                    self.ip += 1;
                }

                Instruction::LessThanOrEqual => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("comparing {:?} less than {:?}", rhs, lhs);
                    current_frame.push_value_to_stack(Value::Bool(lhs <= rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThan => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("comparing {:?} greater than {:?}",  lhs, rhs);
                    current_frame.push_value_to_stack(Value::Bool(lhs > rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThanOrEqual => {
                    let (lhs, rhs) = current_frame.pop_2_values();
                    trace!("comparing {:?} greater than {:?}",  lhs, rhs);
                    current_frame.push_value_to_stack(Value::Bool(lhs >= rhs));
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

            trace!(">> stack at end {:?}", current_frame.stack);

        }

        Ok(current_frame.pop_value_from_stack())

    }
    
}
