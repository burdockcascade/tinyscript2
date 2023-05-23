use tinyscript::{run};
use tinyscript::vm::value::Value;

#[test]
fn hello_world() {
    assert_eq!(run(include_str!("scripts/hello_world.tny"), "HelloWorld.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn integers() {
    assert_eq!(run(include_str!("scripts/var_integers.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn floats() {
    assert_eq!(run(include_str!("scripts/var_floats.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn booleans() {
    assert_eq!(run(include_str!("scripts/var_booleans.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn dictionary() {
    assert_eq!(run(include_str!("scripts/var_dictionary.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn arrays() {
    assert_eq!(run(include_str!("scripts/var_arrays.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn strings() {
    assert_eq!(run(include_str!("scripts/var_strings.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_ifs() {
    assert_eq!(run(include_str!("scripts/if_statements.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
}

#[test]
fn test_class() {
    assert_eq!(run(include_str!("scripts/class_simple.tny"), "Test.main", Value::Array(vec![])).unwrap(), Value::Null);
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
