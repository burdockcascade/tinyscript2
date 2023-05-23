use std::collections::HashMap;
use std::env::var;
use log::{debug, error, info, trace};
use crate::compiler::compiler::CLASS_CONSTRUCTOR_FUNCTION_NAME;
use crate::compiler::token::Token;
use crate::compiler::variable::Variable;
use crate::vm::instruction::Instruction;
use crate::vm::value::Value;

const CLASS_SELF_VARIABLE_NAME: &str = "this";

// Function
pub struct Function {
    name: String,
    class_name: String,
    parameters: Vec<Token>,
    statements: Vec<Token>,
    instructions: Vec<Instruction>,
    anonymous_functions: Vec<Token>,
    variables: HashMap<String, Variable>,
    pub globals: HashMap<String, Value>,
    pub global_lookup: HashMap<String, usize>,
}


impl Function {

    pub fn new(class_name: &str, func_name: &str, parameters: Vec<Token>, statements: Vec<Token>) -> Self {
        trace!("compiling function '{}' in '{}' with parameters {:?}", func_name, class_name, parameters);

        // create a new function
        Function {
            name: func_name.to_string(),
            class_name: class_name.to_string(),
            parameters,
            statements,
            instructions: vec![],
            anonymous_functions: vec![],
            variables: Default::default(),
            globals: Default::default(),
            global_lookup: Default::default(),
        }
    }

    pub fn compile(mut self, globals: HashMap<String, Value>, global_lookup: HashMap<String, usize>) -> Vec<Instruction> {

        // store the globals
        self.globals = globals;
        self.global_lookup = global_lookup;

        // if there are no statements then return
        if self.statements.is_empty() {
            return vec![Instruction::Return(false)];
        }

        // add the 'this' parameter
        self.parameters.insert(0, Token::Identifier(CLASS_SELF_VARIABLE_NAME.to_string()));

        // store the parameters as variables
        self.add_parameters(self.parameters.clone());

        // compile the statements
        self.compile_statements(self.statements.clone().as_slice());

        // if tha last instruction is not a return then add one
        if matches!(self.instructions.last(), Some(Instruction::Return(_))) == false {
            self.instructions.push(Instruction::Return(false));
        }

        self.instructions
    }

    fn add_parameters(&mut self, parameters: Vec<Token>) {
        for param in parameters {
            let pname = param.to_string();
            trace!("storing parameter as variable '{}'", pname);
            self.add_variable(pname, Value::Null);
        }
    }

