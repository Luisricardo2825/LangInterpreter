use logos::{Lexer, Logos};

#[derive(Debug, Logos, PartialEq, Clone)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
    // Palavras-chave
    #[token("let")]
    Let,

    #[token("fn")]
    Fn,

    #[token("return")]
    Return,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("of")]
    Of,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    // Identificadores
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Literais
    #[regex(r"-?(?:0|[1-9](?:_?\d)*)(?:\.\d(?:_?\d)*)?(?:[eE][+-]?\d(?:_?\d)*)?", |lex| lex.slice().replace('_', "").parse::<f64>().unwrap())]
    Number(f64),

    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
    String(String),

    #[token("null")]
    Null,

    // Math Operators
    #[token("=")]
    Assign,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    // Lang operators
    #[token("->")]
    Arrow,

    #[token(".")]
    Dot,

    #[token("=>")]
    FatArrow,

    #[token("...")]
    Ellipsis,

    #[token("++")]
    Increment,

    #[token("--")]
    Decrement,

    #[token("**")]
    Exponentiation,

    #[token("%")]
    Modulo,

    // Delimitadores
    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token("{")]
    BraceOpen,

    #[token("}")]
    BraceClose,

    #[token("[")]
    BracketOpen,

    #[token("]")]
    BracketClose,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    // Boolean algebra
    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("!")]
    Not,

    // Comparadores
    #[token("==")]
    Equal,

    #[token("!=")]
    NotEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    // Comments
    #[regex(r"//.*", logos::skip)] // <- ignora comentários
    Comment,
    #[regex(r"/\*(?:[^*]|\*[^/])*\*/", logos::skip)]
    CommentMultiline,
    // Fim de arquivo
    #[token(";")]
    Semicolon,
}

fn parse_string(lex: &mut Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    Some(slice[1..slice.len() - 1].to_string()) // remove aspas
}
