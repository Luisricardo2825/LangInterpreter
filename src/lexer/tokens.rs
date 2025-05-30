use logos::{Lexer, Logos};

#[derive(Debug, Logos, PartialEq, Clone)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(error = LexingError)]
pub enum Token {
    // Palavras-chave
    // #[token("let")]
    // Let,

    // #[token("const")]
    // Const,

    // #[token("fn")]
    // #[token("function")]
    // Fn,

    // #[token("class")]
    // Class,

    // #[token("new")]
    // New,

    // #[token("this")]
    // This,

    // #[token("static")]
    // Static,

    // #[token("extends")]
    // Extends,

    // #[token("return")]
    // Return,

    // #[token("if")]
    // If,

    // #[token("else")]
    // Else,

    // #[token("while")]
    // While,

    // #[token("for")]
    // For,

    // #[token("in")]
    // In,

    // #[token("of")]
    // Of,

    // #[token("break")]
    // Break,

    // #[token("continue")]
    // Continue,

    // #[token("try")]
    // Try,

    // #[token("catch")]
    // Catch,

    // #[token("finally")]
    // Finally,

    // #[token("throw")]
    // Throw,

    // Identificadores
    #[regex(r"[a-zA-Z_$][a-zA-Z0-9_]*", parser_identifier)]
    Identifier(String),

    // Literais
    #[regex(r"-?(?:0|[1-9](?:_?\d)*)(?:\.\d(?:_?\d)*)?(?:[eE][+-]?\d(?:_?\d)*)?", |lex| lex.slice().replace('_', "").parse::<f64>().unwrap())]
    Number(f64),

    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
    #[regex(r#"'([^'\\]|\\.)*'"#, parse_string)]
    #[regex(r#"`([^`\\]|\\.)*`"#, parse_string)]
    String(String),

    #[token("null")]
    Null,

    #[token("+=")]
    AddAssign,
    #[token("-=")]
    SubAssign,
    #[token("*=")]
    MulAssign,
    #[token("/=")]
    DivAssign,
    #[token("%=")]
    ModAssign,
    #[token("**=")]
    PowAssign,

    // Math Operators
    #[token("=")]
    Assign,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Asterisk,

    #[token("/")]
    Slash,

    // Lang operators
    #[token("...")]
    Ellipsis,

    #[token("->")]
    Arrow,

    #[token(".")]
    Dot,

    #[token("=>")]
    FatArrow,

    #[token("++")]
    Increment,

    #[token("--")]
    Decrement,

    #[token("**")]
    #[token("^")]
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
    #[token(";", priority = 1)]
    Semicolon,

    #[regex(r".", parse_error, priority = 0)]
    Unknown(String),
}

fn parse_error(lex: &mut Lexer<Token>) -> String {
    let id = lex.slice().to_string();
    id
}

fn parser_identifier(lex: &mut Lexer<Token>) -> String {
    let id = lex.slice().to_string();
    id
}

fn parse_string(lex: &mut Lexer<Token>) -> Option<String> {
    let slice = lex.slice();
    Some(slice[1..slice.len() - 1].to_string()) // remove aspas
}

use anyhow::Error;
#[derive(Debug)]
pub struct LexingError(pub Error);

impl<E: std::error::Error + Send + Sync + 'static> From<E> for LexingError {
    fn from(e: E) -> Self {
        LexingError(Error::new(e))
    }
}

impl Default for LexingError {
    fn default() -> Self {
        LexingError(anyhow::anyhow!("Erro léxico padrão"))
    }
}

impl PartialEq for LexingError {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_string() == other.0.to_string()
    }
}

impl Clone for LexingError {
    fn clone(&self) -> Self {
        LexingError(anyhow::anyhow!(self.0.to_string()))
    }
}
