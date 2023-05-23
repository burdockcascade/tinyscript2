use tinyscript::{run};
use tinyscript::vm::value::Value;

#[test]
fn test_hello_world() {
    assert_eq!(run(include_str!("scripts/hello_world.tny"), "HelloWorld.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_variables() {
    assert_eq!(run(include_str!("scripts/variables.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_ifs() {
    assert_eq!(run(include_str!("scripts/if_statements.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_class() {
    assert_eq!(run(include_str!("scripts/simple_class.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_this() {
    assert_eq!(run(include_str!("scripts/class_this.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_loops() {
    assert_eq!(run(include_str!("scripts/loops.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_fibonacci() {
    assert_eq!(run(include_str!("scripts/fib.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}
