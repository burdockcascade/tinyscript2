use tinyscript::compile_and_run;
use tinyscript::vm::value::Value;

#[test]
fn helloworld() {
    assert_eq!(compile_and_run(include_str!("00_helloworld.tny"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn variables() {
    assert_eq!(compile_and_run(include_str!("01_variables.tny"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn loops() {
    assert_eq!(compile_and_run(include_str!("03_loops.tny"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn functions() {
    assert_eq!(compile_and_run(include_str!("04_functions.tny"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn classes() {
    assert_eq!(compile_and_run(include_str!("05_classes.tny"), Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn imports() {
    assert_eq!(compile_and_run(include_str!("10_imports.tny"), Value::Array(vec![])).unwrap(), Value::Null);
}