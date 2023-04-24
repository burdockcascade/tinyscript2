use std::collections::HashMap;

use log::{debug, error, info, trace};

use crate::vm::program::{Instruction, Program};
use crate::vm::value::Value;
use crate::vm::value::Value::{Null, ReturnFrame, ReturnPosition};

pub mod value;
pub mod program;

const LOCAL_VARIABLE_OFFSET: i32 = 2;

pub struct VM {
    instructions: Vec<Instruction>,
    functions: HashMap<String, i32>,
    stack: Vec<Value>,
    ip: i32,
    fp: i32,
}

impl VM {

    pub fn new(program: Program) -> Self {
        VM {
            instructions: program.instructions,
            functions: program.functions,
            stack: vec![],
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

        self.stack.push(Value::ReturnPosition(-1));
        self.stack.push(Value::ReturnFrame(-1));
        self.stack.push(parameters);

        trace!("{:?}", self.instructions);

        // do not run if no instructions
        if self.instructions.len() == 0 {
            debug!("no instructions to run");
            return Ok(Null);
        }

        loop {

            let instruction = self.instructions.get(self.ip as usize).expect(&*format!("instruction #{} not found", self.ip));

            debug!("");
            debug!( "== loop [fp:{}, ip:{} ({:?}), stack:{}]", self.fp, self.ip, instruction, self.stack.len());
            trace!(">> stack at start {:?}", self.stack);

            match instruction {

                Instruction::Assert => {
                    let output = self.stack.pop().expect("nothing on the stack to evaluate for assert");
                    trace!("asserting '{}' is true", output);
                    match output {
                        Value::Bool(val) => assert!(val),
                        _ => unreachable!()
                    }

                    self.ip += 1;
                }

                Instruction::Print => {
                    let output = self.stack.pop().expect("something should be on the stack to print");
                    println!("{:?}", output.to_string());
                    self.ip += 1;
                }

                // FUNCTIONS

                Instruction::Call(arg_len) => {

                    let name = self.stack.pop().expect("function ref should be on the stack");

                    trace!("> calling '{}' with {} args", name, arg_len);

                    let func = self.functions.get(name.to_string().as_str()).expect(&*format!("function '{}' not found", name));

                    // cut args
                    let split_at = self.stack.len() - *arg_len as usize;
                    let args = self.stack.split_off(split_at);
                    trace!("args are {:?}", args);
                    trace!("stack is {:?}", self.stack);

                    // Set the frame header
                    let rp = Value::ReturnPosition(self.ip + 1);
                    let rf = Value::ReturnFrame(self.fp);
                    trace!("pushing frame header {:?} to stack", rp);
                    trace!("pushing frame header {:?} to stack", rf);
                    self.stack.push(rp);
                    self.stack.push(rf);
                    self.fp = (self.stack.len() - 2) as i32;
                    trace!("set fp to {}", self.fp);

                    self.stack.extend(args);

                    trace!("ip jumping from {} to {}", self.ip, func);
                    self.ip = *func;

                }

                Instruction::ReturnValue => {

                    let ret = self.stack.pop().unwrap();
                    trace!("popped return {}", ret);

                    trace!("clearing out stack upto {}", self.fp + 2);
                    self.stack.truncate((self.fp + 2) as usize);

                    let (rp, rf) = self.pop_2_values();

                    match rp {
                        ReturnPosition(pos) => {
                            trace!("setting instruction position to {}", pos);
                            self.ip = pos
                        },
                        _ => unreachable!("{} is not a return position", rp),
                    }

                    match rf {
                        ReturnFrame(pos) => {
                            trace!("setting frame position to {}", pos);
                            self.fp = pos;
                        },
                        _ => unreachable!("{} is not a return frame", rf),
                    }

                    if self.fp == -1 {
                        trace!("HALT!");
                        self.stack.clear();
                        break;
                    }

                    trace!("pushing {:?} onto stack as return value", ret);
                    self.stack.push(ret);
                    trace!("returning to {}", self.ip);

                }

                // CONDITIONS

                Instruction::Jump(delta) => {
                    trace!("moving instruction pointer by {}", delta);
                    self.ip += delta;
                }

                Instruction::JumpIfTrue(delta) => {

                    let b = self.stack.pop().expect("nothing on the stack to pop");

                    match b {
                        Value::Bool(false) => {
                            trace!("condition false");
                            self.ip += 1;
                        }
                        Value::Bool(true) => {
                            trace!("condition true. moving instruction pointer by {}", delta);
                            self.ip += delta;
                        }
                        _ => unreachable!()
                    }

                }

                Instruction::JumpIfFalse(delta) => {

                    let b = self.stack.pop().expect("nothing on the stack to pop");

                    match b {
                        Value::Bool(true) => {
                            trace!("condition true");
                            self.ip += 1;
                        }
                        Value::Bool(false) => {
                            trace!("condition false. moving instruction pointer by {}", delta);
                            self.ip += delta;
                        }
                        _ => unreachable!()
                    }

                }

                // STACK AND VARIABLES

                Instruction::ExtendStackSize(size) => {
                    trace!("extending stack by {:?}", size);
                    self.stack.resize(self.stack.len() + *size as usize, Value::Null);
                    self.ip += 1
                }

                Instruction::StackPop(length) => {
                    self.stack.truncate(self.stack.len() - *length as usize);
                    self.ip += 1
                }

                Instruction::Push(variant) => {
                    trace!("pushing {:?} into stack", variant);
                    self.stack.push(variant.clone());
                    self.ip += 1
                }

                Instruction::StoreLocalVariable(index) => {

                    let x = LOCAL_VARIABLE_OFFSET + index;

                    let v = self.stack.pop().expect("no value on stack");
                    trace!("moving {:?} from stack to variable {} ({})", v, x, (self.fp + x));

                    let pos = (self.fp + x) as usize;

                    if pos >= self.stack.len() {
                        self.stack.push(Null);
                    }

                    self.stack[pos] = v;
                    self.ip += 1;
                }

                Instruction::LoadLocalVariable(index) => {
                    let var = self.stack.get((self.fp + LOCAL_VARIABLE_OFFSET + index) as usize).expect("variable should exist");
                    trace!("copying {} from variable {} onto stack", var, index);
                    self.stack.push(var.clone());
                    self.ip += 1;
                }

                // ARRAYS

                Instruction::ArrayAdd => {
                    let (array, value) = self.pop_2_values();

                    if let Value::Array(mut v) = array {
                        v.push(value);
                        self.stack.push(Value::Array(v));
                    }

                    self.ip += 1;

                }

                Instruction::LoadIndexedValue => {

                    let (v, index) = self.pop_2_values();
                    trace!("looking up {:?} in {:?}", index, v);

                    match (v, index) {
                        (Value::Array(items), Value::Integer(idx)) => {
                            let item = items[idx as usize].clone();
                            self.stack.push(item);
                        },
                        (Value::Dictionary(keys), Value::String(key)) => {
                            let item = keys.get(&*key).expect(&*format!("key {} does not exist in dictionary", key.as_str())).clone();
                            self.stack.push(item);
                        },
                        _ => {
                            error!("variable has no index");
                            break;
                        }
                    }

                    self.ip += 1;
                }

                Instruction::ArrayLength => {

                    let v = self.stack.pop().expect("no value to pop");

                    match v {
                        Value::Array(val) => self.stack.push(Value::Integer(val.len() as i32)),
                        _ => unreachable!("can not get length on non-array {}", v)
                    }

                    self.ip += 1;

                }

                Instruction::DictionaryAdd => {
                    let (key, value) = self.pop_2_values();
                    let dict = self.stack.pop().expect("dictionary should be on the stack");

                    if let Value::Dictionary(mut v) = dict {
                        v.insert(key.to_string(), value);
                        self.stack.push(Value::Dictionary(v));
                    }

                    self.ip += 1;
                }

                // ARITHMETIC

                Instruction::Add => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("adding {:?} and {:?}", lhs, rhs);
                    self.stack.push(lhs + rhs);
                    self.ip += 1;
                }

                Instruction::Sub => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!(" subtracting {:?} from {:?}", rhs, lhs);
                    self.stack.push(lhs - rhs);
                    self.ip += 1;
                }

                Instruction::Multiply => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("multiplying {:?} and {:?}", rhs, lhs);
                    self.stack.push(lhs * rhs);
                    self.ip += 1;
                }

                Instruction::Divide => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("dividing {:?} by {:?}", lhs, rhs);
                    self.stack.push(lhs / rhs);
                    self.ip += 1;
                }

