use tinyscript::{compile, compile_and_run};
use tinyscript::vm::value::Value;

#[test]
fn test_hello_world() {
    assert_eq!(compile_and_run(include_str!("00_hello_world.tny"), "HelloWorld.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_variables() {
    assert_eq!(compile_and_run(include_str!("01_variables.tny"), "Variables.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_ifs() {
    assert_eq!(compile_and_run(include_str!("02_ifs.tny"), "Ifs.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_loops() {
    assert_eq!(compile_and_run(include_str!("03_loops.tny"), "LoopTest.main", Value::Array(vec![])).unwrap(), Value::Null);
}
