use lang::interpreter::Interpreter;
use lang::lexer::tokens::Token;
use lang::parsers::code::parser::Parser;
use logos::Logos;
use std::env;
use std::fs;

fn main() {
    let filename = env::args().nth(1).unwrap_or("lang.x".to_string());
    let show_ast = env::args().nth(2).unwrap_or("false".to_owned());
    let show_ast = show_ast == "true";
    let src = fs::read_to_string(&filename).expect("Failed to read file");

    let tokens = Token::lexer(&src).map(|t| t.unwrap()).collect();
    let mut parser = Parser::new(tokens);
    let ast = parser.parse();

    // bench();
    if show_ast {
        println!("{:#?}", ast);
    }
    let mut interpreter = Interpreter::new(ast);
    interpreter.interpret();
}
