use std::collections::HashMap;
use std::fs;
use log::{debug, error, info, trace};

use crate::compiler::function::Function;
use crate::compiler::token::Token;
use crate::vm::program::Program;
use crate::vm::value::Value;

mod frontend;
mod function;
mod token;

// Compiler
pub struct Compiler {
    functions: Vec<Function>,
    global_lookup: HashMap<String, usize>,
}

impl Compiler {

    pub fn new() -> Self {
        Compiler {
            functions: vec![],
            global_lookup: Default::default(),
        }
    }

    pub fn compile(mut self, program: String) -> Result<Program, String> {

        // create a new program
        let mut p = Program::new();

        // Tokenize Code
        let script: Vec<Token> = frontend::parser::script(program.as_str()).map_err(|e| e.to_string())?;

        // loop through the imports of the script
        debug!("Importing");
        for token in script.iter() {
            match token {
                Token::Import(file) => {
                    debug!("Importing {}", file);
                    let imported_script = fs::read_to_string(file).expect("Unable to read file");
                    // let script: Vec<Token> = frontend::parser::script(&imported_script).map_err(|e| e.to_string()).expect("err");
                },
                _ => {}
            }
        }

        // now loop through the classes of the script
        debug!("Compiling high level items");
        for token in script.iter() {
            match token {
                Token::Class(class_name, items) => {

                    let mut has_constructor = false;

                    // create a new object for the class
                    let mut object = HashMap::new();

                    // loop
                    for item in items {
                        match item {

                            // add the function to the class
                            Token::Function(func_name, _params, _statements) => {

                                if func_name == "constructor" {
                                    // if the class has a constructor, set the flag
                                    has_constructor = true;
                                }

                                // create a new name for the function
                                let new_name = format!("{}.{}", class_name.to_string(), func_name);

                                // insert into object the name of the function with the new name in a FunctionRef
                                object.insert(func_name.to_string(), Value::FunctionRef(new_name.clone()));

                            },

                            // add the variable to the class
                            Token::Variable(name, value) => {
                                object.insert(name.to_string(), Value::Null);
                            },

                            _ => {}
                        }
                    }

                    // add the constructor if it doesn't exist
                    if !has_constructor {
                        // create a new name for the function
                        let new_name = format!("{}.{}", class_name.to_string(), "constructor");

                        // insert into object the name of the function with the new name in a FunctionRef
                        object.insert("constructor".to_string(), Value::FunctionRef(new_name.clone()));
                    }

                    // log class name and object
                    trace!("storing class {:?} with object '{:?}'", class_name.to_string(), object);

                    // insert the class into the globals
                    let global_index = p.insert_global(Value::Class(object));
                    self.global_lookup.insert(class_name.to_string(), global_index);

                },
                _ => {}
            }
        }

        // now loop through the functions of the script
        debug!("Compiling functions");
        for token in script.iter() {
            match token {
                Token::Class(class_name, items) => {

                    for item in items {
                        match item {

                            // add the function to the class
                            Token::Function(func_name, params, statements) => {

                                let new_name = format!("{}.{}", class_name, func_name);

                                // add the 'this' parameter to the function
                                let mut new_params = params.to_vec();
                                new_params.insert(0,Token::Identifier(String::from("this")));

                                // create a new function with the new name
                                let func = Function::new(class_name.as_str(), &new_name, new_params.as_slice(), statements.as_slice(), self.global_lookup.clone());
                                self.functions.push(func);
                            },
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Compile function instructions into one program
        debug!("Compiling program");
        for func in self.functions {

            // insert the function into the program
            trace!("compiling function {} ", func.get_name());

            // add the function to the program
            p.symbols.insert(func.get_name().clone(), p.instructions.len());

            // add the instructions of the function to the program
            p.instructions.extend(func.get_instructions().clone());

        }

        // log the program
        debug!("Program compiled with {} instructions", p.instructions.len());
        trace!("Program is {:?}", p.instructions);

        // return the program
        Ok(p)
    }

}

