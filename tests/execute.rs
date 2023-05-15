use tinyscript::compile_and_run;
use tinyscript::vm::value::Value;

#[test]
fn helloworld() {
    assert_eq!(compile_and_run(include_str!("00_helloworld.tny"), String::from("main"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn variables() {
    assert_eq!(compile_and_run(include_str!("01_variables.tny"), String::from("main"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn loops() {
    assert_eq!(compile_and_run(include_str!("03_loops.tny"), String::from("main"),Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn functions() {
    assert_eq!(compile_and_run(include_str!("04_functions.tny"), String::from("main"),Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn fibonacci() {
    assert_eq!(compile_and_run(include_str!("06_fib.tny"), String::from("main"),Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn class() {
    assert_eq!(compile_and_run(include_str!("05_class.tny"), String::from("main"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn imports() {
    assert_eq!(compile_and_run(include_str!("10_imports.tny"), String::from("main"),Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn experimental() {
    assert_eq!(compile_and_run(include_str!("99_experimental.tny"), String::from("main"),Value::Array(vec![])).unwrap(), Value::Null);
}