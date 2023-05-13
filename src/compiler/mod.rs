use std::collections::HashMap;
use log::{debug, trace};

use crate::compiler::frontend::Token;
use crate::compiler::function::Function;
use crate::vm::program::Program;
use crate::vm::value::Value;

pub mod frontend;
mod function;

// Compiler
pub struct Compiler {
    functions: Vec<Function>,
    classes: HashMap<String, HashMap<String, Value>>,
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
                                new_params.insert(0,Token::Identifier(String::from("this")));
                                trace!("compiling function {} with parameters {:?}", new_name, new_params);

                                // create a new function with the new name
                                self.functions.push(Function::new(&new_name, new_params.as_slice(), statements.as_slice(), self.classes.clone()));
                            },
                            _ => {}
                        }
                    }
                }
                Token::Function(func_name, params, items) => {

                    // compile new function
                    debug!("compiling function {}", func_name);
                    trace!("{} parameters and {} statements", params.len(), items.len());
                    let f = Function::new(func_name, params.as_slice(), items.as_slice(), self.classes.clone());

                    // compile anonymous functions
                    for af in f.anonymous_functions.iter() {

                        match af {
                            Token::Function(anon_name, params, statements) => {
                                debug!("compiling anonymous function {}", anon_name);
                                trace!("{} parameters and {} statements", params.len(), statements.len());
                                self.functions.push(Function::new(&anon_name, params.as_slice(), statements.as_slice(), self.classes.clone()));
                            }
                            _ => unreachable!("anonymous function is not a function")
                        }
                    }

                    // push the function to the functions vector
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
        debug!("Compiling program");
        for func in self.functions {

            // insert the function into the program
            trace!("compiling function {} with {} instructions and {} vars", func.name, func.instructions.len(), func.variables.len());
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

