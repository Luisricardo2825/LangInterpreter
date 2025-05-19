use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        value: Option<Expr>,
    },
    FuncDecl {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_ifs: Vec<(Expr, Option<Vec<Stmt>>)>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        init: Box<Stmt>,
        condition: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Stmt>,
    },
    ExprStmt(Expr),
    Return(Option<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    MemberAccess {
        object: Box<Expr>,
        property: Box<Expr>,
    },
    UnaryOp {
        op: UnaryOperator,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Block(Vec<Stmt>),
}

impl Expr {
    pub fn is_literal(&self) -> bool {
        matches!(self, Expr::Literal(_))
    }
    pub fn to_number(&self) -> Option<f64> {
        match self {
            Expr::Literal(Literal::Number(n)) => Some(*n),
            _ => None,
        }
    }
    pub fn to_string(&self) -> Option<String> {
        match self {
            Expr::Literal(Literal::String(s)) => Some(s.clone()),
            Expr::Identifier(s) => Some(s.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Math(MathOperator),
    Compare(CompareOperator),
    Logical(LogicalOperator),
}

#[derive(Debug, Clone)]
pub enum MathOperator {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone)]
pub enum CompareOperator {
    Eq, // ==
    Ne, // !=
    Gt, // >
    Ge, // >=
    Lt, // <
    Le, // <=
}

#[derive(Debug, Clone)]
pub enum LogicalOperator {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Neg,
    Not,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Void,
    /// null.
    Null,
    /// true or false.
    Bool(bool),
    /// Any floating point number.
    Number(f64),
    /// Any quoted string.
    String(String),
    /// An array of values
    Array(Vec<Expr>),
    /// An dictionary mapping keys and values.
    Object(HashMap<String, Expr>),
}
