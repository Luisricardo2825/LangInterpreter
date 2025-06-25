use lang::interpreter::Interpreter;

fn main() {
    let mut interpreter = Interpreter::new_empty();
    interpreter.interpret_bench();
}