    // get name
    pub fn get_full_name(&self) -> String {
        return format!("{}.{}", self.class_name, self.name);
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
            Token::Variable(left, right) => self.compile_variable(left, right),
            Token::Assign(left, right) => self.compile_assignment(left, right),
            Token::IfElse(expr, then_body, else_body) => self.compile_ifelse(expr, then_body, else_body),
            Token::WhileLoop(expr, statements) => self.compile_whileloop(expr, statements),
            Token::ForEach(item, array, stmts) => self.compile_foreach(item, array, stmts),
            Token::Return(expr) => self.compile_return(expr),
            Token::ForI(start, end, step, stmts) => self.compile_forloop(start, end, step, stmts),
            Token::Chain(start, chain) => self.compile_chain(start, chain),
            Token::Comment(_) => {},
            _ => todo!("statement: {:?}", statement)
        }
    }

    // compile a chain of statements
    fn compile_chain(&mut self, start: &Token, chain: &[Token]) {

        // load the start of the chain
        trace!("compiling chain start {:?}", start);
        self.compile_expression(start);

        // for each item in chain
        for item in chain {

            trace!("compiling chain item {:?}", item);

            // push load object member instruction onto stack
            match item {
                Token::Identifier(name) => {
                    self.instructions.push(Instruction::StackPush(Value::String(name.to_string())));
                    self.instructions.push(Instruction::GetKeyValue)
                },
                Token::Call(name, args) => {

                    // load the object member
                    trace!("loading object member {:?}", name);
                    self.instructions.push(Instruction::StackPush(Value::String(name.to_string())));
                    self.instructions.push(Instruction::GetKeyValue);

                    // push 'this' onto stack
                    let variable = self.get_variable(start.to_string());
                    self.instructions.push(Instruction::LoadLocalVariable(variable.index));

                    // compile the arguments
                    for arg in args {
                        self.compile_expression(arg);
                    }

                    // call the function
                    trace!("calling function with {} args", args.len());
                    self.instructions.push(Instruction::Call(args.len() + 1));

                },
                _ => unreachable!("chain item is not a variable or index")
            }

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

        // Declare variable
        self.add_variable(name.to_string(), Value::Null);
        let slot = self.get_variable(name.to_string()).index;

        // compile the value
        self.compile_expression(value);

        // store the value
        self.instructions.push(Instruction::MoveToLocalVariable(slot));
    }

    // compile assignment
    fn compile_assignment(&mut self, left: &Box<Token>, right: &Box<Token>) {

        debug!("compiling assignment {:?} = {:?}", left, right);

        match *left.clone() {

            // store value in variable
            Token::Identifier(name) => {
                trace!("storing value in variable {}", name.to_string());

                // get the variable slot
                let slot = self.get_variable(name.to_string()).index;

                // compile the value
                self.compile_expression(right);

                // store the value
                self.instructions.push(Instruction::MoveToLocalVariable(slot));
            },

            // store value in array index
            Token::ArrayIndex(name, index) => {
                trace!("storing value in index {:?} of {}", index, name.to_string());

                // load the variable
                let slot = self.get_variable(name.to_string()).index;
                self.instructions.push(Instruction::LoadLocalVariable(slot));

                // compile the value
                self.compile_expression(right);

                // compile the index
                self.compile_expression(&index.clone());

                // add value to array
                self.instructions.push(Instruction::SetKeyValue);

                // update variable
                self.instructions.push(Instruction::MoveToLocalVariable(slot));
            },

            _ => panic!("name is not an identifier or index")
        }

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
        self.instructions.push(Instruction::JumpBackward(self.instructions.len() - start_of_loop));

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
        self.instructions.push(Instruction::JumpBackward(self.instructions.len() - start_ins_ptr));

        // Update jump not true value
        let jump_to_pos = self.instructions.len() - jump_not_true;
        self.instructions[jump_not_true] = Instruction::JumpIfFalse(jump_to_pos as i32);

    }

    // compile for each loop
    fn compile_foreach(&mut self, item: &Box<Token>, array: &Box<Token>, block: &[Token]) {
        trace!("compiling for each");

        // // Find or create variables
        // let array = self.get_variable_slot(array.to_string());
        // let item = self.add_variable(item.to_string());
        // let arraylen = self.create_temp_variable();
        // let array_idx = self.create_temp_variable();
        //
        // // Get array length
        // self.instructions.push(Instruction::LoadLocalVariable(array));
        // self.instructions.push(Instruction::ArrayLength);
        // self.instructions.push(Instruction::MoveToLocalVariable(arraylen));
        //
        // // Store index in tmp variable
        // self.instructions.push(Instruction::StackPush(Value::Integer(0)));
        // self.instructions.push(Instruction::MoveToLocalVariable(array_idx));
        //
        // // Start of loop
        // let start_ins_ptr = self.instructions.len();
        //
        // // Update item value
        // self.instructions.push(Instruction::LoadLocalVariable(array));
        // self.instructions.push(Instruction::LoadLocalVariable(array_idx));
        // self.instructions.push(Instruction::LoadIndexedValue);
        // self.instructions.push(Instruction::MoveToLocalVariable(item));
        //
        // // Compile statements inside loop block
        // self.compile_statements(block);
        //
        // // Increment index
        // self.instructions.push(Instruction::LoadLocalVariable(array_idx));
        // self.instructions.push(Instruction::StackPush(Value::Integer(1)));
        // self.instructions.push(Instruction::Add);
        // self.instructions.push(Instruction::MoveToLocalVariable(array_idx));
        //
        // // Jump if not equal
        // self.instructions.push(Instruction::LoadLocalVariable(arraylen));
        // self.instructions.push(Instruction::LoadLocalVariable(array_idx));
        // self.instructions.push(Instruction::Equal);
        // let jump_to_pos = start_ins_ptr as i32 - self.instructions.len() as i32;
        // self.instructions.push(Instruction::JumpIfFalse(jump_to_pos as i32));

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
        self.instructions[jump_to_end] = Instruction::JumpForward(self.instructions.len() - jump_to_end);
    }

    fn compile_new_object(&mut self, class_name: String, params: &[Token]) {
        trace!("class = {:?}, params = {:?}", class_name, params);

        // find class
        let global_id = self.global_lookup.get(&class_name).unwrap();

        // load global
        self.instructions.push(Instruction::LoadGlobal(*global_id));

        // create object
        self.instructions.push(Instruction::CreateObject);

        // store object in temp variable
        let tmp_name = format!("tmp{}", self.instructions.len());
        self.add_variable(tmp_name.clone(), Value::Null);
        let obj_var = self.get_variable(tmp_name.clone()).index;
        self.instructions.push(Instruction::CopyToLocalVariable(obj_var));

        // load constructor functionref
        self.instructions.push(Instruction::StackPush(Value::String(CLASS_CONSTRUCTOR_FUNCTION_NAME.parse().unwrap())));
        self.instructions.push(Instruction::GetKeyValue);

        // load object
        self.instructions.push(Instruction::LoadLocalVariable(obj_var));

        // load params
        for param in params {
            self.compile_expression(param);
        }

        // call constructor
        self.instructions.push(Instruction::Call(params.len() + 1));

        // load object for assignment
        self.instructions.push(Instruction::LoadLocalVariable(obj_var));

    }


    // compile expression
    fn compile_expression(&mut self, token: &Token) {
        match token {

            // todo
            Token::AnonFunction(params, statements) => {
                let func_name = format!("func{}", self.instructions.len());

                // add function to anon functions
                self.anonymous_functions.push(Token::Function(func_name.clone(), params.clone(), statements.clone()));

                // Push ref to function
                self.instructions.push(Instruction::StackPush(Value::FunctionRef(func_name)));
            }

            Token::Null => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::StackPush(Value::Null));
            }

            Token::Integer(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::StackPush(Value::Integer(*v)));
            }

            Token::Float(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::StackPush(Value::Float(*v)));
            }

            Token::Bool(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::StackPush(Value::Bool(*v)));
            }

            Token::String(v) => {
                trace!("pushing {:?} onto stack", token);
                self.instructions.push(Instruction::StackPush(Value::String(v.to_string())));
            }

            Token::Identifier(id) => {
                trace!("pushing {:?} onto stack", token);
                let idx = self.get_variable(id.clone()).index;
                self.instructions.push(Instruction::LoadLocalVariable(idx));
            }

            Token::Array(elements) => {

                // Create empty array
                self.instructions.push(Instruction::StackPush(Value::Array(vec![])));

                for element in elements {
                    self.compile_expression(element);
                    self.instructions.push(Instruction::ArrayAdd);
                }

            }

            Token::Dictionary(pairs) => {

                // Create empty array
                self.instructions.push(Instruction::StackPush(Value::Dictionary(HashMap::default())));

                for pair in pairs {
                    if let Token::KeyValuePair(k, value) = pair {
                        self.instructions.push(Instruction::StackPush(Value::String(k.to_string())));
                        self.compile_expression(value);
                        self.instructions.push(Instruction::DictionaryAdd);
                    }
                }

            }

            Token::Object(class_name, params) => self.compile_new_object(class_name.to_string(), params),

            Token::ArrayIndex(id, index) => {
                trace!("i = {:?}, e = {:?}", id, index);

                // load array
                let idx = self.get_variable(id.to_string()).index;
                self.instructions.push(Instruction::LoadLocalVariable(idx));

                // compile index
                self.compile_expression(index);

                // get array value
                self.instructions.push(Instruction::GetKeyValue);

            }

            Token::Call(name, args) => {
                trace!("call = {:?}, args = {:?}", name, args);
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
            Token::Chain(init, chain) => self.compile_chain(init, chain),

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
        let mut arg_len = args.len();

        trace!("call to function '{:?}' with {} args", name.to_string(), arg_len);

        // push functionref onto stack
        if self.variables.contains_key(&*name.to_string()) {
            let index = self.get_variable(name.to_string()).index;
            self.instructions.push(Instruction::LoadLocalVariable(index))
        } else {
            self.instructions.push(Instruction::LoadLocalVariable(0));
            self.instructions.push(Instruction::StackPush(Value::String(name.to_string())));
            self.instructions.push(Instruction::GetKeyValue);
            self.instructions.push(Instruction::LoadLocalVariable(0));
            arg_len += 1;
        }

        // compile the arguments
        for arg in args {
            self.compile_expression(arg);
        }

        self.instructions.push(Instruction::Call(arg_len));
    }

    // compile a return statement
    fn compile_return(&mut self, expr: &Box<Token>) {
        self.compile_expression(expr);
        self.instructions.push(Instruction::Return(true));
    }

    // get index of variable or error if it doesn't exist
    fn get_variable(&self, name: String) -> &Variable {
        if let Some(variable) = self.variables.get(&*name) {
            variable
        } else {
            panic!("variable '{}' does not exist", name);
        }
    }

    // add variable and return its index or error if it already exists
    fn add_variable(&mut self, name: String, value: Value) {

        // check if variable already exists
        if self.variables.contains_key(name.as_str()) {
            panic!("variable '{}' already exists", name);
        }

        // create variable
        let variable = Variable::new(name.clone(), self.variables.len(), value);

        // add variable to list
        self.variables.insert(name.clone(), variable);
    }


}
