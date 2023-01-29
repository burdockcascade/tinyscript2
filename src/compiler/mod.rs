use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use log::{debug, error, info, trace};

use crate::compiler::frontend::Token;
use crate::vm::program::{Instruction, Program};
use crate::vm::value::Value;
use crate::vm::value::Value::Null;

pub mod frontend;

pub struct Compiler {
    functions: Vec<Function>,
    curfunc: usize
}

struct Function {
    name: String,
    instructions: Vec<Instruction>,
    variables: HashMap<String, i32>
}


impl Compiler {
    pub fn new() -> Self {
        Compiler {
            functions: vec![],
            curfunc: 0,
        }
    }

    pub fn compile(mut self, script: Vec<Token>) -> Result<Program, String> {

        info!("Compiling program");

        // Start with root statements
        self.compile_function(&Box::new(Token::Identifier(String::from("__root"))), vec![Token::Identifier(String::from("argv"))].as_slice(), script.as_slice());

        info!("Compiling program");

        let mut p = Program {
            instructions: vec![],
            functions: Default::default(),
        };

        // Compile function instructions into one program
        for mut func in self.functions {

            debug!("compiling func {} with {} instructions and {} vars", func.name, func.instructions.len(), func.variables.len());

            if func.name == "__root" {
                func.instructions.push(Instruction::Halt(String::from("eof")));
            }

            p.functions.insert(func.name.clone(), p.instructions.len() as i32);
            trace!("{:?}", func.instructions);

            p.instructions.extend(func.instructions);

        }

        debug!("program is {:?}", p.instructions);

        Ok(p)
    }

    fn compile_class(&mut self, name: Box<Token>, items: Vec<Token>) {

        trace!("found class {} with {} items", name.to_string(), items.len());

    }

    fn compile_function(&mut self, name: &Box<Token>, params: &[Token], statements: &[Token]) {

        self.curfunc = self.functions.len();
        let f = Function {
            name: name.to_string(),
            instructions: vec![],
            variables: HashMap::default()
        };
        self.functions.push(f);

        trace!("new function '{:?}' ({}) discovered with {} parameters", self.functions[self.curfunc].name, self.curfunc, params.len());

        let pos = self.functions[self.curfunc].instructions.len() as i32;
        let sz = self.functions[self.curfunc].variables.len() as i32;
        self.functions[self.curfunc].instructions.push(Instruction::ExtendStackSize(sz));

        for param in params {
            let vlen = self.functions[self.curfunc].variables.len() as i32;
            self.functions[self.curfunc].variables.entry(param.to_string()).or_insert(vlen);
        }

        self.compile_statements(statements);

        match self.functions[self.curfunc].instructions.last().expect("instructions") {
            Instruction::ReturnValue => {},
            _ => {
                self.functions[self.curfunc].instructions.push(Instruction::Push(Null));
                self.functions[self.curfunc].instructions.push(Instruction::ReturnValue)
            },
        }

        self.functions[self.curfunc].instructions[pos as usize] = Instruction::ExtendStackSize(self.functions[self.curfunc].variables.len() as i32);
        trace!("end of function definition for '{:?}' ({})", name, self.curfunc);


        self.curfunc = 0;
        trace!("switching to  function definition '{:?}' ({})", self.functions[self.curfunc].name, self.curfunc);


    }

    fn compile_statements(&mut self, statements: &[Token]) {

        for statement in statements {
            match statement {
                Token::Import(file) => self.import_file(file),
                Token::Assert(exp) => self.compile_assert(exp),
                Token::Print(exp) => self.compile_print(exp),
                Token::Call(name, args) => self.compile_call(name, args),
                Token::Variable(name, token) => self.compile_variable(name, token),
                Token::Assign(name, token) => self.compile_assignment(name, token),
                Token::IndexAssign(name, indexes, token) => self.compile_index_assignment(name, indexes, token),
                Token::Function(name, params, statements) => self.compile_function(name, params.as_slice(), statements.as_slice()),
                Token::IfElse(expr, then_body, else_body) => self.compile_ifelse(expr, then_body, else_body),
                Token::WhileLoop(expr, statements) => self.compile_whileloop(expr, statements),
                Token::ForEach(item, array, stmts) => self.compile_foreach(item, array, stmts),
                Token::Return(expr) => self.compile_return(expr),
                Token::ForI(name, from, to, step, stmts) => self.compile_forloop(name, from, to, step, stmts),
                // Token::Class(name, vars) => self.compile_class(name, vars),
                _ => todo!()
            }
        }

    }

