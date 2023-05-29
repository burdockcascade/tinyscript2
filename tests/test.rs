use tinyscript::{run};
use tinyscript::vm::value::Value;

// HELLO WORLD

#[test]
fn hello_world() {
    assert_eq!(run(include_str!("scripts/hello_world.tny"), "HelloWorld.main", None).unwrap(), Value::Null);
}

// VARIABLES

#[test]
fn integers() {
    assert_eq!(run(include_str!("scripts/var_integers.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn floats() {
    assert_eq!(run(include_str!("scripts/var_floats.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn booleans() {
    assert_eq!(run(include_str!("scripts/var_booleans.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn dictionary() {
    assert_eq!(run(include_str!("scripts/var_dictionary.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn arrays() {
    assert_eq!(run(include_str!("scripts/var_arrays.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn strings() {
    assert_eq!(run(include_str!("scripts/var_strings.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn chain() {
    assert_eq!(run(include_str!("scripts/var_chain.tny"), "Test.main", None).unwrap(), Value::Null);
}

// IFS

#[test]
fn if_statement() {
    assert_eq!(run(include_str!("scripts/if_statement.tny"), "Test.test", None).unwrap(), Value::Null);
}

#[test]
fn if_false() {
    assert_eq!(run(include_str!("scripts/if_false.tny"), "Test.test", None).unwrap(), Value::Null);
}

#[test]
fn if_else() {
    assert_eq!(run(include_str!("scripts/if_else.tny"), "Test.test", None).unwrap(), Value::Null);
}

#[test]
fn if_else_false() {
    assert_eq!(run(include_str!("scripts/if_else_false.tny"), "Test.test", None).unwrap(), Value::Null);
}

// CLASSES

#[test]
fn test_class() {
    assert_eq!(run(include_str!("scripts/class_simple.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn test_this() {
    assert_eq!(run(include_str!("scripts/class_this.tny"), "Test.main", None).unwrap(), Value::Null);
}

// LOOPS

#[test]
fn for_i_loop() {
    assert_eq!(run(include_str!("scripts/loop_for_i.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn for_in_loop() {
    assert_eq!(run(include_str!("scripts/loop_for_in.tny"), "Test.main", None).unwrap(), Value::Null);
}

#[test]
fn while_loop() {
    assert_eq!(run(include_str!("scripts/loop_while.tny"), "Test.main", None).unwrap(), Value::Null);
}

// COMPLEX SCRIPTS

#[test]
fn fibonacci() {
    assert_eq!(run(include_str!("scripts/fib.tny"), "Test.main", None).unwrap(), Value::Null);
}
