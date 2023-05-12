use std::{fs::File, io::Read};

use logos::Logos;
use regex::Regex;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\n\f]+")] // Ignore this regex pattern between tokens
enum Token {
    // Tokens can be literal strings, of any length.
    #[token("const")]
    Fast,

    #[token(".")]
    Period,

    // Or regular expressions.
    #[regex(r"(let|mut):([\w]*)(>|\+|\*|/)(.*);")]
    Text,
}

fn main() {
    let regex_val = r"(let|mut):([\w]*)(>|\+|\*|\/)(.*);";
    let mut text = String::new();
    let mut file_ = File::open("lang.x").expect("Cannot get file");
    let seperator = Regex::new(regex_val).expect("Invalid regex");
    file_
        .read_to_string(&mut text)
        .expect("Error reading content");

    let mut lex = Token::lexer(text.as_str());
    // assert_eq!(lex.next(), Some(Ok(Token::Text)));
    // assert_eq!(lex.span(), 0..6);
    // assert_eq!(lex.slice(), "Create");
    while lex.next().is_some() {
        let slice = lex.slice();
        let cap = seperator.captures(slice).unwrap();
        // println!("{:?}", slice);
        // let sp: Vec<&str> = slice.split(":").collect();
        // let declaration_keyword = sp[0]; // let/mut
        // let rest: Vec<String> = Regex::new(r"(>|\+|-|\*|\\/)")
        //     .unwrap()
        //     .split(&sp[1].to_owned())
        //     .map(|x| x.to_string())
        //     .collect();

        // let var_name = &rest[0];
        // let var_value = &rest[1];
        let declaration_keyword = cap.get(1).map_or("", |m| m.as_str());
        let var_name = cap.get(2).map_or("", |m| m.as_str());
        let operator = cap.get(3).map_or("", |m| m.as_str());
        let var_value = cap.get(4).map_or("", |m| m.as_str());
        println!(
            "{} {} {} {}",
            declaration_keyword, var_name, operator, var_value
        );
    }
}