    fn import_file(&mut self, file: &String) {

        debug!("Importing {}", file);
        let imported_script = fs::read_to_string(file).expect("Unable to read file");

        let script: Vec<Token> = frontend::parser::script(&imported_script).map_err(|e| e.to_string()).expect("err");

        self.compile_statements(script.as_slice());

    }

    fn compile_assert(&mut self, exp: &Box<Token>) {
        self.compile_expression(exp);
        self.functions[self.curfunc].instructions.push(Instruction::Assert);
    }

    fn compile_variable(&mut self, name: &Box<Token>, value: &Box<Token>) {

        if self.functions[self.curfunc].variables.contains_key(name.to_string().as_str()) {
            error!("variable {} has already been declared", name.to_string());
        }

        // Declare variable
        self.get_variable(name.to_string());

        // Compile variable value
        self.compile_assignment(name, value);
    }

    fn compile_index_assignment(&mut self, name: &Box<Token>, indexes: &[Token], exp: &Box<Token>) {
        trace!("assigning value {} to {} index of {}", exp.to_string(), indexes[0].to_string(), name.to_string());

        let idx = self.get_variable(name.to_string());
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(idx));

        for index in indexes {
            self.compile_expression(index);
            self.functions[self.curfunc].instructions.push(Instruction::LoadIndexedValue);
        }

