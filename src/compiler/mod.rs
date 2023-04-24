use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::rc::Rc;
use log::{debug, error, info, trace};

use crate::compiler::frontend::Token;
use crate::vm::program::{Instruction, Program};
use crate::vm::value::Value;
use crate::vm::value::Value::Null;

pub mod frontend;

pub struct Compiler {
    functions: Vec<Function>,
    classes: HashMap<String, HashMap<String, Value>>,
}

struct Function {
    name: String,
    instructions: Vec<Instruction>,
    variables: HashMap<String, i32>
}

impl Compiler {

    pub fn new() -> Self {
        Compiler {
            classes: HashMap::new(),
            functions: vec![]
        }
    }

    pub fn compile(mut self, script: Vec<Token>) -> Result<Program, String> {

        // loop through the imports of the script
        debug!("Importing");
        for token in script.as_slice() {
            match token {
                Token::Import(file) => {
                    debug!("Importing {}", file);
                    // let imported_script = fs::read_to_string(file).expect("Unable to read file");
                    // let script: Vec<Token> = frontend::parser::script(&imported_script).map_err(|e| e.to_string()).expect("err");
                },
                _ => {}
            }
        }

        // now loop through the classes of the script
        debug!("Compiling classes");
        for token in script.as_slice() {
            match token {
                Token::Class(class_name, items) => {

                    // create a new object for the class
                    let mut object = HashMap::new();

                    // loop
                    for item in items {
                        match item {

                            // add the function to the class
                            Token::Function(func_name, _params, _statements) => {

                                // create a new name for the function
                                let mut new_name = class_name.to_string();
                                new_name.push_str("_");
                                new_name.push_str(&func_name.to_string());

                                // insert into object the name of the function with the new name in a FunctionRef
                                object.insert(func_name.to_string(), Value::FunctionRef(new_name));
                            },

                            // add the variable to the class
                            Token::Variable(name, value) => {
                                object.insert(name.to_string(), Value::Null);
                            },

                            _ => {}
                        }
                    }

                    // insert the class into the classes hashmap
                    self.classes.insert(class_name.to_string(), object.clone());

                    // log class name and object
                    trace!("storing class {:?} with object '{:?}'", class_name.to_string(), object);

                },
                _ => {}
            }
        }

        // now loop through the functions of the script
        debug!("Compiling functions");
        for token in script.as_slice() {
            match token {
                Token::Class(class_name, items) => {
                    for item in items {
                        match item {

                            // add the function to the class
                            Token::Function(func_name, params, statements) => {
                                let mut new_name = class_name.to_string();
                                new_name.push_str("_");
                                new_name.push_str(&func_name.to_string());

                                // add the 'this' parameter to the function
                                let mut new_params = params.to_vec();
                                new_params.push(Token::Identifier(String::from("this")));

                                // create a new function with the new name
                                self.functions.push(Function::new(&Box::new(Token::Identifier(new_name.clone())), new_params.as_slice(), statements.as_slice()));
                            },
                            _ => {}
                        }
                    }
                }
                Token::Function(func_name, params, items) => {
                    let f = Function::new(&func_name, params.as_slice(), items.as_slice());
                    self.functions.push(f);
                },
                _ => {}
            }
        }

        // create a new program
        let mut p = Program {
            instructions: vec![],
            functions: Default::default(),
        };

        // Compile function instructions into one program
        debug!("Compiling functions into program");
        for func in self.functions {
            trace!("compiling func {} with {} instructions and {} vars", func.name, func.instructions.len(), func.variables.len());
            p.functions.insert(func.name.clone(), p.instructions.len() as i32);
            p.instructions.extend(func.instructions);
        }

        // log the program
        debug!("Program compiled with {} instructions", p.instructions.len());
        trace!("Program is {:?}", p.instructions);

        // return the program
        Ok(p)
    }

}

impl Function {

