use tinyscript::{compile_and_run};
use tinyscript::vm::value::Value;

#[test]
fn test_hello_world() {
    assert_eq!(compile_and_run(include_str!("00_hello_world.tny"), "HelloWorld.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_variables() {
    assert_eq!(compile_and_run(include_str!("01_variables.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_ifs() {
    assert_eq!(compile_and_run(include_str!("02_ifs.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_class() {
    assert_eq!(compile_and_run(include_str!("04_class.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_this() {
    assert_eq!(compile_and_run(include_str!("05_this.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_loops() {
    assert_eq!(compile_and_run(include_str!("03_loops.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_fibonacci() {
    assert_eq!(compile_and_run(include_str!("21_fib.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}