                Instruction::Pow => {
                    // todo
                    self.ip += 1;
                }


                // OPERANDS

                Instruction::Equal => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("equal {:?} and {:?}",  lhs, rhs);
                    self.stack.push(Value::Bool(lhs == rhs));
                    self.ip += 1;
                }

                Instruction::NotEqual => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("not equal {:?} and {:?}", lhs, rhs);
                    self.stack.push(Value::Bool(lhs != rhs));
                    self.ip += 1;
                }

                Instruction::LessThan => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("comparing {:?} less than {:?}", rhs, lhs);
                    self.stack.push(Value::Bool(lhs < rhs));
                    self.ip += 1;
                }

                Instruction::LessThanOrEqual => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("comparing {:?} less than {:?}", rhs, lhs);
                    self.stack.push(Value::Bool(lhs <= rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThan => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("comparing {:?} greater than {:?}",  lhs, rhs);
                    self.stack.push(Value::Bool(lhs > rhs));
                    self.ip += 1;
                }

                Instruction::GreaterThanOrEqual => {
                    let (lhs, rhs) = self.pop_2_values();
                    trace!("comparing {:?} greater than {:?}",  lhs, rhs);
                    self.stack.push(Value::Bool(lhs >= rhs));
                    self.ip += 1;
                }

                // CONTROL

                Instruction::Halt(msg) => {
                    info!("{}", msg);
                    break;
                }

            }

            if self.ip == self.instructions.len() as i32 {
                debug!("end of script");
                break;
            }

            trace!(">> stack at end {:?}", self.stack);

        }

        Ok(self.stack.pop().or(Option::from(Value::Null)).unwrap())

    }

    fn pop_2_values(&mut self) -> (Value, Value) {
        let rhs = self.stack.pop().expect("no 1st value on ops stack");
        let lhs = self.stack.pop().expect("no 2nd value on ops stack");
        return (lhs, rhs);
    }

}

#[cfg(test)]
mod test {
    use crate::vm::program::Instruction::*;
    use crate::vm::program::Program;
    use crate::vm::value::Value;
    use crate::vm::VM;

    #[test]
    fn add_integers() {

        let program = Program {
            instructions: vec![
                Push(Value::Integer(7)),
                Push(Value::Integer(9)),
                Add,
            ],
            functions: Default::default(),
        };

        let vm: VM = VM::new(program);
        let v = vm.exec(Value::Array(vec![])).expect("err");

        assert_eq!(v, Value::Integer(16))

    }

    #[test]
    fn subtract_integers() {

        let program = Program {
            instructions: vec![
                Push(Value::Integer(12)),
                Push(Value::Integer(9)),
                Sub,
            ],
            functions: Default::default(),
        };

        let vm: VM = VM::new(program);
        let v = vm.exec(Value::Array(vec![])).expect("err");

        assert_eq!(v, Value::Integer(3))

    }

}