    pub fn new(name: &Box<Token>, params: &[Token], statements: &[Token]) -> Self {

        // create a new function
        let mut f = Function {
            name: name.to_string(),
            instructions: vec![],
            variables: HashMap::default()
        };

        trace!("=== new function '{:?}' started with {} parameters and {} statements", f.name, params.len(), statements.len());

        // extend the stack size
        let pos = f.instructions.len() as i32;
        let sz = f.variables.len() as i32;
        f.instructions.push(Instruction::ExtendStackSize(sz));

        // store the parameters as variables
        for param in params {
            trace!("storing parameter as variable '{}'", param.to_string());
            f.get_or_create_variable(param.to_string());
        }

        // compile the statements
        f.compile_statements(statements);

        // add a return value if there isn't one
        match f.instructions.last().expect("instructions") {
            Instruction::ReturnValue => {},
            _ => {
                f.instructions.push(Instruction::Push(Null));
                f.instructions.push(Instruction::ReturnValue)
            },
        }

        // set the stack size
        f.instructions[pos as usize] = Instruction::ExtendStackSize(f.variables.len() as i32);
        trace!("end of function definition for '{:?}'", name);

        return f;

    }

    // compile a list of statements
    fn compile_statements(&mut self, statements: &[Token]) {
        for statement in statements {
            self.compile_statement(statement);
        }
    }

    // compile a statement
    fn compile_statement(&mut self, statement: &Token) {
        match statement {
            Token::Assert(exp) => self.compile_assert(exp),
            Token::Print(exp) => self.compile_print(exp),
            Token::Call(name, args) => self.compile_call(name, args),
            Token::Variable(name, token) => self.compile_variable(name, token),
            Token::Assign(name, token) => self.compile_assignment(name, token),
            Token::IndexAssign(name, indexes, token) => self.compile_index_assignment(name, indexes, token),
            Token::IfElse(expr, then_body, else_body) => self.compile_ifelse(expr, then_body, else_body),
            Token::WhileLoop(expr, statements) => self.compile_whileloop(expr, statements),
            Token::ForEach(item, array, stmts) => self.compile_foreach(item, array, stmts),
            Token::Return(expr) => self.compile_return(expr),
            Token::ForI(start, end, step, stmts) => self.compile_forloop(start, end, step, stmts),
            _ => todo!("statement: {:?}", statement)
        }
    }

    // compile an assert statement
    fn compile_assert(&mut self, exp: &Box<Token>) {
        self.compile_expression(exp);
        trace!("asserting");
        self.instructions.push(Instruction::Assert);
    }

    // compile a variable declaration
    fn compile_variable(&mut self, name: &Box<Token>, value: &Box<Token>) {

        // Check if variable has already been declared
        if self.variables.contains_key(name.to_string().as_str()) {
            error!("variable {} has already been declared", name.to_string());
        }

        // Declare variable
        self.get_or_create_variable(name.to_string());

        // Compile variable value
        self.compile_assignment(name, value);
    }

    fn compile_index_assignment(&mut self, name: &Box<Token>, indexes: &[Token], exp: &Box<Token>) {
        trace!("assigning value {} to {} index of {}", exp.to_string(), indexes[0].to_string(), name.to_string());

        // check if variable has been declared
        let idx = self.get_or_create_variable(name.to_string());
        self.instructions.push(Instruction::LoadLocalVariable(idx));

        // push indexes onto stack
        for index in indexes {
            self.compile_expression(index);
            self.instructions.push(Instruction::LoadIndexedValue);
        }

        self.compile_expression(exp);

    }

    // compile assignment
    fn compile_assignment(&mut self, name: &Box<Token>, exp: &Box<Token>) {

        // check if variable has been declared
        if !self.variables.contains_key(name.to_string().as_str()) {
            error!("variable {} has not been declared", name.to_string());
        }

        // push variant onto stack
        self.compile_expression(exp);

        // Assign variable to slot
        let index = self.get_or_create_variable(name.to_string());

        // store value in variable
        trace!("assigning value to variable '{}' ({})", name.to_string(), index);
        self.instructions.push(Instruction::StoreLocalVariable(index));
    }

