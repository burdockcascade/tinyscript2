use tinyscript::{compile, compile_and_run};
use tinyscript::vm::value::Value;

#[test]
fn test_compile() {
    compile(include_str!("test.tny")).expect("should compile");
}

#[test]
fn helloworld() {
    assert_eq!(compile_and_run(include_str!("test.tny"), String::from("main.main"), Value::Array(vec![])).unwrap(), Value::Null);
}