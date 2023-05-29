use std::collections::HashMap;
use std::fs;
use log::{debug, trace};
use crate::compiler::frontend;

use crate::compiler::function::Function;
use crate::compiler::token::Token;
use crate::vm::program::Program;
use crate::vm::value::Value;

pub const CLASS_CONSTRUCTOR_FUNCTION_NAME: &str = "constructor";

// Compiler
pub struct Compiler {
    globals: HashMap<String, Value>,
    global_lookup: HashMap<String, usize>
}

impl Compiler {

    pub fn new() -> Self {
        Compiler {
            globals: Default::default(),
            global_lookup: Default::default()
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
                    // let imported_script = fs::read_to_string(file).expect("Unable to read file");
                    // let script: Vec<Token> = frontend::parser::script(&imported_script).map_err(|e| e.to_string()).expect("err");
                },
                _ => {}
            }
        }


        let mut functions = Vec::new();

        debug!("Declaring top level items");
        for token in script.iter() {
            match token {
                Token::Class(class_name, items) => {

                    // create a new object for the class
                    let mut object = HashMap::new();
                    let mut class_fields = vec![];

                    // loop
                    for item in items.iter() {
                        match item {

                            // add the function to the class
                            Token::Function(func_name, params, statements) => {
                                let func = Function::new(class_name, func_name, params.clone(), statements.clone());
                                object.insert(func_name.to_string(), Value::FunctionRef(func.get_full_name().clone()));
                                functions.push(func);
                            },

                            // add the variable to the class
                            Token::Variable(name, ..) => {
                                class_fields.push(item.clone());
                                object.insert(name.to_string(), Value::Null);
                            },

                            _ => {}
                        }
                    }

                    // add the default constructor if it doesn't exist
                    if object.contains_key(CLASS_CONSTRUCTOR_FUNCTION_NAME) {

                    } else {
                        let default_constructor = Function::new(class_name, CLASS_CONSTRUCTOR_FUNCTION_NAME, Default::default(), class_fields);
                        let fname = default_constructor.get_full_name().clone();
                        functions.push(default_constructor);
                        object.insert(CLASS_CONSTRUCTOR_FUNCTION_NAME.to_string(), Value::FunctionRef(fname));
                    }

                    // log class name and object
                    trace!("storing class {:?} with object '{:?}'", class_name.to_string(), object);

                    // insert the class into the globals
                    let v = Value::Class(object);
                    let global_index = p.insert_global(v.clone());
                    self.global_lookup.insert(class_name.to_string(), global_index);
                    self.globals.insert(class_name.to_string(), v.clone());

                },
                _ => {}
            }
        }

        debug!("Compiling functions");
        for func in functions {
            let fname = func.get_full_name().clone();
            debug!("Compiling function {}", fname);
            let ins = func.compile(self.globals.clone(), self.global_lookup.clone());
            p.symbols.insert(fname, p.instructions.len());
            p.instructions.extend(ins);
        }

        // log the program
        debug!("Program compiled with {} instructions", p.instructions.len());
        trace!("Program is {:?}", p.instructions);

        // return the program
        Ok(p)
    }

}