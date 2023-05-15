use tinyscript::compile;

#[test]
fn compile_helloworld() {
    compile(include_str!("00_helloworld.tny")).expect("should compile");
}

#[test]
fn compile_variables() {
    compile(include_str!("01_variables.tny")).expect("should compile");
}

#[test]
fn compile_loops() {
    compile(include_str!("03_loops.tny")).expect("should compile");
}

#[test]
fn compile_functions() {
    compile(include_str!("04_functions.tny")).expect("should compile");
}

#[test]
fn compile_classes() {
    compile(include_str!("05_class.tny")).expect("should compile");;
}

#[test]
fn compile_imports() {
    compile(include_str!("10_imports.tny")).expect("should compile");
}

#[test]
fn compile_x() {
    compile(include_str!("99_experimental.tny")).expect("should compile");
}