        self.compile_expression(exp);

    }

    fn compile_assignment(&mut self, name: &Box<Token>, exp: &Box<Token>) {

        if !self.functions[self.curfunc].variables.contains_key(name.to_string().as_str()) {
            error!("variable {} has not been declared", name.to_string());
        }

        // push variant onto stack
        self.compile_expression(exp);

        // Assign variable to slot
        let index = self.get_variable(name.to_string());

        trace!("assigning value to variable '{}' ({})", name.to_string(), index);
        let i = Instruction::StoreLocalVariable(index);
        self.functions[self.curfunc].instructions.push(i);
    }

    fn compile_forloop(&mut self, name: &Box<Token>, from: &Box<Token>, to: &Box<Token>, optional_step: &Option<Box<Token>>, block: &[Token]) {

        trace!("compiling for loop");

        // Create Variable
        let ticker = self.get_variable(name.to_string());
        let endval = self.get_tmp_variable();
        let step = self.get_tmp_variable();

        // Compile initial value and store in ticker
        self.compile_expression(&from);
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(ticker));

        // Compile end value
        self.compile_expression(&to);
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(endval));

        // Compile expression or default to 1
        match optional_step {
            None => self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Integer(1))),
            Some(v) => self.compile_expression(&v),
        }
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(step));

        // Start of loop
        let start_ins_ptr = self.functions[self.curfunc].instructions.len();

        // Compare to end value
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(ticker));
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(endval));
        self.functions[self.curfunc].instructions.push(Instruction::GreaterThan);
        let jump_to_end = self.functions[self.curfunc].instructions.len();
        self.functions[self.curfunc].instructions.push(Instruction::Halt(String::from("No where to jump")));

        // Compile statements inside loop block
        self.compile_statements(block);

        // Step ticker
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(ticker));
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(step));
        self.functions[self.curfunc].instructions.push(Instruction::Add);
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(ticker));

        // Jump to start
        let offset_to_start = start_ins_ptr as i32 - self.functions[self.curfunc].instructions.len() as i32 ;
        self.functions[self.curfunc].instructions.push(Instruction::Jump(offset_to_start));

        // Update jump forward instruction
        let offset_to_end = self.functions[self.curfunc].instructions.len() as i32 - start_ins_ptr as i32;
        self.functions[self.curfunc].instructions[jump_to_end] = Instruction::JumpIfTrue(offset_to_end);


    }

    fn compile_foreach(&mut self, item: &Box<Token>, array: &Box<Token>, block: &[Token]) {
        trace!("compiling for each");

        // Find or create variables
        let array = self.get_variable(array.to_string());
        let item = self.get_variable(item.to_string());
        let arraylen = self.get_tmp_variable();
        let array_idx = self.get_tmp_variable();

        // Get array length
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(array));
        self.functions[self.curfunc].instructions.push(Instruction::ArrayLength);
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(arraylen));

        // Store index in tmp variable
        self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Integer(0)));
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(array_idx));

        // Start of loop
        let start_ins_ptr = self.functions[self.curfunc].instructions.len();

        // Update item value
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(array));
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(array_idx));
        self.functions[self.curfunc].instructions.push(Instruction::LoadIndexedValue);
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(item));

        // Compile statements inside loop block
        self.compile_statements(block);

        // Increment index
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(array_idx));
        self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Integer(1)));
        self.functions[self.curfunc].instructions.push(Instruction::Add);
        self.functions[self.curfunc].instructions.push(Instruction::StoreLocalVariable(array_idx));

        // Jump if not equal
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(arraylen));
        self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(array_idx));
        self.functions[self.curfunc].instructions.push(Instruction::Equal);
        let jump_to_pos = start_ins_ptr as i32 - self.functions[self.curfunc].instructions.len() as i32;
        self.functions[self.curfunc].instructions.push(Instruction::JumpIfFalse(jump_to_pos as i32));


        // Clean up stack
        // self.funcstack[self.scope].instructions.push(Instruction::StackPop(2));
    }

    fn compile_whileloop(&mut self, expr: &Box<Token>, block: &[Token]) {
        trace!("compiling while loop");

        // Mark instruction pointer
        let start_ins_ptr = self.functions[self.curfunc].instructions.len();

        // Compile expression
        self.compile_expression(&expr);

        // Jump to end if expression is false
        let jump_not_true = self.functions[self.curfunc].instructions.len();
        self.functions[self.curfunc].instructions.push(Instruction::Halt(String::from("no jump-not-true provided")));

        // Compile statements inside loop block
        self.compile_statements(block);

        // Goto loop start
        let ins_to_skip = start_ins_ptr as i32 - self.functions[self.curfunc].instructions.len() as i32;
        self.functions[self.curfunc].instructions.push(Instruction::Jump(ins_to_skip));

        // Update jump not true value
        let jump_to_pos = self.functions[self.curfunc].instructions.len() - jump_not_true;
        self.functions[self.curfunc].instructions[jump_not_true] = Instruction::JumpIfFalse(jump_to_pos as i32);

    }

    fn compile_ifelse(&mut self, expr: &Box<Token>, then_body: &[Token], else_body: &Option<Vec<Token>>) {
        trace!("compiling ifelse");

        // Compile If Statement
        self.compile_expression(&expr);

        // Jump to Else if not True
        let jump_to_else= self.functions[self.curfunc].instructions.len();
        self.functions[self.curfunc].instructions.push(Instruction::Halt(String::from("no where to jump to")));

        // Compile Statements for True
        self.compile_statements(then_body);
        let jump_to_end= self.functions[self.curfunc].instructions.len();
        self.functions[self.curfunc].instructions.push(Instruction::Halt(String::from("can not jump tot end")));

        // Update Else Jump
        let jump_to_pos = self.functions[self.curfunc].instructions.len() - jump_to_else;
        self.functions[self.curfunc].instructions[jump_to_else] = Instruction::JumpIfFalse(jump_to_pos as i32);

        match else_body {
            None => {}
            Some(els) => {
                let _ = self.compile_statements(els.as_slice());
            }
        }

        // Update Jump to End
        let jump_to_pos = self.functions[self.curfunc].instructions.len() - jump_to_end;
        self.functions[self.curfunc].instructions[jump_to_end] = Instruction::Jump(jump_to_pos as i32);
    }



    fn compile_expression(&mut self, token: &Token) {
        match token {

            // todo
            Token::AnonFunction(params, statements) => {
                let func_name = format!("func{}", self.functions[self.curfunc].instructions.len());
                self.compile_function(&Box::new(Token::Identifier(func_name.clone())), params, statements);
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::FunctionRef(func_name)));
            }

            Token::Null => {
                trace!("pushing {:?} onto stack", token);
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Null));
            }

            Token::Integer(v) => {
                trace!("pushing {:?} onto stack", token);
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Integer(*v)));
            }

            Token::Float(v) => {
                trace!("pushing {:?} onto stack", token);
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Float(*v)));
            }

            Token::Bool(v) => {
                trace!("pushing {:?} onto stack", token);
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Bool(*v)));
            }

            Token::String(v) => {
                trace!("pushing {:?} onto stack", token);
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::String(v.to_string())));
            }

            Token::Identifier(id) => {
                let idx = self.get_variable(id.clone());
                self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(idx));
            }

            Token::Array(elements) => {

                // Create empty array
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Array(vec![])));

                for element in elements {
                    self.compile_expression(element);
                    self.functions[self.curfunc].instructions.push(Instruction::ArrayAdd);
                }

            }

            Token::Dictionary(pairs) => {

                // Create empty array
                self.functions[self.curfunc].instructions.push(Instruction::Push(Value::Dictionary(HashMap::default())));

                for pair in pairs {
                    if let Token::KeyValuePair(k, value) = pair {
                        self.functions[self.curfunc].instructions.push(Instruction::Push(Value::String(k.to_string())));
                        self.compile_expression(value);
                        self.functions[self.curfunc].instructions.push(Instruction::DictionaryAdd);
                    }
                }

            }

            Token::Index(id, indexes) => {
                trace!("i = {:?}, e = {:?}", id, indexes);

                let idx = self.get_variable(id.to_string());

                let i = Instruction::LoadLocalVariable(idx);
                self.functions[self.curfunc].instructions.push(i);

                for index in indexes {
                    self.compile_expression(index);
                    self.functions[self.curfunc].instructions.push(Instruction::LoadIndexedValue);
                }
            }

            Token::Call(name, args) => {
                self.compile_call(name, args);
            }

            Token::Eq(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.functions[self.curfunc].instructions.push(Instruction::Equal);
            }

            Token::Ne(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.functions[self.curfunc].instructions.push(Instruction::NotEqual);
            }

            Token::Add(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.functions[self.curfunc].instructions.push(Instruction::Add);
            }

            Token::Sub(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.functions[self.curfunc].instructions.push(Instruction::Sub);
            }

            Token::Mul(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.functions[self.curfunc].instructions.push(Instruction::Multiply);
            }

            Token::Div(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.functions[self.curfunc].instructions.push(Instruction::Divide);
            }

            Token::Pow(t1, t2) => {
                self.compile_expression(t1);
                self.compile_expression(t2);
                self.functions[self.curfunc].instructions.push(Instruction::Pow);
            }

            Token::Lt(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.functions[self.curfunc].instructions.push(Instruction::LessThan);
            }

            Token::Le(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.functions[self.curfunc].instructions.push(Instruction::LessThanOrEqual);
            }

            Token::Gt(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.functions[self.curfunc].instructions.push(Instruction::GreaterThan);
            }

            Token::Ge(a, b) => {
                self.compile_expression(a);
                self.compile_expression(b);
                self.functions[self.curfunc].instructions.push(Instruction::GreaterThanOrEqual);
            }

            _ => unreachable!()
        }
    }

    fn compile_print(&mut self, exp: &Box<Token>) {
        self.compile_expression(&exp);
        self.functions[self.curfunc].instructions.push(Instruction::Print);
    }

    fn compile_call(&mut self, name: &Box<Token>, args: &Vec<Token>) {
        let arg_len = args.len();

        trace!("call to function '{:?}' with {} args", name.to_string(), arg_len);

        for arg in args {
            self.compile_expression(arg);
        }

        if self.functions[self.curfunc].variables.contains_key(&*name.to_string()) {
            let index = self.get_variable(name.to_string());
            self.functions[self.curfunc].instructions.push(Instruction::LoadLocalVariable(index))
        } else {
            self.functions[self.curfunc].instructions.push(Instruction::Push(Value::FunctionRef(name.to_string())))
        }

        self.functions[self.curfunc].instructions.push(Instruction::Call(arg_len as i32));
    }

    fn compile_return(&mut self, expr: &Box<Token>) {
        self.compile_expression(expr);
        self.functions[self.curfunc].instructions.push(Instruction::ReturnValue);
    }

    fn get_tmp_variable(&mut self) -> i32 {
        let vlen = self.functions[self.curfunc].variables.len() as i32;
        *self.functions[self.curfunc].variables.entry(format!("var{}", vlen)).or_insert(vlen)
    }

    fn get_variable(&mut self, name: String) -> i32 {
        let vlen = self.functions[self.curfunc].variables.len() as i32;
        *self.functions[self.curfunc].variables.entry(name.parse().unwrap()).or_insert(vlen)
    }

}

// #[cfg(test)]
// mod test {
//     use crate::{Compiler, frontend, Token};
//
//     #[test]
//     fn compile_variables() {
//         let compiler: Compiler = Compiler::new();
//         let script: Vec<Token> = frontend::parser::script(include_str!("../tests/01_hello_world.toy")).unwrap();
//
//         assert!(compiler.compile(script).is_err())
//     }
// }