    // compile for loop
    fn compile_forloop(&mut self, start: &Box<Token>, end: &Box<Token>, step: &Box<Token>, block: &[Token]) {

        trace!("compiling for loop");

        // compile start
        self.compile_statement(start);

        // Mark instruction pointer
        let start_of_loop = self.instructions.len();

        // Compile expression
        self.compile_expression(&end);

        // Jump to end if expression is false
        let jump_not_true = self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("no jump-not-true provided")));

        // Compile statements inside loop block
        self.compile_statements(block);

        // compile step
        self.compile_statement(step);

        // Goto loop start
        let ins_to_skip = start_of_loop as i32 - self.instructions.len() as i32;
        self.instructions.push(Instruction::Jump(ins_to_skip));

        // Update jump not true value
        let jump_to_pos = self.instructions.len() - jump_not_true;
        self.instructions[jump_not_true] = Instruction::JumpIfFalse(jump_to_pos as i32);

    }

    // compile while loop
    fn compile_whileloop(&mut self, expr: &Box<Token>, block: &[Token]) {
        trace!("compiling while loop");

        // Mark instruction pointer
        let start_ins_ptr = self.instructions.len();

        // Compile expression
        self.compile_expression(&expr);

        // Jump to end if expression is false
        let jump_not_true = self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("no jump-not-true provided")));

        // Compile statements inside loop block
        self.compile_statements(block);

        // Goto loop start
        let ins_to_skip = start_ins_ptr as i32 - self.instructions.len() as i32;
        self.instructions.push(Instruction::Jump(ins_to_skip));

        // Update jump not true value
        let jump_to_pos = self.instructions.len() - jump_not_true;
        self.instructions[jump_not_true] = Instruction::JumpIfFalse(jump_to_pos as i32);

    }

    // compile for each loop
    fn compile_foreach(&mut self, item: &Box<Token>, array: &Box<Token>, block: &[Token]) {
        trace!("compiling for each");

        // Find or create variables
        let array = self.get_or_create_variable(array.to_string());
        let item = self.get_or_create_variable(item.to_string());
        let arraylen = self.create_temp_variable();
        let array_idx = self.create_temp_variable();

        // Get array length
        self.instructions.push(Instruction::LoadLocalVariable(array));
        self.instructions.push(Instruction::ArrayLength);
        self.instructions.push(Instruction::StoreLocalVariable(arraylen));

        // Store index in tmp variable
        self.instructions.push(Instruction::Push(Value::Integer(0)));
        self.instructions.push(Instruction::StoreLocalVariable(array_idx));

        // Start of loop
        let start_ins_ptr = self.instructions.len();

        // Update item value
        self.instructions.push(Instruction::LoadLocalVariable(array));
        self.instructions.push(Instruction::LoadLocalVariable(array_idx));
        self.instructions.push(Instruction::LoadIndexedValue);
        self.instructions.push(Instruction::StoreLocalVariable(item));

        // Compile statements inside loop block
        self.compile_statements(block);

        // Increment index
        self.instructions.push(Instruction::LoadLocalVariable(array_idx));
        self.instructions.push(Instruction::Push(Value::Integer(1)));
        self.instructions.push(Instruction::Add);
        self.instructions.push(Instruction::StoreLocalVariable(array_idx));

        // Jump if not equal
        self.instructions.push(Instruction::LoadLocalVariable(arraylen));
        self.instructions.push(Instruction::LoadLocalVariable(array_idx));
        self.instructions.push(Instruction::Equal);
        let jump_to_pos = start_ins_ptr as i32 - self.instructions.len() as i32;
        self.instructions.push(Instruction::JumpIfFalse(jump_to_pos as i32));


        // Clean up stack
        // self.funcstack[self.scope].instructions.push(Instruction::StackPop(2));
    }


    // compile if statement
    fn compile_ifelse(&mut self, expr: &Box<Token>, then_body: &[Token], else_body: &Option<Vec<Token>>) {
        trace!("compiling ifelse");

        // Compile If Statement
        self.compile_expression(&expr);

        // Jump to Else if not True
        let jump_to_else= self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("no where to jump to")));

        // Compile Statements for True
        self.compile_statements(then_body);
        let jump_to_end= self.instructions.len();
        self.instructions.push(Instruction::Halt(String::from("can not jump tot end")));

        // Update Else Jump
        let jump_to_pos = self.instructions.len() - jump_to_else;
        self.instructions[jump_to_else] = Instruction::JumpIfFalse(jump_to_pos as i32);

        match else_body {
            None => {}
            Some(els) => {
                let _ = self.compile_statements(els.as_slice());
            }
        }

        // Update Jump to End
        let jump_to_pos = self.instructions.len() - jump_to_end;
        self.instructions[jump_to_end] = Instruction::Jump(jump_to_pos as i32);
    }


    // compile expression
    fn compile_expression(&mut self, token: &Token) {
        match token {

            // todo
            Token::AnonFunction(params, statements) => {
                // let func_name = format!("func{}", self.instructions.len());
                // self.compile_function(&Box::new(Token::Identifier(func_name.clone())), params, statements);
                // self.instructions.push(Instruction::Push(Value::FunctionRef(func_name)));
            }

            Token::Null => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::Push(Value::Null));
            }

            Token::Integer(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::Push(Value::Integer(*v)));
            }

            Token::Float(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::Push(Value::Float(*v)));
            }

            Token::Bool(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::Push(Value::Bool(*v)));
            }

            Token::String(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::Push(Value::String(v.to_string())));
            }

            Token::Identifier(id) => {
                let idx = self.get_or_create_variable(id.clone());
                self.instructions.push(Instruction::LoadLocalVariable(idx));
            }

            Token::Array(elements) => {

                // Create empty array
                self.instructions.push(Instruction::Push(Value::Array(vec![])));

                for element in elements {
                    self.compile_expression(element);
                    self.instructions.push(Instruction::ArrayAdd);
                }

            }

            Token::Dictionary(pairs) => {

                // Create empty array
                self.instructions.push(Instruction::Push(Value::Dictionary(HashMap::default())));

                for pair in pairs {
                    if let Token::KeyValuePair(k, value) = pair {
                        self.instructions.push(Instruction::Push(Value::String(k.to_string())));
                        self.compile_expression(value);
                        self.instructions.push(Instruction::DictionaryAdd);
                    }
                }

            }

            Token::Object(class, params) => {
                trace!("class = {:?}, params = {:?}", class, params);
                self.instructions.push(Instruction::Push(Value::Object(Rc::new(HashMap::default()))));
            }

            Token::Index(id, indexes) => {
                trace!("i = {:?}, e = {:?}", id, indexes);

                let idx = self.get_or_create_variable(id.to_string());

                let i = Instruction::LoadLocalVariable(idx);
                self.instructions.push(i);

                for index in indexes {
                    self.compile_expression(index);
                    self.instructions.push(Instruction::LoadIndexedValue);
                }
            }

            Token::Call(name, args) => {
                self.compile_call(name, args);
            }

            Token::Eq(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Equal);
            }

            Token::Ne(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::NotEqual);
            }

            Token::Add(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Add);
            }

            Token::Sub(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Sub);
            }

            Token::Mul(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Multiply);
            }

            Token::Div(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Divide);
            }

            Token::Pow(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.instructions.push(Instruction::Pow);
            }

            Token::Lt(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::LessThan);
            }

            Token::Le(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::LessThanOrEqual);
            }

            Token::Gt(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::GreaterThan);
            }

            Token::Ge(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.instructions.push(Instruction::GreaterThanOrEqual);
            }

            // handle call chain and print debug info
            Token::CallChain(init, chain) => {
                trace!("call chain: {:?} and {:?}", init, chain);
            }

            // handle unreadable token and print what it is
            _ => panic!("unhandled token: {:?}", token),

        }
    }

    // compile a print statement
    fn compile_print(&mut self, exp: &Box<Token>) {
        self.compile_expression(&exp);
        self.instructions.push(Instruction::Print);
    }

    // compile a function call
    fn compile_call(&mut self, name: &Box<Token>, args: &Vec<Token>) {
        let arg_len = args.len();

        trace!("call to function '{:?}' with {} args", name.to_string(), arg_len);

        // compile the arguments
        for arg in args {
            self.compile_expression(arg);
        }

        if self.variables.contains_key(&*name.to_string()) {
            let index = self.get_or_create_variable(name.to_string());
            self.instructions.push(Instruction::LoadLocalVariable(index))
        } else {
            self.instructions.push(Instruction::Push(Value::FunctionRef(name.to_string())))
        }

        self.instructions.push(Instruction::Call(arg_len as i32));
    }

    // compile a return statement
    fn compile_return(&mut self, expr: &Box<Token>) {
        self.compile_expression(expr);
        self.instructions.push(Instruction::ReturnValue);
    }

    // create a new temporary variable
    fn create_temp_variable(&mut self) -> i32 {
        self.get_or_create_variable(format!("var{}", self.variables.len()))
    }

    // get the index of a variable or create it if it doesn't exist
    fn get_or_create_variable(&mut self, name: String) -> i32 {
        let vlen = self.variables.len() as i32;
        *self.variables.entry(name).or_insert(vlen)
    }


}
